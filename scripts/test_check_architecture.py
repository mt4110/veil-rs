import unittest

from pathlib import Path
import tempfile

from check_architecture import check_metadata, check_tree, check_public_scope, EXCLUDED_RELEASE_PATHS


def metadata(extra=None):
    packages = []
    for name in ("veil-config", "veil-core", "veil-cli"):
        dependencies = []
        if name != "veil-config":
            dependencies.append({"name": "veil-config", "kind": None, "path": "/config"})
        if name == "veil-cli" and extra:
            dependencies.append(extra)
        packages.append({"id": name, "name": name, "dependencies": dependencies})
    return {"packages": packages, "workspace_default_members": [p["id"] for p in packages]}


class ArchitectureTests(unittest.TestCase):
    def test_local_profile_passes(self):
        self.assertEqual(check_metadata(metadata()), [])

    def test_optional_network_dependency_is_rejected(self):
        dep = {"name": "reqwest", "kind": None, "path": None, "optional": True}
        self.assertTrue(check_metadata(metadata(dep)))

    def test_reverse_local_edge_is_rejected(self):
        data = metadata()
        data["packages"][0]["dependencies"].append({"name": "veil-cli", "kind": None, "path": "/cli"})
        self.assertTrue(check_metadata(data))

    def test_unrelated_default_member_is_rejected(self):
        data = metadata()
        data["workspace_default_members"].pop()
        self.assertTrue(check_metadata(data))

    def test_transitive_and_build_network_dependencies_are_rejected(self):
        self.assertTrue(check_tree("veil-cli", "veil-cli v0.17.0\nwrapper v1.0.0\nreqwest v0.12.0\n"))

    def test_git_network_features_are_rejected(self):
        for default, features in [(True, []), (False, ["https"]), (False, ["ssh"])]:
            dep = {"name": "git2", "kind": None, "path": None, "uses_default_features": default, "features": features}
            self.assertTrue(check_metadata(metadata(dep)))

    def test_native_git_transport_dependencies_are_rejected(self):
        self.assertTrue(check_tree("veil-cli", "openssl-sys v0.9.0\nlibssh2-sys v0.3.0\n"))

    def test_vulnerable_or_unknown_libgit2_versions_are_rejected(self):
        for version in ["0.18.3+1.9.2", "0.18.5+1.9.4", "0.18.8"]:
            self.assertTrue(check_tree("veil-cli", f"libgit2-sys v{version}\n"))

    def test_fixed_libgit2_version_passes(self):
        self.assertEqual(check_tree("veil-cli", "libgit2-sys v0.18.8+1.9.7\n"), [])


class PublicScopeTests(unittest.TestCase):
    def test_existing_verify_api_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/veil-core/src/lib.rs"
            path.parent.mkdir(parents=True)
            path.write_text("pub mod verify;\npub use verify::VerifyOptions;\n")
            self.assertEqual(check_public_scope(root), [])

    def test_deferred_sources_and_contracts_are_rejected(self):
        for name in EXCLUDED_RELEASE_PATHS:
            with self.subTest(path=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("deferred")
                self.assertTrue(check_public_scope(root))

    def test_shared_dispatch_cannot_reintroduce_new_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "crates/veil-cli/src/main.rs"
            path.parent.mkdir(parents=True)
            path.write_text("Commands::Decide(args)")
            self.assertTrue(check_public_scope(root))


if __name__ == "__main__":
    unittest.main()
