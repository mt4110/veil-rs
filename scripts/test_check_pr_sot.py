from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / 'scripts/check_pr_sot.py'


class SotGuardTests(unittest.TestCase):
    def setUp(self):
        # Preserve isolated fixtures for inspection; never touch the owner checkout.
        self.repo = Path(tempfile.mkdtemp(prefix='veil-sot-test-'))
        self.env = {**os.environ, 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1'}
        self.run_git('init', '-q', '-b', 'main')
        self.run_git('config', 'user.name', 'Fixture')
        self.run_git('config', 'user.email', 'fixture@example.invalid')
        self.run_git('config', 'commit.gpgsign', 'false')
        (self.repo / 'empty-hooks').mkdir()
        self.run_git('config', 'core.hooksPath', str(self.repo / 'empty-hooks'))
        self.write('README.md', 'initial\n', stage=True)
        self.write('docs/pr/PR-1-old.md', 'existing SOT\n', stage=True)
        self.run_git('commit', '-qm', 'initial')
        self.run_git('update-ref', 'refs/remotes/origin/main', 'HEAD')

    def run_git(self, *args, check=True):
        return subprocess.run(['git', *args], cwd=self.repo, text=True, capture_output=True, check=check, env=self.env)

    def write(self, name, content, stage=False):
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        if stage:
            self.run_git('add', name)

    def guard(self, *args):
        return subprocess.run(['python3', str(CHECKER), *args], cwd=self.repo, text=True, capture_output=True, env=self.env)

    def test_original_readme_failure_ci_and_index(self):
        self.write('README.md', 'updated\n', True)
        self.assertEqual(self.guard('--staged').returncode, 1)
        self.run_git('commit', '-qm', 'README without SOT fixture')
        self.assertEqual(self.guard('--base', 'origin/main', '--head', 'HEAD').returncode, 1)

    def test_docs_ci_only(self):
        self.write('docs/guide.md', 'guide\n', True)
        self.write('.github/example.yml', 'name: example\n', True)
        self.assertEqual(self.guard('--staged').returncode, 0)

    def test_unstaged_sot_does_not_count(self):
        self.write('README.md', 'updated\n', True)
        self.write('docs/pr/PR-TBD-example.md', 'unstaged\n')
        self.assertEqual(self.guard('--staged').returncode, 1)

    def test_staged_sot_counts_even_if_worktree_differs(self):
        self.write('README.md', 'updated\n', True)
        self.write('docs/pr/PR-153-example.md', 'staged\n', True)
        self.write('docs/pr/PR-153-example.md', 'unstaged edits\n')
        self.assertEqual(self.guard('--staged').returncode, 0)
        self.run_git('commit', '-qm', 'with SOT')
        self.assertEqual(self.guard('--base', 'origin/main', '--head', 'HEAD').returncode, 0)

    def test_deleted_sot_does_not_count(self):
        self.write('README.md', 'updated\n', True)
        # Stage deletion without deleting the working file.
        self.run_git('update-index', '--force-remove', 'docs/pr/PR-1-old.md')
        self.assertEqual(self.guard('--staged').returncode, 1)

    def test_prior_branch_changes_are_checked(self):
        self.write('README.md', 'updated\n', True)
        self.run_git('commit', '-qm', 'earlier incomplete commit fixture')
        self.write('docs/guide.md', 'next commit\n', True)
        self.assertEqual(self.guard('--staged').returncode, 1)
        self.write('docs/pr/PR-TBD-example.md', 'SOT\n', True)
        self.assertEqual(self.guard('--staged').returncode, 0)

    def test_missing_base_fails_closed(self):
        self.assertEqual(self.guard('--base', 'missing-ref', '--staged').returncode, 2)

    def test_no_changes_and_filename_with_newline(self):
        self.assertEqual(self.guard('--staged').returncode, 0)
        self.write('docs/guide\nREADME.md', 'docs\n', True)
        self.assertEqual(self.guard('--staged').returncode, 0)
        self.write('source\nfile.py', 'code\n', True)
        self.assertEqual(self.guard('--staged').returncode, 1)

    def install(self):
        return subprocess.run(['bash', str(ROOT / 'scripts/install_git_hooks.sh')], cwd=self.repo,
                              text=True, capture_output=True, env=self.env)

    def test_installer_preserves_local_custom_path_and_hook_execution(self):
        hooks = self.repo / 'empty-hooks'
        hook = hooks / 'pre-commit'
        hook.write_text('#!/bin/sh\necho existing-check >&2\nexit 37\n')
        hook.chmod(0o755)
        result = self.install()
        self.assertEqual(result.returncode, 1)
        self.assertEqual(self.run_git('config', '--get', 'core.hooksPath').stdout.strip(), str(hooks))
        self.write('README.md', 'changed\n', True)
        commit = self.run_git('commit', '-qm', 'existing hook still runs', check=False)
        self.assertNotEqual(commit.returncode, 0)
        self.assertIn('existing-check', commit.stderr)

    def test_installer_preserves_global_custom_path(self):
        self.run_git('config', '--unset', 'core.hooksPath')
        config = self.repo / 'global-config'
        config.write_text('[core]\n hooksPath = /example/custom-hooks\n')
        self.env['GIT_CONFIG_GLOBAL'] = str(config)
        before = config.read_bytes()
        self.assertEqual(self.install().returncode, 1)
        self.assertEqual(config.read_bytes(), before)
        self.assertEqual(self.run_git('config', '--local', '--get', 'core.hooksPath', check=False).returncode, 1)

    def test_installer_preserves_worktree_override(self):
        self.run_git('config', 'extensions.worktreeConfig', 'true')
        self.run_git('config', '--worktree', 'core.hooksPath', '/example/worktree-hooks')
        before = self.run_git('config', '--local', '--get', 'core.hooksPath').stdout
        self.assertEqual(self.install().returncode, 1)
        self.assertEqual(self.run_git('config', '--worktree', '--get', 'core.hooksPath').stdout.strip(), '/example/worktree-hooks')
        self.assertEqual(self.run_git('config', '--local', '--get', 'core.hooksPath').stdout, before)

    def test_installer_preserves_default_executable_hook(self):
        self.run_git('config', '--unset', 'core.hooksPath')
        hook = self.repo / '.git/hooks/pre-push'
        hook.write_text('#!/bin/sh\nexit 0\n')
        hook.chmod(0o755)
        before = hook.read_bytes()
        self.assertEqual(self.install().returncode, 1)
        self.assertEqual(hook.read_bytes(), before)
        self.assertEqual(self.run_git('config', '--get', 'core.hooksPath', check=False).returncode, 1)

    def test_installed_hook_blocks_actual_commit_and_keeps_formatter(self):
        for name in ['scripts/check_pr_sot.py', 'scripts/install_git_hooks.sh', '.githooks/pre-commit', 'ops/cleanFormatter']:
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, path)
        self.run_git('add', 'scripts', '.githooks', 'ops')
        self.run_git('commit', '-qm', 'fixture hook baseline')
        self.run_git('update-ref', 'refs/remotes/origin/main', 'HEAD')
        self.run_git('config', '--unset', 'core.hooksPath')
        subprocess.run(['bash', 'scripts/install_git_hooks.sh'], cwd=self.repo, check=True, capture_output=True, env=self.env)
        subprocess.run(['bash', 'scripts/install_git_hooks.sh'], cwd=self.repo, check=True, capture_output=True, env=self.env)
        self.write('README.md', 'updated\n', True)
        before = self.run_git('rev-parse', 'HEAD').stdout
        result = self.run_git('commit', '-qm', 'must fail', check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('missing changed', result.stderr)
        self.assertEqual(self.run_git('rev-parse', 'HEAD').stdout, before)
        self.write('docs/pr/PR-TBD-example.md', 'SOT\n', True)
        self.run_git('commit', '-qm', 'must pass')
        self.write('README.md', 'file:' + '//example\n', True)
        result = self.run_git('commit', '-qm', 'formatter must fail', check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('forbidden raw file URL', result.stderr)


if __name__ == '__main__':
    unittest.main()
