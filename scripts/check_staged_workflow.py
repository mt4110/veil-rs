#!/usr/bin/env python3
"""Exercise first scan, Git index checks, exclusions and repeat use with fake data."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def verify(binary: Path, language: str | None = None) -> dict:
    binary = binary.resolve(strict=True)
    # Retain isolated fixtures for diagnostics; no owner repository is modified.
    root = Path(tempfile.mkdtemp(prefix='veil-staged-workflow-'))
    repo = root / 'project'
    repo.mkdir()
    home = root / 'home'
    home.mkdir()
    hooks = root / 'empty-hooks'
    hooks.mkdir()
    env = {k: v for k, v in os.environ.items() if not k.startswith(('VEIL_', 'GIT_'))}
    env.update(HOME=str(home), XDG_CONFIG_HOME=str(home / '.config'),
               GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM='1', NO_COLOR='1')

    def git(*args: str) -> None:
        subprocess.run(['git', *args], cwd=repo, env=env, check=True, capture_output=True, timeout=30)

    git('init', '-q')
    git('config', 'user.name', 'Workflow fixture')
    git('config', 'user.email', 'fixture@example.invalid')
    git('config', 'commit.gpgsign', 'false')
    git('config', 'core.hooksPath', str(hooks))
    source = repo / 'main.py'
    clean = 'print("hello")\n'
    secret = 'AKIA' + 'A' * 16
    fake_source = f'aws_key = "{secret}"\n'
    source.write_text(clean, encoding='utf-8')
    (repo / '.gitignore').write_text('ignored.txt\n', encoding='utf-8')
    results: list[dict] = []

    def require(condition: bool, message: str) -> None:
        if not condition:
            raise AssertionError(f'{message}; fixture: {root}')

    def snapshot() -> dict[str, str]:
        files = [p for p in repo.rglob('*') if p.is_file() and '.git' not in p.relative_to(repo).parts]
        files += [repo / '.git' / 'index', repo / '.git' / 'HEAD']
        return {str(p.relative_to(repo)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in files if p.exists()}

    def scan(name: str, expected: int, args: list[str], *, cwd: Path = repo,
             scanned: int | None = None, skipped: int | None = None, finding_path: str | None = None) -> None:
        before = snapshot()
        result = subprocess.run([str(binary), *(['--lang', language] if language else []), 'scan', *args, '--mask-mode', 'redact',
                                 '--no-color', '--fail-on-findings', '1', '--format', 'json'],
                                cwd=cwd, env=env, capture_output=True, text=True, timeout=60)
        if result.returncode != expected:
            raise AssertionError(f'{name}: expected exit {expected}, got {result.returncode}; fixture: {root}')
        if secret in result.stdout or secret in result.stderr:
            raise AssertionError(f'{name}: fake secret was not redacted')
        if snapshot() != before:
            raise AssertionError(f'{name}: source, index or HEAD changed')
        summary = {}
        if expected != 2:
            report = json.loads(result.stdout)
            require(report['schemaVersion'] == 'veil-v1', f'{name}: wrong schema')
            summary = report['summary']
            if scanned is not None:
                require(summary['scanned_files'] == scanned, f'{name}: expected scanned={scanned}, got {summary["scanned_files"]}')
            if skipped is not None:
                require(summary['skipped_files'] == skipped, f'{name}: expected skipped={skipped}, got {summary["skipped_files"]}')
            if finding_path:
                require(any(f['path'] == finding_path for f in report['findings']), f'{name}: expected finding path missing')
        results.append({'case': name, 'exit': result.returncode,
                        'scanned_files': summary.get('scanned_files'),
                        'skipped_files': summary.get('skipped_files'), 'preserved': True})

    scan('first-file-clean', 0, ['main.py'], scanned=1)
    scan('staged-empty-before-first-commit', 0, ['--staged'], scanned=0)
    git('add', 'main.py')
    scan('staged-clean-before-first-commit', 0, ['--staged'], scanned=1)
    source.write_text(fake_source, encoding='utf-8')
    git('add', 'main.py')
    source.write_text(clean, encoding='utf-8')
    scan('working-copy-fixed', 0, ['main.py'], scanned=1)
    scan('index-still-has-secret', 1, ['--staged'], scanned=1, finding_path='main.py')
    git('add', 'main.py')
    scan('restaged-fix', 0, ['--staged'], scanned=1)
    (repo / 'unchanged.txt').write_text(fake_source, encoding='utf-8')
    git('add', '.gitignore', 'unchanged.txt')
    git('commit', '-qm', 'clean initial fixture')
    scan('empty-stage-after-commit', 0, ['--staged'], scanned=0)
    (repo / 'untracked.txt').write_text(fake_source, encoding='utf-8')
    (repo / 'ignored.txt').write_text(fake_source, encoding='utf-8')
    source.write_text(fake_source, encoding='utf-8')
    scan('unstaged-and-untracked-not-inspected', 0, ['--staged'], scanned=0)
    scan('unstaged-change-visible-to-file-scan', 1, ['main.py'], finding_path='main.py')
    source.write_text(clean, encoding='utf-8')
    # Scan the ignored fixture's directory with no untracked decoy present in it.
    ignored_dir = repo / 'ignored-input'
    ignored_dir.mkdir()
    (ignored_dir / '.gitignore').write_text('secret.txt\n', encoding='utf-8')
    (ignored_dir / 'secret.txt').write_text(fake_source, encoding='utf-8')
    (ignored_dir / 'safe.txt').write_text('safe\n', encoding='utf-8')
    scan('filesystem-ignored-secret-excluded', 0, ['ignored-input'], scanned=1)
    source.write_text(fake_source, encoding='utf-8')
    git('add', 'main.py')
    scan('next-change-staged', 1, ['--staged'], scanned=1, finding_path='main.py')
    source.write_text(clean + '# next edit\n', encoding='utf-8')
    git('add', 'main.py')
    scan('next-change-fixed-and-restaged', 0, ['--staged'], scanned=1)
    git('commit', '-qm', 'next clean fixture')
    (repo / 'asset.bin').write_bytes(b'\x00\x01' + secret.encode())
    git('add', 'asset.bin')
    scan('staged-binary-skipped', 0, ['--staged'], scanned=0, skipped=1)
    git('add', '-f', 'ignored.txt')
    scan('force-added-ignored-secret-excluded', 0, ['--staged'], scanned=0, skipped=1)
    scan('explicit-ignored-file', 1, ['ignored.txt'], finding_path='ignored.txt')
    (repo / 'invalid-utf8.txt').write_bytes(b'hello\xff\n')
    scan('incomplete-utf8-error', 2, ['invalid-utf8.txt'])
    scan('missing-input-error', 2, ['missing.py'])
    scan('outside-git-error', 2, ['--staged'], cwd=home)
    return {'status': 'pass', 'cases': results, 'fixture': str(root)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--lang', choices=('ja', 'en'))
    args = parser.parse_args()
    try:
        result = verify(args.binary, args.lang)
    except (AssertionError, OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired, ValueError) as error:
        print(f'Staged workflow check failed: {error}')
        return 1
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
