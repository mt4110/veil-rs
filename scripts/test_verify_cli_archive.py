from pathlib import Path
import hashlib
import json
import os
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import zipfile

from check_cli_release import build_archive, build_bundle
from test_check_cli_release import fixture
from verify_cli_archive import verify_archive, write_report, source_fingerprint


class ArchiveVerificationTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        fixture(self.root)

    def candidate(self, binary_name="veil"):
        binary = self.root / binary_name
        binary.write_bytes(b"synthetic test binary")
        binary.chmod(0o755)
        bundle = self.root / "bundle"
        build_bundle(self.root, binary, bundle)
        archive = self.root / "candidate.zip"
        build_archive(bundle, binary_name, archive)
        return archive

    def rewrite(self, archive, transform):
        other = self.root / "modified.zip"
        with zipfile.ZipFile(archive) as source, zipfile.ZipFile(other, "x") as target:
            for entry in source.infolist():
                content = source.read(entry)
                for info, data in transform(entry, content):
                    target.writestr(info, data)
        return other

    def test_native_smoke_runs_extracted_binary_and_records_license_counts(self):
        archive = self.candidate()
        with patch("verify_cli_archive.check_binary") as smoke, patch("verify_cli_archive.inspect_linux_binary", return_value={}):
            report = verify_archive(self.root, archive, "veil", "0.17.0")
            invoked = smoke.call_args.args[0]
            self.assertNotEqual(invoked, self.root / "veil")
            self.assertEqual(invoked.name, "veil")
            self.assertEqual(report["status"], "passed")
            self.assertEqual(report["third_party_packages"], 1)
            self.assertEqual(report["third_party_payloads"], 1)
            self.assertEqual(report["archive_sha256"], hashlib.sha256(archive.read_bytes()).hexdigest())
            self.assertEqual(report["checksum_entries"], report["archive_entries"] - 1)

    def test_windows_binary_name_uses_same_allowlist_and_smoke(self):
        archive = self.candidate("veil.exe")
        with patch("verify_cli_archive.check_binary") as smoke, patch("verify_cli_archive.inspect_linux_binary") as elf, patch("verify_cli_archive.inspect_windows_binary", return_value={"machine": "x86_64"}):
            report = verify_archive(self.root, archive, "veil.exe", "0.17.0")
            self.assertEqual(smoke.call_args.args[0].name, "veil.exe")
            self.assertEqual(report["windows_pe"]["machine"], "x86_64")
            self.assertIsNone(report["linux_elf"])
            elf.assert_not_called()

    def test_missing_notice_duplicate_and_private_entries_rejected_before_execution(self):
        import warnings
        for kind in ["missing", "duplicate", "private", "traversal"]:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as scratch:
                original_root = self.root
                self.root = Path(scratch)
                fixture(self.root)
                archive = self.candidate()
                with warnings.catch_warnings():
                    warnings.simplefilter("ignore", UserWarning)
                    def transform(entry, content):
                        if entry.filename != "NOTICE":
                            return [(entry, content)]
                        if kind == "missing":
                            return []
                        if kind == "duplicate":
                            return [(entry, content), (entry, content)]
                        name = ".private/notes.txt" if kind == "private" else "../outside.txt"
                        return [(entry, content), (zipfile.ZipInfo(name), b"private marker")]
                    changed = self.rewrite(archive, transform)
                with patch("verify_cli_archive.check_binary") as smoke, self.assertRaises(ValueError):
                    verify_archive(self.root, changed, "veil", "0.17.0")
                smoke.assert_not_called()
                self.root = original_root

    def test_license_tampering_is_rejected_before_execution(self):
        archive = self.candidate()
        changed = self.rewrite(archive, lambda entry, data: [(entry, b"changed license" if entry.filename.endswith("LICENSE.txt") else data)])
        with patch("verify_cli_archive.check_binary") as smoke, self.assertRaises(ValueError):
            verify_archive(self.root, changed, "veil", "0.17.0")
        smoke.assert_not_called()

    def test_self_consistent_old_documents_are_rejected(self):
        archive = self.candidate()
        # The old ZIP is internally consistent but no longer matches this candidate.
        (self.root / "README.md").write_text("Updated current README")
        with patch("verify_cli_archive.check_binary") as smoke, self.assertRaisesRegex(ValueError, "differs from the current"):
            verify_archive(self.root, archive, "veil", "0.17.0")
        smoke.assert_not_called()

    def test_symlink_zip_entry_rejected(self):
        archive = self.candidate()
        def transform(entry, data):
            if entry.filename == "veil":
                entry.external_attr = 0o120777 << 16
            return [(entry, data)]
        changed = self.rewrite(archive, transform)
        with self.assertRaisesRegex(ValueError, "regular file"):
            verify_archive(self.root, changed, "veil", "0.17.0")

    @unittest.skipIf(os.name == "nt", "Unix executable permission")
    def test_executable_permission_loss_is_rejected(self):
        archive = self.candidate()
        def transform(entry, data):
            if entry.filename == "veil":
                entry.external_attr = 0o100644 << 16
            return [(entry, data)]
        changed = self.rewrite(archive, transform)
        with self.assertRaisesRegex(ValueError, "executable permission"):
            verify_archive(self.root, changed, "veil", "0.17.0")

    def test_failed_native_execution_cannot_be_reported_as_passed(self):
        archive = self.candidate()
        with patch("verify_cli_archive.check_binary", side_effect=OSError("wrong OS binary")), self.assertRaises(OSError):
            verify_archive(self.root, archive, "veil", "0.17.0")

    def test_existing_report_is_preserved(self):
        path = self.root / "record.json"
        path.write_text("existing record")
        with self.assertRaises(FileExistsError):
            write_report(path, {"status": "passed"})
        self.assertEqual(path.read_text(), "existing record")

    def test_static_candidate_requires_reviewed_runtime_notices(self):
        archive = self.candidate()
        with patch("verify_cli_archive.platform.system", return_value="Linux"), patch(
            "verify_cli_archive.check_binary"
        ), patch("verify_cli_archive.inspect_linux_binary", return_value={"static_verified": True}):
            with self.assertRaisesRegex(ValueError, "runtime license packet is missing"):
                verify_archive(self.root, archive, "veil", "0.17.0", require_static=True)

    def test_git_autocrlf_does_not_change_license_originals_or_lock(self):
        # A disposable Git index exercises conversion without changing the user's index.
        attributes = Path(__file__).resolve().parents[1] / ".gitattributes"
        (self.root / ".gitattributes").write_bytes(attributes.read_bytes())
        original = b"copyright original\nsecond line\r\n"
        name = "third_party/licenses/sample-1.0/LICENSE.txt"
        (self.root / name).write_bytes(original)
        (self.root / "Cargo.lock").write_bytes(b"lock line\n")
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        commands = [["config", "core.autocrlf", "true"], ["add", "--", ".gitattributes", "Cargo.lock", name]]
        for command in commands:
            subprocess.run(["git", *command], cwd=self.root, capture_output=True, check=True)
        for file, expected in [(name, original), ("Cargo.lock", b"lock line\n")]:
            subprocess.run(["git", "checkout-index", "--prefix=checked/", "--", file], cwd=self.root, check=True)
            self.assertEqual((self.root / "checked" / file).read_bytes(), expected)

    def test_source_fingerprint_covers_embedded_rules_and_ignores_private_files(self):
        (self.root / "Cargo.lock").write_text("lock")
        script = self.root / "scripts/verify_cli_archive.py"
        script.parent.mkdir()
        script.write_text("verification source")
        rules = self.root / "crates/veil/rules/default/00_manifest.toml"
        rules.parent.mkdir(parents=True)
        rules.write_text("rule asset")
        before = source_fingerprint(self.root)
        private = self.root / ".private/notes.txt"
        private.parent.mkdir()
        private.write_text("must not affect the public source identity")
        self.assertEqual(source_fingerprint(self.root), before)
        rules.write_text("updated rule asset")
        self.assertNotEqual(source_fingerprint(self.root), before)


if __name__ == "__main__":
    unittest.main()
