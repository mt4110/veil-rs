"""Installer behavior against native fixture executables or an extracted CLI bundle."""
from pathlib import Path
import hashlib
import os
import platform
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]


def checksums(packet):
    names = sorted(p.relative_to(packet).as_posix() for p in packet.rglob('*')
                   if p.is_file() and p.name != 'SHA256SUMS')
    (packet / 'SHA256SUMS').write_text(''.join(
        hashlib.sha256((packet / n).read_bytes()).hexdigest() + '  ' + n + '\n'
        for n in names))


def fixture(packet, binary):
    packet.mkdir()
    shutil.copy2(binary, packet / 'veil')
    shutil.copy2(ROOT / 'install.sh', packet / 'install.sh')
    for name in ['LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'THIRD_PARTY_NOTICES.md',
                 'third_party/manifest.json', 'docs/cli/install.md', 'docs/cli/install.en.md']:
        p = packet / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text('Synthetic installer test packet, not distribution licenses.\n')
    checksums(packet)


class InstallerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.scratch = tempfile.TemporaryDirectory(prefix='veil-install-test-')
        cls.addClassCleanup(cls.scratch.cleanup)
        cls.root = Path(cls.scratch.name).resolve()
        # Native C fixtures test version transitions without changing any released
        # binary or requiring Rust. Distribution checks additionally use real Veil.
        for version in ['1.2.2', '1.2.3']:
            source = cls.root / (version + '.c')
            source.write_text('#include <stdio.h>\nint main(void) { puts("veil ' + version + '"); return 0; }\n')
            subprocess.run(['cc', str(source), '-o', str(cls.root / version)], check=True,
                           capture_output=True, timeout=60)
        cls.old = cls.root / 'old'
        fixture(cls.old, cls.root / '1.2.2')
        supplied = os.environ.get('VEIL_INSTALL_TEST_BUNDLE')
        cls.template = Path(supplied).resolve() if supplied else cls.root / 'current'
        if not supplied:
            fixture(cls.template, cls.root / '1.2.3')
        cls.version = subprocess.check_output([str(cls.template / 'veil'), '--version'], text=True).strip()
        assert (cls.template / 'install.sh').read_bytes() == (ROOT / 'install.sh').read_bytes()
        cls.version_number = cls.version.removeprefix('veil ')
        cls.target = 'aarch64-darwin' if platform.system() == 'Darwin' else 'x86_64-linux'
        cls.tools = cls.root / 'tools'
        cls.tools.mkdir()
        # Installer subprocesses cannot find cargo, rustc, cc, curl or sudo.
        for name in ['sh', 'awk', 'sha256sum', 'shasum', 'od', 'tr', 'cut', 'uname',
                     'dirname', 'readlink', 'cmp', 'mkdir', 'mktemp', 'cp', 'mv', 'ln', 'sed']:
            path = shutil.which(name)
            if path:
                (cls.tools / name).symlink_to(path)

    def setUp(self):
        self.case_dir = Path(tempfile.mkdtemp(dir=self.root)).resolve()
        self.prefix = self.case_dir / "user's prefix with spaces"
        self.home = self.case_dir / 'home'
        self.home.mkdir()
        for name in ['.zshrc', '.zprofile', '.bashrc', '.bash_profile', '.profile']:
            (self.home / name).write_text('# preserved installer test configuration\n')
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(('VEIL_', 'LC_')) and k not in ['LANG', 'ENV', 'BASH_ENV', 'CDPATH']}
        self.env.update(HOME=str(self.home), PATH=str(self.tools), LANG='C')

    def run_install(self, packet=None, *args, code=0, env=None):
        result = subprocess.run(['/bin/sh', str((packet or self.template) / 'install.sh'),
                                 '--prefix', str(self.prefix), *args], env=env or self.env,
                                capture_output=True, text=True, timeout=120)
        self.assertEqual(result.returncode, code, result.stdout + result.stderr)
        for p in self.home.iterdir():
            self.assertEqual(p.read_text(), '# preserved installer test configuration\n')
        return result

    def entry(self):
        return self.prefix / 'bin/veil'

    def destination(self):
        return self.prefix / 'share/veil' / self.version_number / self.target

    def mutable_packet(self):
        p = self.case_dir / 'packet'
        shutil.copytree(self.template, p)
        return p

    def test_install_preserves_packet_and_is_idempotent(self):
        self.run_install(None, '--lang', 'en')
        self.assertEqual(self.entry().resolve(), self.destination() / 'veil')
        for row in (self.template / 'SHA256SUMS').read_text().splitlines():
            _, name = row.split('  ', 1)
            self.assertEqual((self.template / name).read_bytes(), (self.destination() / name).read_bytes())
        previous = self.entry().lstat()
        self.run_install(None, '--lang', 'ja')
        self.assertEqual(self.entry().lstat().st_ino, previous.st_ino)

    def test_new_shell_uses_printed_path_line_in_both_languages(self):
        for language in ['ja', 'en']:
            result = self.run_install(None, '--lang', language)
            path_line = next(l for l in result.stdout.splitlines() if l.startswith('export PATH='))
            for shell in ['bash', 'zsh']:
                executable = shutil.which(shell)
                if not executable:
                    if shell == 'zsh' and platform.system() == 'Linux':
                        continue  # Ubuntu's standard interactive shell is bash.
                    self.fail('Required native shell is unavailable: ' + shell)
                settings = self.case_dir / (language + '-' + shell)
                settings.mkdir()
                startup = settings / ('.zshrc' if shell == 'zsh' else '.bashrc')
                startup.write_text(path_line + '\n')
                env = {**self.env, 'ZDOTDIR': str(settings), 'HISTFILE': '/dev/null'}
                # Load an isolated interactive startup file in a new shell;
                # global/user profiles are not imported by these test commands.
                flags = ['--noprofile', '--rcfile', str(startup), '-ic'] if shell == 'bash' else ['-d', '-ic']
                r = subprocess.run([executable, *flags, 'command -v veil; veil --version'], env=env,
                                   capture_output=True, text=True, check=True, timeout=30)
                self.assertEqual(r.stdout.splitlines(), [str(self.entry()), self.version])
        result = self.run_install(None, '--lang', 'en', env={**self.env, 'PATH': str(self.prefix / 'bin') + ':' + str(self.tools)})
        self.assertIn('PATH already contains', result.stdout)

    def test_default_prefix(self):
        result = subprocess.run(['/bin/sh', str(self.template / 'install.sh'), '--lang', 'en'],
                                env=self.env, capture_output=True, text=True, timeout=120)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.home / '.local/bin/veil').is_symlink())
        # The default prefix is the sole addition; existing dotfiles are intact.
        for p in self.home.glob('.*rc'):
            self.assertEqual(p.read_text(), '# preserved installer test configuration\n')

    def test_managed_version_requires_explicit_activation_and_retains_old(self):
        self.run_install(self.old)
        old = self.entry().resolve()
        self.run_install(None, code=2)
        self.assertEqual(self.entry().resolve(), old)
        self.run_install(None, '--activate')
        self.assertEqual(self.entry().resolve(), self.destination() / 'veil')
        self.assertTrue(old.is_file())
        self.run_install(self.old, '--activate')
        self.assertEqual(self.entry().resolve(), old)

    def test_existing_regular_cli_is_never_replaced(self):
        self.entry().parent.mkdir(parents=True)
        self.entry().write_bytes(b'preserve existing CLI')
        for args in [(), ('--activate',)]:
            self.run_install(None, *args, code=2)
            self.assertEqual(self.entry().read_bytes(), b'preserve existing CLI')
        self.assertFalse((self.prefix / 'share').exists())

    def test_external_and_dangling_links_are_never_replaced(self):
        for kind in ['file', 'directory', 'missing']:
            self.prefix = self.case_dir / kind
            self.entry().parent.mkdir(parents=True)
            outside = self.case_dir / ('outside-' + kind)
            if kind == 'file':
                outside.write_text('preserved')
            if kind == 'directory':
                outside.mkdir()
            self.entry().symlink_to(outside)
            self.run_install(None, '--activate', code=2)
            self.assertEqual(os.readlink(self.entry()), str(outside))

    def test_invalid_checksum_leaves_existing_entry(self):
        self.run_install(self.old)
        old = self.entry().resolve()
        packet = self.mutable_packet()
        (packet / 'NOTICE').write_text('modified')
        self.run_install(packet, '--activate', code=2)
        self.assertEqual(self.entry().resolve(), old)
        self.assertFalse(self.destination().exists())

    def test_manifest_traversal_duplicates_and_missing_licenses_are_rejected(self):
        packet = self.mutable_packet()
        manifest = packet / 'SHA256SUMS'
        original = manifest.read_text()
        (packet / '-').write_text('Never treat this regular file as standard input.\n')
        for content in [original + 'a' * 64 + '  ../outside\n',
                        original + 'a' * 64 + '  -\n',
                        original + original.splitlines()[0] + '\n',
                        '\n'.join(l for l in original.splitlines() if not l.endswith('  LICENSE-MIT')) + '\n']:
            manifest.write_text(content)
            self.run_install(packet, code=2)
            self.assertFalse(self.prefix.exists())

    def test_symlink_packet_member_is_rejected(self):
        packet = self.mutable_packet()
        original = packet / 'NOTICE'
        renamed = packet / 'original-notice'
        original.rename(renamed)
        original.symlink_to(renamed)
        self.run_install(packet, code=2)
        self.assertFalse(self.prefix.exists())

    def test_same_version_different_contents_preserves_active_cli(self):
        self.run_install()
        packet = self.mutable_packet()
        (packet / 'NOTICE').write_text('different but internally valid packet')
        checksums(packet)
        original = self.entry().read_bytes()
        self.run_install(packet, '--activate', code=2)
        self.assertEqual(self.entry().read_bytes(), original)

    def test_damaged_managed_packet_is_not_activated_over(self):
        self.run_install(self.old)
        previous = self.entry().resolve()
        (previous.parent / 'NOTICE').write_text('damaged installed packet')
        self.run_install(None, '--activate', code=2)
        self.assertEqual(self.entry().resolve(), previous)
        self.assertFalse(self.destination().exists())

    def test_nonexecutable_existing_version_is_not_reported_as_success(self):
        self.run_install()
        binary = self.destination() / 'veil'
        binary.chmod(0o600)
        self.run_install(code=2)
        self.assertEqual(self.entry().resolve(), binary)

    def failing_tool(self, name, condition):
        directory = self.case_dir / ('fail-' + name)
        directory.mkdir()
        tool = directory / name
        tool.write_text('#!/bin/sh\n' + condition + '\nexec ' + str(self.tools / name) + ' "$@"\n')
        tool.chmod(0o755)
        return {**self.env, 'PATH': str(directory) + ':' + str(self.tools)}

    def test_partial_copy_failure_preserves_old_cli(self):
        self.run_install(self.old)
        old = self.entry().resolve()
        env = self.failing_tool('cp', 'case "$3" in */NOTICE) exit 1 ;; esac')
        self.run_install(None, '--activate', code=2, env=env)
        self.assertEqual(self.entry().resolve(), old)
        self.assertFalse(self.destination().exists())
        self.assertTrue(list((self.prefix / 'share/veil').glob('.install.*')))

    def test_copy_corruption_is_detected_before_activation(self):
        self.run_install(self.old)
        old = self.entry().resolve()
        env = self.failing_tool('cp', 'case "$3" in */NOTICE) printf corrupt > "$4"; exit 0 ;; esac')
        self.run_install(None, '--activate', code=2, env=env)
        self.assertEqual(self.entry().resolve(), old)

    def test_activation_failure_preserves_old_cli(self):
        self.run_install(self.old)
        old = self.entry().resolve()
        env = self.failing_tool('mv', 'case "$*" in *".veil-activate-"*) exit 1 ;; esac')
        self.run_install(None, '--activate', code=2, env=env)
        self.assertEqual(self.entry().resolve(), old)
        self.assertTrue(self.destination().is_dir())

    def test_unwritable_prefix_preserves_existing_state(self):
        self.run_install(self.old)
        old = self.entry().resolve()
        env = self.failing_tool('mkdir', 'exit 1')
        self.run_install(None, '--activate', code=2, env=env)
        self.assertEqual(self.entry().resolve(), old)

    def test_permission_denied_directory(self):
        if os.geteuid() == 0:
            self.skipTest('root bypasses ordinary filesystem permissions')
        self.prefix.mkdir()
        self.prefix.chmod(0o500)
        try:
            self.run_install(code=2)
            self.assertEqual(list(self.prefix.iterdir()), [])
        finally:
            self.prefix.chmod(0o700)

    def test_symlink_parent_rejected(self):
        outside = self.case_dir / 'outside'
        outside.mkdir()
        self.prefix.symlink_to(outside)
        self.run_install(code=2)
        self.assertEqual(list(outside.iterdir()), [])

    def test_wrong_architecture_is_rejected_before_prefix_creation(self):
        packet = self.mutable_packet()
        with (packet / 'veil').open('r+b') as f:
            f.write(b'not-an-executable-header')
        checksums(packet)
        self.run_install(packet, code=2)
        self.assertFalse(self.prefix.exists())

    def test_language_priority_invalid_values_and_help_are_side_effect_free(self):
        r = self.run_install(None, '--help', env={**self.env, 'LANG': 'ja_JP.UTF-8'})
        self.assertIn('ユーザー領域', r.stdout)
        r = self.run_install(None, '--lang', 'en', '--help', env={**self.env, 'VEIL_LANG': 'invalid'})
        self.assertIn('no network or sudo', r.stdout)
        r = self.run_install(None, '--help', env={**self.env, 'LC_ALL': 'C', 'LANG': 'ja_JP.UTF-8'})
        self.assertIn('no network or sudo', r.stdout)
        r = subprocess.run(['/bin/sh', str(self.template / 'install.sh'), '--help'],
                           env={k: v for k, v in self.env.items() if k != 'HOME'},
                           capture_output=True, text=True, timeout=10)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.run_install(None, '--lang', 'invalid', '--help', code=2)
        self.run_install(None, '--unknown', code=2)
        self.run_install(None, '--prefix', code=2)
        self.assertFalse(self.prefix.exists())


if __name__ == '__main__':
    unittest.main()
