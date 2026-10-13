from pathlib import Path
import hashlib
import json
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

from check_cli_release import build_bundle, build_archive, check_bundle, check_document_links
from check_third_party import MANIFEST, validate, package_ids, check_dependencies, check_rust_version
from test_check_cli_release import fixture


class ThirdPartyTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        fixture(self.root)

    def manifest(self):
        return json.loads((self.root / MANIFEST).read_text())

    def save(self, data):
        (self.root / MANIFEST).write_text(json.dumps(data))

    def test_license_links_and_payload_survive_zip(self):
        import zipfile
        data = self.manifest()
        name = data["files"][0]["path"]
        (self.root / "THIRD_PARTY_NOTICES.md").write_text(f"[License]({name})\n[Notice](NOTICE)\n[Inventory]({MANIFEST})")
        binary = self.root / "veil"
        binary.write_bytes(b"synthetic binary")
        bundle = self.root / "bundle"
        build_bundle(self.root, binary, bundle)
        build_archive(bundle, "veil", self.root / "candidate.zip")
        with zipfile.ZipFile(self.root / "candidate.zip") as archive:
            self.assertEqual(archive.read(name), (self.root / name).read_bytes())
            self.assertIn("NOTICE", archive.namelist())
            self.assertIn("THIRD_PARTY_NOTICES.md", archive.namelist())
        check_bundle(bundle, "veil")

    def test_modified_license_rejected_before_output(self):
        name = self.manifest()["files"][0]["path"]
        (self.root / name).write_text("changed copyright")
        binary = self.root / "veil"
        binary.write_bytes(b"binary")
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            build_bundle(self.root, binary, self.root / "bundle")
        self.assertFalse((self.root / "bundle").exists())

    def test_missing_notice_and_license_are_rejected(self):
        # Rename fixture files; no real repository file is removed.
        for name in ["NOTICE", self.manifest()["files"][0]["path"]]:
            path = self.root / name
            parked = self.root / "parked"
            path.rename(parked)
            with self.assertRaises(ValueError):
                check_document_links(self.root)
            parked.rename(path)

    def test_extra_private_file_or_directory_is_rejected(self):
        for name in ["third_party/private.txt", "third_party/.private/notes.txt"]:
            with tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                fixture(root)
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("private marker")
                with self.assertRaisesRegex(ValueError, "unapproved third-party entry"):
                    validate(root)

    def test_manifest_cannot_select_private_or_executable_files(self):
        original = self.manifest()
        for name in [".private/notes.txt", "third_party/licenses/sample-1.0/../../private.txt",
                     "third_party/licenses/sample-1.0/.hidden.txt", "third_party/licenses/sample-1.0/run.py",
                     "third_party/licenses/sample-1.0/SECRET.env", "third_party/licenses/sample-1.0/private.txt", "/tmp/private.txt",
                     "third_party/sources/private.crate", "third_party/licenses/sample-1.0/%2e.txt",
                     "third_party//licenses/sample-1.0/LICENSE.txt"]:
            data = json.loads(json.dumps(original))
            data["files"][0]["path"] = name
            self.save(data)
            with self.subTest(name=name), self.assertRaises(ValueError):
                validate(self.root)

    def test_redirected_license_rejected(self):
        name = self.manifest()["files"][0]["path"]
        path = self.root / name
        parked = self.root / "original"
        path.rename(parked)
        path.symlink_to(parked)
        with self.assertRaisesRegex(ValueError, "symlink"):
            validate(self.root)

    def test_broken_license_link_and_private_link_are_rejected(self):
        for target in ["third_party/licenses/sample-1.0/MISSING.txt", ".private/notes.md"]:
            (self.root / "THIRD_PARTY_NOTICES.md").write_text(f"[License]({target})")
            with self.assertRaises(ValueError):
                check_document_links(self.root)

    def test_missing_index_navigation_is_rejected(self):
        (self.root / "THIRD_PARTY_NOTICES.md").write_text("Attribution documents are somewhere in this archive")
        with self.assertRaisesRegex(ValueError, "index must link every"):
            check_document_links(self.root)

    def test_duplicate_or_cross_package_references_are_rejected(self):
        original = self.manifest()
        for kind in ["package", "file", "reference"]:
            data = json.loads(json.dumps(original))
            if kind == "package":
                data["packages"].append(data["packages"][0])
            elif kind == "file":
                data["files"].append(data["files"][0])
            else:
                data["packages"][0]["name"] = "other"
            self.save(data)
            with self.assertRaises(ValueError):
                validate(self.root)

    def test_source_archive_is_mandatory_and_bound_to_lock_checksum(self):
        data = self.manifest()
        p = data["packages"][0]
        p["name"] = "colored"
        name = "third_party/licenses/colored-1.0/LICENSE.txt"
        old = self.root / data["files"][0]["path"]
        new = self.root / name
        new.parent.mkdir()
        old.rename(new)
        # Empty leftover directory is deliberately excluded from the reviewed set.
        old.parent.rename(self.root / "old-sample")
        data["files"][0]["path"] = name
        p["license_files"] = [name]
        self.save(data)
        with self.assertRaisesRegex(ValueError, "required source archive"):
            validate(self.root)
        source = "third_party/sources/colored-1.0.crate"
        path = self.root / source
        path.parent.mkdir()
        path.write_bytes(b"source archive fixture")
        sha = hashlib.sha256(path.read_bytes()).hexdigest()
        p["source_archive"] = source
        data["files"].append({"path": source, "sha256": sha, "origin": "https://example.com/source.crate"})
        self.save(data)
        with self.assertRaisesRegex(ValueError, "Cargo.lock checksum"):
            validate(self.root)
        p["checksum"] = sha
        self.save(data)
        validate(self.root)

    def test_lock_change_and_mismatched_inventory_are_rejected(self):
        data = self.manifest()
        lock = self.root / "Cargo.lock"
        lock.write_text('[[package]]\nname="sample"\nversion="1.0"\nchecksum="' + "a" * 64 + '"\nsource="registry+https://github.com/rust-lang/crates.io-index"\n')
        data["cargo_lock_sha256"] = hashlib.sha256(lock.read_bytes()).hexdigest()
        self.save(data)
        validate(self.root, check_lock=True)
        data["packages"][0]["checksum"] = "b" * 64
        self.save(data)
        with self.assertRaisesRegex(ValueError, "differs from Cargo.lock"):
            validate(self.root, check_lock=True)
        lock.write_text(lock.read_text() + "\n# change\n")
        with self.assertRaisesRegex(ValueError, "Cargo.lock changed"):
            validate(self.root, check_lock=True)

    def test_dependency_graph_handles_duplicate_target_entries(self):
        tree = "veil-cli v0.17.0 (/repo/crates/veil-cli)\nsample v1.0\nsample v1.0 (*)\nsample v2.0\n"
        self.assertEqual(package_ids(tree), {("sample", "1.0"), ("sample", "2.0")})
        with self.assertRaises(ValueError):
            package_ids("unknown format")

    def test_missing_or_added_dependency_requires_refresh(self):
        data = self.manifest()
        for tree in ["sample v1.0\n", "sample v1.0\nnew-crate v2.0\n", "sample v2.0\n"]:
            with patch("check_third_party.validate", return_value=data), patch(
                "check_third_party.subprocess.run", return_value=SimpleNamespace(stdout=tree)
            ):
                if tree == "sample v1.0\n":
                    check_dependencies(self.root)
                else:
                    with self.assertRaisesRegex(ValueError, "dependency graph changed"):
                        check_dependencies(self.root)

    def test_unknown_rust_version_and_missing_std_notice_rejected(self):
        with self.assertRaisesRegex(ValueError, "no reviewed"):
            check_rust_version(self.root, "9.0.0")
        data = self.manifest()
        data["rust_versions"] = ["1.92.0"]
        self.save(data)
        with self.assertRaisesRegex(ValueError, "missing Rust"):
            validate(self.root)

    def test_native_component_cannot_reuse_another_owners_license(self):
        data = self.manifest()
        data["native_components"] = [{"id": "musl-1.2.3", "license": "MIT",
                                      "license_files": [data["files"][0]["path"]]}]
        self.save(data)
        with self.assertRaisesRegex(ValueError, "native component license"):
            validate(self.root)


if __name__ == "__main__":
    unittest.main()
