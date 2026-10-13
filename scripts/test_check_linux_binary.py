import unittest
from check_linux_binary import parse_elf, check_musl_link_map, execution_context


class LinuxLinkageTests(unittest.TestCase):
    def test_static_elf_has_no_glibc_floor(self):
        result = parse_elf("Machine: AArch64", "Program Headers:\nLOAD 0x0", "There is no dynamic section", "No version information", require_static=True)
        self.assertTrue(result["static_verified"])
        self.assertIsNone(result["glibc_symbol_floor"])

    def test_interpreter_or_needed_library_or_glibc_prevents_static_claim(self):
        for programs, dynamic, versions in [
            ("Program Headers:\n INTERP 0x0", "", ""),
            ("Program Headers:\n LOAD 0x0", "(NEEDED) Shared library: [libz.so.1]", ""),
            ("Program Headers:\n LOAD 0x0", "", "Name: GLIBC_2.39"),
            ("Program Headers:\n LOAD 0x0", "", "Name: GLIBC_ABI_DT_RELR"),
        ]:
            with self.subTest(programs=programs, dynamic=dynamic, versions=versions), self.assertRaisesRegex(ValueError, "static Linux"):
                parse_elf("Machine: AArch64", programs, dynamic, versions, require_static=True)

    def test_glibc_floor_uses_numeric_order_and_reports_runtime_dependencies(self):
        result = parse_elf("Machine: Advanced Micro Devices X86-64", "Program Headers:\n INTERP 0x0",
                           "(NEEDED) Shared library: [libc.so.6]", "GLIBC_2.9 GLIBC_2.39 GLIBC_2.17")
        self.assertEqual(result["glibc_symbol_floor"], "2.39")
        self.assertEqual(result["needed_libraries"], ["libc.so.6"])
        self.assertFalse(result["static_verified"])

    def test_missing_elf_information_is_not_static_evidence(self):
        with self.assertRaises(ValueError):
            parse_elf("", "", "", "", require_static=True)

    def test_link_map_retains_evidence_of_reviewed_static_runtimes(self):
        text = "/rust/lib/rustlib/aarch64-unknown-linux-musl/lib/self-contained/libc.a(start.o)\n/rust/lib/rustlib/aarch64-unknown-linux-musl/lib/self-contained/libunwind.a(unwind.o)"
        self.assertFalse(check_musl_link_map(text)["gnu_libgcc_archive_contribution"])

    def test_link_map_rejects_unreviewed_gcc_and_missing_runtime(self):
        for text in ["libgcc.a(div.o)", "libgcc_eh.a(unwind.o)", "", "/usr/lib/libc.a(start.o)\n/usr/lib/libunwind.a(unwind.o)"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                check_musl_link_map(text)

    def test_native_mode_requires_consistent_host_and_elf(self):
        result = execution_context("native", "x86_64", "Advanced Micro Devices X86-64", "X64")
        self.assertEqual(result["mode"], "native")
        for host, runtime, elf, emulator in [(None, "x86_64", None, None), ("arm64", "aarch64", "Advanced Micro Devices X86-64", None),
                                             ("arm64", "x86_64", "Advanced Micro Devices X86-64", None),
                                             ("x64", "x86_64", "Advanced Micro Devices X86-64", "QEMU")]:
            with self.subTest(host=host, runtime=runtime), self.assertRaises(ValueError):
                execution_context("native", runtime, elf, host, emulator)

    def test_emulation_and_unknown_are_not_implicitly_native(self):
        result = execution_context("emulated", "x86_64", "Advanced Micro Devices X86-64", "aarch64", "QEMU user mode")
        self.assertEqual(result["host_architecture"], "aarch64")
        self.assertEqual(result["binary_architecture"], "x86_64")
        self.assertEqual(result["mode"], "emulated")
        self.assertEqual(execution_context("unknown", "x86_64", None)["mode"], "unknown")
        with self.assertRaises(ValueError):
            execution_context("emulated", "x86_64", None, "aarch64")
