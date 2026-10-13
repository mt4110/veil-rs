from pathlib import Path
import struct
import tempfile
import unittest
from check_windows_binary import inspect_windows_binary, check_windows_gnu_link_map
from verify_cli_archive import archive_execution_context


def pe(dll=b'KERNEL32.dll', delay=None):
    data = bytearray(1024)
    data[:2] = b'MZ'
    struct.pack_into('<I', data, 0x3c, 0x80)
    data[0x80:0x84] = b'PE\0\0'
    struct.pack_into('<HH', data, 0x84, 0x8664, 1)
    struct.pack_into('<HH', data, 0x94, 240, 2)
    opt = 0x98
    struct.pack_into('<H', data, opt, 0x20b)
    struct.pack_into('<Q', data, opt + 24, 0x140000000)
    struct.pack_into('<H', data, opt + 68, 3)
    struct.pack_into('<I', data, opt + 108, 16)
    struct.pack_into('<II', data, opt + 120, 0x1000, 40)
    struct.pack_into('<IIII', data, opt + 240 + 8, 512, 0x1000, 512, 512)
    struct.pack_into('<IIIII', data, 512, 0, 0, 0, 0x1080, 0)
    data[640:640 + len(dll) + 1] = dll + b'\0'
    if delay:
        struct.pack_into('<II', data, opt + 112 + 13 * 8, 0x1100, 64)
        struct.pack_into('<II', data, 768, 1, 0x1180)
        data[896:896 + len(delay) + 1] = delay + b'\0'
    return data


class WindowsBinaryTests(unittest.TestCase):
    def inspect(self, data):
        with tempfile.TemporaryDirectory() as scratch:
            path = Path(scratch) / 'veil.exe'
            path.write_bytes(data)
            return inspect_windows_binary(path)

    def test_system_imports_and_delay_imports_recorded(self):
        report = self.inspect(pe(delay=b'USER32.dll'))
        self.assertEqual(report['machine'], 'x86_64')
        self.assertEqual(report['imports'], ['kernel32.dll'])
        self.assertEqual(report['delay_imports'], ['user32.dll'])
        self.assertEqual(report['additional_dlls'], [])

    def test_non_system_runtime_is_not_silently_omitted(self):
        for name in [b'libgcc_s_seh-1.dll', b'libwinpthread-1.dll', b'VCRUNTIME140.dll']:
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'non-system DLL'):
                self.inspect(pe(name))

    def test_delayed_non_system_runtime_rejected(self):
        with self.assertRaisesRegex(ValueError, 'non-system DLL'):
            self.inspect(pe(delay=b'private.dll'))

    def test_redirected_dll_paths_rejected(self):
        for name in [b'../kernel32.dll', b'C:\\kernel32.dll', b'\xff.dll']:
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.inspect(pe(name))

    def test_truncated_headers_sections_and_imports_rejected(self):
        for length in [0, 63, 128, 200, 600, 700]:
            with self.subTest(length=length), self.assertRaises(ValueError):
                self.inspect(pe()[:length])

    def test_invalid_import_rva_and_directory_size_rejected(self):
        for rva, size in [(0x2000, 40), (0x1000, 20), (0, 40), (0x1000, 0xffffffff)]:
            data = pe()
            struct.pack_into('<II', data, 0x98 + 120, rva, size)
            with self.subTest(rva=rva, size=size), self.assertRaises(ValueError):
                self.inspect(data)

    def test_invalid_delay_attributes_rejected(self):
        data = pe(delay=b'user32.dll')
        struct.pack_into('<I', data, 768, 2)
        with self.assertRaisesRegex(ValueError, 'delay-import'):
            self.inspect(data)

    def test_non_console_or_dll_is_rejected(self):
        for offset, value in [(0x98 + 68, 2), (0x96, 0x2002)]:
            data = pe()
            struct.pack_into('<H', data, offset, value)
            with self.assertRaises(ValueError):
                self.inspect(data)

    def test_link_map_requires_reviewed_archives_and_gcc_generation(self):
        text = '\n'.join('/usr/lib/gcc/x86_64-w64-mingw32/14-posix/' + n + '(member.o)' for n in ['libmingw32.a', 'libmingwex.a', 'libgcc.a', 'libgcc_eh.a'])
        self.assertEqual(len(check_windows_gnu_link_map(text)['contributing_archives']), 4)
        for changed in [text.replace('libmingwex.a', 'other.a'), text.replace('/14-posix/', '/15-posix/')]:
            with self.assertRaises(ValueError):
                check_windows_gnu_link_map(changed)

    def test_wine_qemu_is_recorded_separately_from_windows_native(self):
        report = {'platform': 'Linux', 'architecture': 'x86_64', 'windows_pe': {'machine': 'x86_64'}}
        result = archive_execution_context(report, 'emulated', 'aarch64', 'QEMU user-mode', '/usr/lib/wine/wine64')
        self.assertEqual(result['compatibility_layer'], 'Wine')
        self.assertFalse(result['windows_native'])
        for mode, host, emulator, runner in [('native', 'x86_64', None, '/wine'), ('emulated', None, None, '/wine'), ('native', 'x86_64', None, None)]:
            with self.assertRaises(ValueError):
                archive_execution_context(report, mode, host, emulator, runner)

    def test_native_windows_requires_matching_binary_isa(self):
        report = {'platform': 'Windows', 'architecture': 'AMD64', 'windows_pe': {'machine': 'x86_64'}}
        self.assertTrue(archive_execution_context(report, 'native', 'X64', None, None)['windows_native'])
        with self.assertRaises(ValueError):
            archive_execution_context(report, 'native', 'ARM64', None, None)
