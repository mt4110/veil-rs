from pathlib import Path
import hashlib
import json
import os
import zipfile
import tempfile
import unittest

from check_cli_release import (DOCUMENTS, PACKAGES, ALIGNED_PACKAGES, release_documents, build_bundle, build_archive, check_bundle, check_document_links, check_metadata, check_macos_linkage)


def fixture(root, skip=()):
    (root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.17.0"\nrust-version = "1.92.0"\n')
    (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.92.0"\n')
    (root / "flake.nix").write_text('{ pname = "veil"; version = "0.17.0"; }')
    for name in ALIGNED_PACKAGES:
        path = root / "crates" / name
        path.mkdir(parents=True)
        (path / "Cargo.toml").write_text('[package]\nversion = "0.17.0"\nrust-version.workspace = true\n')
    for name in DOCUMENTS:
        if name in skip:
            continue
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(name, encoding="utf-8")
    if "third_party/manifest.json" not in skip:
        license_name = "third_party/licenses/sample-1.0/LICENSE.txt"
        path = root / license_name
        path.parent.mkdir(parents=True)
        path.write_text("Sample copyright and license\n")
        checksum = "a" * 64
        (root / "third_party/manifest.json").write_text(json.dumps({
            "schemaVersion": "veil-third-party-v1", "profile": "veil-cli/default/normal,build/all-targets",
            "cargo_lock_sha256": checksum, "rust_versions": [],
            "packages": [{"name": "sample", "version": "1.0", "license": "MIT", "checksum": checksum,
                          "license_files": [license_name]}],
            "files": [{"path": license_name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                       "origin": "https://example.com/published-sample/LICENSE"}],
        }))
        if "THIRD_PARTY_NOTICES.md" not in skip:
            (root / "THIRD_PARTY_NOTICES.md").write_text(f"[License]({license_name})\n[Notice](NOTICE)\n[Inventory](third_party/manifest.json)")


class ReleaseTests(unittest.TestCase):
    def test_macos_system_libraries_pass(self):
        check_macos_linkage("veil:\n\t/usr/lib/libSystem.B.dylib (compatibility version 1)\n")

    def test_macos_machine_specific_libraries_are_rejected(self):
        for prefix in ["/nix/store/", "/opt/homebrew/", "@rpath/"]:
            with self.assertRaises(ValueError):
                check_macos_linkage(f"veil:\n\t{prefix}libexample.dylib (compatibility version 1)\n")

    def test_missing_macos_dependency_information_is_rejected(self):
        for output in ["", "veil:\n"]:
            with self.assertRaises(ValueError):
                check_macos_linkage(output)

    def test_supported_metadata(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            self.assertEqual(check_metadata(root), "0.17.0")

    def test_version_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            path = root / "crates/veil-core/Cargo.toml"
            path.write_text(path.read_text().replace("0.17.0", "0.18.0"))
            with self.assertRaises(ValueError):
                check_metadata(root)

    def test_lsp_version_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            path = root / "crates/veil-lsp/Cargo.toml"
            path.write_text(path.read_text().replace("0.17.0", "0.18.0"))
            with self.assertRaisesRegex(ValueError, "veil-lsp: version differs"):
                check_metadata(root)

    def test_nix_cli_version_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            path = root / "flake.nix"
            path.write_text(path.read_text().replace("0.17.0", "0.18.0"))
            with self.assertRaisesRegex(ValueError, "Nix CLI version differs"):
                check_metadata(root)

    def test_prerelease_version_is_aligned_across_targets(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            paths = [root / "Cargo.toml", root / "flake.nix", *(root / "crates" / name / "Cargo.toml" for name in ALIGNED_PACKAGES)]
            for path in paths:
                path.write_text(path.read_text().replace("0.17.0", "0.21.0-rc.1"))
            self.assertEqual(check_metadata(root), "0.21.0-rc.1")

    def test_toolchain_mismatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.82.0"\n')
            with self.assertRaises(ValueError):
                check_metadata(root)

    def test_bundle_only_contains_allowlisted_files(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            binary = root / "compiled-veil"
            binary.write_bytes(b"synthetic test binary")
            (root / "private-data.txt").write_text("must not be packaged")
            private = root / ".private/sales/notes.md"
            private.parent.mkdir(parents=True)
            private.write_text("private-content-marker")
            other = root / "docs/pr/not-for-release.md"
            other.parent.mkdir(parents=True)
            other.write_text("private-content-marker")
            (root / "README.md").write_text("[Tutorial](docs/cli/getting-started.md)")
            (root / "docs/cli/getting-started.md").write_text("[Guidelines](usage-guidelines.md)")
            output = root / "bundle"
            build_bundle(root, binary, output)
            self.assertEqual({p.relative_to(output).as_posix() for p in output.rglob("*") if p.is_file()}, set(release_documents(root)) | {"veil", "SHA256SUMS"})
            self.assertEqual(len((output / "SHA256SUMS").read_text().splitlines()), len(release_documents(root)) + 1)
            for name in DOCUMENTS:
                self.assertNotIn("private-content-marker", (output / name).read_text())
            check_document_links(output)

    def test_existing_bundle_is_preserved(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            binary = root / "veil.exe"
            binary.write_bytes(b"synthetic test binary")
            output = root / "bundle"
            output.mkdir()
            (output / "keep.txt").write_text("keep")
            with self.assertRaises(FileExistsError):
                build_bundle(root, binary, output)
            self.assertEqual((output / "keep.txt").read_text(), "keep")

    def test_missing_document_is_rejected_before_creating_bundle(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root, skip=("docs/cli/getting-started.md",))
            binary = root / "veil"
            binary.write_bytes(b"test binary")
            output = root / "bundle"
            with self.assertRaises(ValueError):
                build_bundle(root, binary, output)
            self.assertFalse(output.exists())

    def test_private_or_unlisted_local_links_do_not_expand_bundle(self):
        links = [".private/sales/notes.md", "docs/pr/notes.md", "../private.md",
                 "%2eprivate/sales/notes.md", "file:///private.md", "/private.md",
                 r".private\sales\notes.md", "docs/cli/absent.md"]
        for target in links:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                fixture(root)
                private = root / ".private/sales/notes.md"
                private.parent.mkdir(parents=True)
                private.write_text("must not be copied")
                (root / "README.md").write_text(f"[Bad]({target})")
                binary = root / "veil"
                binary.write_bytes(b"test binary")
                with self.assertRaises(ValueError):
                    build_bundle(root, binary, root / "bundle")
                self.assertFalse((root / "bundle").exists())

    def test_reference_and_html_links_are_checked(self):
        for content in ["[bad][private]\n[private]: .private/notes.md", "<a href='.private/notes.md'>bad</a>", "<img src='.private/notes.png'>", "[bad][missing]"]:
            with self.subTest(content=content), tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                fixture(root)
                (root / "README.md").write_text(content)
                with self.assertRaises(ValueError):
                    check_document_links(root)

    def test_relative_links_and_real_fragments_pass(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            (root / "README.md").write_text("[Guide](docs/cli/usage-guidelines.md#safe-output)\n[Again][guide]\n[guide]: docs/cli/usage-guidelines.md#safe-output")
            (root / "docs/cli/usage-guidelines.md").write_text("# Safe Output\n[Home](../../README.md)")
            check_document_links(root)
            (root / "README.md").write_text("[Bad](docs/cli/usage-guidelines.md#missing)")
            with self.assertRaises(ValueError):
                check_document_links(root)

    def test_example_links_do_not_trigger_packaging(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            (root / "README.md").write_text("```md\n[example](.private/data.md)\n```\n`[example](.private/data.md)`\n[Online](https://example.com/guide)")
            check_document_links(root)

    def test_malformed_inline_links_and_example_only_anchors_are_rejected(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root)
            for content in ["[Bad](.private/my notes.md)", "[Bad](docs/cli/usage-guidelines.md#fake)"]:
                (root / "README.md").write_text(content)
                (root / "docs/cli/usage-guidelines.md").write_text("```md\n# Fake\n```\n# Real")
                with self.assertRaises(ValueError):
                    check_document_links(root)

    @unittest.skipIf(os.name == "nt", "symlink fixture requires Unix")
    def test_symlink_document_and_parent_are_rejected(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            fixture(root, skip=("docs/cli/getting-started.md",))
            private = root / "private.txt"
            private.write_text("must not be copied")
            (root / "docs/cli/getting-started.md").symlink_to(private)
            with self.assertRaises(ValueError):
                check_document_links(root)
        with tempfile.TemporaryDirectory() as scratch, tempfile.TemporaryDirectory() as outside:
            root = Path(scratch)
            fixture(root, skip=tuple(name for name in DOCUMENTS if name.startswith("docs/")))
            other = Path(outside)
            fixture(other)
            (root / "docs").symlink_to(other / "docs", target_is_directory=True)
            with self.assertRaises(ValueError):
                check_document_links(root)

    def make_bundle(self, root, name="veil"):
        fixture(root)
        binary = root / name
        binary.write_bytes(b"test binary")
        output = root / "bundle"
        build_bundle(root, binary, output)
        return output

    def test_archive_contents_are_allowlisted_and_checksums_cover_docs(self):
        for binary_name in ["veil", "veil.exe"]:
            with self.subTest(binary_name=binary_name), tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                bundle = self.make_bundle(root, binary_name)
                archive = root / "candidate.zip"
                build_archive(bundle, binary_name, archive)
                with zipfile.ZipFile(archive) as zipped:
                    self.assertEqual(set(zipped.namelist()), set(release_documents(root)) | {binary_name, "SHA256SUMS"})
                    for line in zipped.read("SHA256SUMS").decode().splitlines():
                        digest, name = line.split("  ", 1)
                        self.assertEqual(hashlib.sha256(zipped.read(name)).hexdigest(), digest)
                    extracted = root / "extracted"
                    zipped.extractall(extracted)
                check_bundle(extracted, binary_name)
                self.assertTrue((extracted / "docs/cli/getting-started.md").is_file())
                self.assertTrue((extracted / "docs/cli/usage-guidelines.md").is_file())

    def test_private_extra_bundle_entries_prevent_archive_creation(self):
        for relative in [".private/sales/note.md", "docs/pr/private.md", "scan-output.json"]:
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as scratch:
                root = Path(scratch)
                bundle = self.make_bundle(root)
                extra = bundle / relative
                extra.parent.mkdir(parents=True, exist_ok=True)
                extra.write_text("must not be archived")
                archive = root / "candidate.zip"
                with self.assertRaises(ValueError):
                    build_archive(bundle, "veil", archive)
                self.assertFalse(archive.exists())

    def test_modified_bundle_document_prevents_archive_creation(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            bundle = self.make_bundle(root)
            (bundle / "docs/cli/getting-started.md").write_text("changed")
            with self.assertRaises(ValueError):
                build_archive(bundle, "veil", root / "candidate.zip")
            self.assertFalse((root / "candidate.zip").exists())

    def test_existing_archive_is_preserved(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            bundle = self.make_bundle(root)
            archive = root / "existing.zip"
            archive.write_bytes(b"previous archive")
            with self.assertRaises(FileExistsError):
                build_archive(bundle, "veil", archive)
            self.assertEqual(archive.read_bytes(), b"previous archive")

    def test_archive_cannot_be_written_inside_bundle(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            bundle = self.make_bundle(root)
            archive = bundle / "candidate.zip"
            with self.assertRaises(ValueError):
                build_archive(bundle, "veil", archive)
            self.assertFalse(archive.exists())

    @unittest.skipIf(os.name == "nt", "symlink fixture requires Unix")
    def test_symlink_in_bundle_is_not_archived(self):
        with tempfile.TemporaryDirectory() as scratch:
            root = Path(scratch)
            bundle = self.make_bundle(root)
            (bundle / "alias").symlink_to(root / "README.md")
            with self.assertRaises(ValueError):
                build_archive(bundle, "veil", root / "candidate.zip")
            self.assertFalse((root / "candidate.zip").exists())


if __name__ == "__main__":
    unittest.main()
