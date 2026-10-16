#!/usr/bin/env python3
"""Run native installer acceptance against an extracted candidate (publishes nothing)."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import stat
import tempfile
import unittest
import zipfile

from test_install_cli import InstallerTests
from check_cli_release import ROOT, bundle_files, check_bundle, regular_document


def extract_packet(archive_path: Path, packet: Path) -> None:
    allowed = bundle_files('veil', ROOT)
    with zipfile.ZipFile(archive_path) as archive:
        entries = archive.infolist()
        if len(entries) != len(allowed) or {e.filename for e in entries} != allowed:
            raise ValueError('installer archive file set differs from the allowlist')
        for entry in entries:
            mode = entry.external_attr >> 16
            limit = 128 * 1024 * 1024 if entry.filename == 'veil' else 256 * 1024 if entry.filename == 'SHA256SUMS' else regular_document(ROOT, entry.filename).stat().st_size
            if stat.S_IFMT(mode) not in (0, stat.S_IFREG) or entry.flag_bits & 1 or not 0 < entry.file_size <= limit:
                raise ValueError('invalid installer archive entry')
            destination = packet / entry.filename
            destination.parent.mkdir(parents=True, exist_ok=True)
            with archive.open(entry) as source, destination.open('xb') as output:
                shutil.copyfileobj(source, output)
            if entry.filename == 'veil':
                if not mode & stat.S_IXUSR:
                    raise ValueError('archive CLI is not executable')
                destination.chmod(0o755)
    check_bundle(packet, 'veil')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='veil-install-archive-') as scratch:
        packet = Path(scratch).resolve()
        extract_packet(args.archive, packet)
        os.environ['VEIL_INSTALL_TEST_BUNDLE'] = str(packet)
        result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(InstallerTests))
    print(json.dumps({'status': 'passed' if result.wasSuccessful() else 'failed',
                      'cases': result.testsRun, 'skipped': len(result.skipped),
                      'platform': platform.system(), 'architecture': platform.machine(),
                      'packet': 'extracted native candidate',
                      'interactive_shells': ['bash', 'zsh'] if platform.system() == 'Darwin' else ['bash'],
                      'fixture_versions': 'native C executables for old-version transition only',
                      'installer_path_excludes': ['cargo', 'rustc', 'cc', 'curl', 'sudo']}))
    return 0 if result.wasSuccessful() else 1


if __name__ == '__main__':
    raise SystemExit(main())
