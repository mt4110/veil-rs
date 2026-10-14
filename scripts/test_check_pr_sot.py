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
        return subprocess.run(['git', *args], cwd=self.repo, text=True, capture_output=True, check=check)

    def write(self, name, content, stage=False):
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        if stage:
            self.run_git('add', name)

    def guard(self, *args):
        return subprocess.run(['python3', str(CHECKER), *args], cwd=self.repo, text=True, capture_output=True)

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

    def test_installed_hook_blocks_actual_commit_and_keeps_formatter(self):
        for name in ['scripts/check_pr_sot.py', 'scripts/install_git_hooks.sh', '.githooks/pre-commit', 'ops/cleanFormatter']:
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, path)
        self.run_git('add', 'scripts', '.githooks', 'ops')
        self.run_git('commit', '-qm', 'fixture hook baseline')
        self.run_git('update-ref', 'refs/remotes/origin/main', 'HEAD')
        subprocess.run(['bash', 'scripts/install_git_hooks.sh'], cwd=self.repo, check=True, capture_output=True)
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
