#!/usr/bin/env python3
"""Check JA/EN installation-to-repeat-use from one native archive, without Rust."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

from check_cli_install import extract_packet
from check_staged_workflow import verify


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    args = parser.parse_args()
    results = []
    with tempfile.TemporaryDirectory(prefix='veil-onboarding-') as scratch:
        root = Path(scratch).resolve()
        packet = root / 'packet'
        packet.mkdir()
        extract_packet(args.archive, packet)
        tools = root / 'tools'
        tools.mkdir()
        for name in ('sh', 'bash', 'zsh', 'awk', 'sha256sum', 'shasum', 'od', 'tr',
                     'cut', 'uname', 'dirname', 'readlink', 'cmp', 'mkdir', 'mktemp',
                     'cp', 'mv', 'ln', 'sed', 'git'):
            executable = shutil.which(name)
            if executable:
                (tools / name).symlink_to(executable)
        shells = ['bash', 'zsh'] if platform.system() == 'Darwin' else ['bash']
        for shell in shells:
            if not (tools / shell).exists():
                raise AssertionError(f'required shell unavailable: {shell}')
        for name in ('cargo', 'rustc', 'curl', 'sudo'):
            assert shutil.which(name, path=str(tools)) is None
        for language in ('ja', 'en'):
            home = root / language
            home.mkdir()
            prefix = home / 'veil tools'
            env = {k: v for k, v in os.environ.items()
                   if not k.startswith(('VEIL_', 'GIT_', 'LC_'))
                   and k not in ('ENV', 'BASH_ENV', 'CDPATH')}
            env.update(HOME=str(home), XDG_CONFIG_HOME=str(home / '.config'),
                       PATH=str(tools), LANG='C', GIT_CONFIG_NOSYSTEM='1',
                       GIT_CONFIG_GLOBAL=os.devnull, NO_COLOR='1')
            installed = subprocess.run(
                ['/bin/sh', str(packet / 'install.sh'), '--prefix', str(prefix),
                 '--lang', language], env=env, capture_output=True, text=True,
                timeout=120)
            if installed.returncode:
                raise RuntimeError('Installer failed: ' + installed.stdout + installed.stderr)
            version = subprocess.check_output([str(packet / 'veil'), '--lang', 'en', '--version'], env=env, text=True).strip()
            path_line = next(line for line in installed.stdout.splitlines()
                             if line.startswith('export PATH='))
            for shell in shells:
                command = (path_line + '\ncommand -v veil\nveil --version\n'
                           f'veil --lang {language} scan --help')
                flags = ['--noprofile', '--norc', '-c'] if shell == 'bash' else ['-d', '-f', '-c']
                child = subprocess.run([str(tools / shell), *flags, command],
                                       env=env, cwd=home, capture_output=True,
                                       text=True, check=True, timeout=30)
                assert child.stdout.splitlines()[0] == str(prefix / 'bin/veil')
                assert child.stdout.splitlines()[1] == version
                guide_name = 'getting-started.md' if language == 'ja' else 'getting-started.en.md'
                assert guide_name in child.stdout
                assert ('使い方:' if language == 'ja' else 'Usage:') in child.stdout
            # The installed executable runs all first/staged/restaged/next-use
            # cases with an isolated Git repository and the restricted PATH.
            old_env = os.environ.copy()
            try:
                os.environ.clear()
                os.environ.update(env)
                workflow = verify(prefix / 'bin/veil', language)
            finally:
                os.environ.clear()
                os.environ.update(old_env)
            guide = packet / 'docs/cli' / ('getting-started.md' if language == 'ja'
                                           else 'getting-started.en.md')
            assert guide.is_file()
            results.append({'language': language, 'shells': shells,
                            'workflow': workflow, 'guide_in_archive': True})
    print(json.dumps({'status': 'pass', 'platform': platform.system(),
                      'architecture': platform.machine(), 'languages': results,
                      'rust_on_path': False, 'synthetic_only': True}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
