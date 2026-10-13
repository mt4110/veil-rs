# Third-party notices

Veil CLI is distributed under the project MIT / Apache-2.0 terms. Third-party components retain their own licenses and copyrights. Read [NOTICE](NOTICE), the [MIT license](LICENSE-MIT), and the [Apache license](LICENSE-APACHE).

This reviewed inventory covers the default `veil-cli` normal and build dependency graph for all Cargo targets. It intentionally includes target-specific and build-only crates; it is not a claim that every listed component is linked into every binary. Test-only dependencies and other workspace applications are excluded. Existing v1 verify is part of the default CLI. The [manifest](third_party/manifest.json) records exact versions, Cargo.lock checksums, document digests and public origins. Upstream texts are preserved byte-for-byte as `.txt`; relative references inside those original texts are not project navigation links.

## Source and native components

`colored` 2.2.0 (MPL-2.0) and `libgit2-sys` 0.18.8+1.9.7 ship their unmodified published source archives alongside the binary. The archives contain Cargo build instructions and, for libgit2, vendored libgit2 1.9.7 and its sources. Extract each `.crate` as a gzip-compressed tar archive. Their SHA-256 values match Cargo.lock.

`libgit2-sys` wrapper MIT/Apache terms do not replace native libgit2 terms: retain its COPYING (GPL-2.0 with the documented linking exception) and AUTHORS. The default CLI disables libgit2 HTTPS/SSH support. Nested bundled license alternatives (including zlib, llhttp, PCRE2 and WinHTTP) are retained conservatively, without asserting all are enabled. Zstd native code is available under its upstream BSD/GPL alternatives; its BSD text is included. Preserve complete upstream notices when making a distribution.

Where a published crate omitted license text, documents come from its recorded upstream source commit. `fxhash` and the two winapi import-library crates use their published copyright/license headers plus the Apache-2.0 option; copyright holders have not been invented. SPDX alternatives in the table remain upstream declarations, not a statement that all alternatives apply simultaneously.

The objc2-family upstream file declares licensing but links to the general MIT text. Its exact published Cargo.toml.orig (AUTHORS.txt) and the unmodified SPDX MIT template are supplied alongside that declaration; the template placeholders are not an invented copyright notice.

## Static Linux runtime

The Rust 1.92.0 musl candidates link the toolchain self-contained musl 1.2.3 (including upstream Rust security patches) and LLVM libunwind. Their original licenses are included below, in addition to Rust standard-library notices. Earlier candidates (before this narrowed source) had ARM64 and x86_64 link maps confirming these archives and no GNU libgcc archive contribution; other compiler/target runtime changes require a fresh audit. The x86_64 candidate was cross-built and executed under QEMU; native x86_64 execution remains untested. These notices are included conservatively in every bundle.

- musl-1.2.3: [COPYRIGHT.txt](third_party/licenses/musl-1.2.3/COPYRIGHT.txt)
- llvm-libunwind-rust-1.92.0: [LICENSE-LLVM.txt](third_party/licenses/llvm-libunwind-rust-1.92.0/LICENSE-LLVM.txt), [LICENSE-LIBUNWIND.txt](third_party/licenses/llvm-libunwind-rust-1.92.0/LICENSE-LIBUNWIND.txt)

## Windows GNU runtime

The earlier Rust 1.92.0 x86_64-pc-windows-gnu candidate, before this narrowed source, statically included MinGW-w64 12.0.0 CRT code and GCC 14.2.0 runtime/unwinder code, confirmed by its link map. The GNU candidate was cross-built and executed through Wine plus QEMU; native Windows and the separate MSVC candidate remain untested. Windows system DLLs, Wine and QEMU binaries are not redistributed.

The MinGW package has per-file licensing exceptions; its complete Debian copyright and upstream COPYING/AUTHORS are retained. The installed GCC package copyright contains historical version names; it is preserved unchanged and supplemented with the GCC 14.2.0 upstream GPLv3 and Runtime Library Exception 3.1. Package archive identities/checksums are recorded in the manifest. These documents are included conservatively in every bundle; they do not assert that every listed component applies to every target. Refresh the runtime audit when changing the compiler or C toolchain.

- MinGW-w64 12.0.0: [COPYING](third_party/licenses/mingw-w64-12.0.0/COPYING.txt), [AUTHORS](third_party/licenses/mingw-w64-12.0.0/AUTHORS.txt), [Debian package copyright](third_party/licenses/mingw-w64-12.0.0/COPYRIGHT-Debian.txt).
- GCC runtime 14.2.0: [GPLv3](third_party/licenses/gcc-runtime-14.2.0/COPYING3.txt), [Runtime Library Exception](third_party/licenses/gcc-runtime-14.2.0/COPYING.RUNTIME.txt), [Debian package copyright](third_party/licenses/gcc-runtime-14.2.0/COPYRIGHT-Debian.txt).

## Rust standard library

Official Rust distribution standard-library notices and licenses are included for Rust 1.92.0 and 1.98.1. The distribution archive checksums and members are recorded in the manifest. These are the complete upstream attribution documents, including target-specific entries; not every entry is active on every platform. Packaging checks the selected compiler version and rejects versions without a reviewed packet. Build the binary with that same compiler; a version check alone cannot identify an arbitrary prebuilt binary. Re-audit target/runtime additions (including separately installed native libraries) before distributing them.

- Rust 1.98.1: [standard-library notices](third_party/licenses/rust-1.98.1/COPYRIGHT-library.html.txt), [MIT](third_party/licenses/rust-1.98.1/LICENSE-MIT.txt), [Apache-2.0](third_party/licenses/rust-1.98.1/LICENSE-APACHE.txt).

- Rust 1.92.0: [standard-library notices](third_party/licenses/rust-1.92.0/COPYRIGHT-library.html.txt), [MIT](third_party/licenses/rust-1.92.0/LICENSE-MIT.txt), [Apache-2.0](third_party/licenses/rust-1.92.0/LICENSE-APACHE.txt).


## Cargo packages

| Package | Upstream license expression | License / attribution documents | Source |

| --- | --- | --- | --- |

| adler2 2.0.1 | 0BSD OR MIT OR Apache-2.0 | [LICENSE-0BSD.txt](third_party/licenses/adler2-2.0.1/LICENSE-0BSD.txt), [LICENSE-APACHE.txt](third_party/licenses/adler2-2.0.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/adler2-2.0.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/adler2/2.0.1) |

| aes 0.8.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/aes-0.8.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/aes-0.8.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/aes/0.8.4) |

| aho-corasick 1.1.4 | Unlicense OR MIT | [COPYING.txt](third_party/licenses/aho-corasick-1.1.4/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/aho-corasick-1.1.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/aho-corasick/1.1.4) |

| android_system_properties 0.1.5 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/android_system_properties-0.1.5/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/android_system_properties-0.1.5/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/android_system_properties/0.1.5) |

| anstream 0.6.21 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/anstream-0.6.21/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/anstream-0.6.21/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/anstream/0.6.21) |

| anstyle-parse 0.2.7 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/anstyle-parse-0.2.7/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/anstyle-parse-0.2.7/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/anstyle-parse/0.2.7) |

| anstyle-query 1.1.5 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/anstyle-query-1.1.5/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/anstyle-query-1.1.5/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/anstyle-query/1.1.5) |

| anstyle-wincon 3.0.11 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/anstyle-wincon-3.0.11/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/anstyle-wincon-3.0.11/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/anstyle-wincon/3.0.11) |

| anstyle 1.0.13 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/anstyle-1.0.13/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/anstyle-1.0.13/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/anstyle/1.0.13) |

| anyhow 1.0.101 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/anyhow-1.0.101/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/anyhow-1.0.101/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/anyhow/1.0.101) |

| arbitrary 1.4.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/arbitrary-1.4.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/arbitrary-1.4.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/arbitrary/1.4.2) |

| arrayref 0.3.9 | BSD-2-Clause | [LICENSE.txt](third_party/licenses/arrayref-0.3.9/LICENSE.txt) | [published crate](https://crates.io/crates/arrayref/0.3.9) |

| arrayvec 0.7.6 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/arrayvec-0.7.6/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/arrayvec-0.7.6/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/arrayvec/0.7.6) |

| autocfg 1.5.0 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/autocfg-1.5.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/autocfg-1.5.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/autocfg/1.5.0) |

| bitflags 1.3.2 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/bitflags-1.3.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/bitflags-1.3.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/bitflags/1.3.2) |

| bitflags 2.10.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/bitflags-2.10.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/bitflags-2.10.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/bitflags/2.10.0) |

| blake3 1.8.3 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | [LICENSE_A2.txt](third_party/licenses/blake3-1.8.3/LICENSE_A2.txt), [LICENSE_A2LLVM.txt](third_party/licenses/blake3-1.8.3/LICENSE_A2LLVM.txt), [LICENSE_CC0.txt](third_party/licenses/blake3-1.8.3/LICENSE_CC0.txt) | [published crate](https://crates.io/crates/blake3/1.8.3) |

| block-buffer 0.10.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/block-buffer-0.10.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/block-buffer-0.10.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/block-buffer/0.10.4) |

| block2 0.6.2 | MIT | [LICENSE.md.txt](third_party/licenses/block2-0.6.2/LICENSE.md.txt), [LICENSE-MIT.txt](third_party/licenses/block2-0.6.2/LICENSE-MIT.txt), [AUTHORS.txt](third_party/licenses/block2-0.6.2/AUTHORS.txt) | [published crate](https://crates.io/crates/block2/0.6.2) |

| bstr 1.12.1 | MIT OR Apache-2.0 | [COPYING.txt](third_party/licenses/bstr-1.12.1/COPYING.txt), [LICENSE-APACHE.txt](third_party/licenses/bstr-1.12.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/bstr-1.12.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/bstr/1.12.1) |

| bumpalo 3.19.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/bumpalo-3.19.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/bumpalo-3.19.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/bumpalo/3.19.1) |

| byteorder 1.5.0 | Unlicense OR MIT | [COPYING.txt](third_party/licenses/byteorder-1.5.0/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/byteorder-1.5.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/byteorder/1.5.0) |

| bzip2 0.6.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/bzip2-0.6.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/bzip2-0.6.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/bzip2/0.6.1) |

| cc 1.2.55 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/cc-1.2.55/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/cc-1.2.55/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/cc/1.2.55) |

| cfg-if 1.0.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/cfg-if-1.0.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/cfg-if-1.0.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/cfg-if/1.0.4) |

| cfg_aliases 0.2.1 | MIT | [LICENSE.txt](third_party/licenses/cfg_aliases-0.2.1/LICENSE.txt) | [published crate](https://crates.io/crates/cfg_aliases/0.2.1) |

| chrono 0.4.44 | MIT OR Apache-2.0 | [LICENSE.txt.txt](third_party/licenses/chrono-0.4.44/LICENSE.txt.txt) | [published crate](https://crates.io/crates/chrono/0.4.44) |

| cipher 0.4.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/cipher-0.4.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/cipher-0.4.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/cipher/0.4.4) |

| clap 4.5.57 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/clap-4.5.57/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/clap-4.5.57/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/clap/4.5.57) |

| clap_builder 4.5.57 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/clap_builder-4.5.57/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/clap_builder-4.5.57/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/clap_builder/4.5.57) |

| clap_derive 4.5.55 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/clap_derive-4.5.55/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/clap_derive-4.5.55/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/clap_derive/4.5.55) |

| clap_lex 0.7.7 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/clap_lex-0.7.7/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/clap_lex-0.7.7/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/clap_lex/0.7.7) |

| colorchoice 1.0.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/colorchoice-1.0.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/colorchoice-1.0.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/colorchoice/1.0.4) |

| colored 2.2.0 | MPL-2.0 | [LICENSE.txt](third_party/licenses/colored-2.2.0/LICENSE.txt) | [included archive](third_party/sources/colored-2.2.0.crate) |

| console 0.15.11 | MIT | [LICENSE.txt](third_party/licenses/console-0.15.11/LICENSE.txt) | [published crate](https://crates.io/crates/console/0.15.11) |

| constant_time_eq 0.3.1 | CC0-1.0 OR MIT-0 OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/constant_time_eq-0.3.1/LICENSE-APACHE.txt), [LICENSE-CC0.txt](third_party/licenses/constant_time_eq-0.3.1/LICENSE-CC0.txt), [LICENSE-MIT0.txt](third_party/licenses/constant_time_eq-0.3.1/LICENSE-MIT0.txt) | [published crate](https://crates.io/crates/constant_time_eq/0.3.1) |

| constant_time_eq 0.4.2 | CC0-1.0 OR MIT-0 OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/constant_time_eq-0.4.2/LICENSE-APACHE.txt), [LICENSE-CC0.txt](third_party/licenses/constant_time_eq-0.4.2/LICENSE-CC0.txt), [LICENSE-MIT0.txt](third_party/licenses/constant_time_eq-0.4.2/LICENSE-MIT0.txt) | [published crate](https://crates.io/crates/constant_time_eq/0.4.2) |

| core-foundation-sys 0.8.7 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/core-foundation-sys-0.8.7/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/core-foundation-sys-0.8.7/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/core-foundation-sys/0.8.7) |

| cpufeatures 0.2.17 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/cpufeatures-0.2.17/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/cpufeatures-0.2.17/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/cpufeatures/0.2.17) |

| crc-catalog 2.4.0 | MIT OR Apache-2.0 | [Apache-2.0.txt.txt](third_party/licenses/crc-catalog-2.4.0/LICENSES/Apache-2.0.txt.txt), [MIT.txt.txt](third_party/licenses/crc-catalog-2.4.0/LICENSES/MIT.txt.txt) | [published crate](https://crates.io/crates/crc-catalog/2.4.0) |

| crc32fast 1.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/crc32fast-1.5.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/crc32fast-1.5.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/crc32fast/1.5.0) |

| crc 3.4.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/crc-3.4.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/crc-3.4.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/crc/3.4.0) |

| crossbeam-deque 0.8.6 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/crossbeam-deque-0.8.6/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/crossbeam-deque-0.8.6/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/crossbeam-deque/0.8.6) |

| crossbeam-epoch 0.9.18 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/crossbeam-epoch-0.9.18/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/crossbeam-epoch-0.9.18/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/crossbeam-epoch/0.9.18) |

| crossbeam-utils 0.8.21 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/crossbeam-utils-0.8.21/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/crossbeam-utils-0.8.21/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/crossbeam-utils/0.8.21) |

| crossterm 0.25.0 | MIT | [LICENSE.txt](third_party/licenses/crossterm-0.25.0/LICENSE.txt) | [published crate](https://crates.io/crates/crossterm/0.25.0) |

| crossterm_winapi 0.9.1 | MIT | [LICENSE.txt](third_party/licenses/crossterm_winapi-0.9.1/LICENSE.txt) | [published crate](https://crates.io/crates/crossterm_winapi/0.9.1) |

| crypto-common 0.1.7 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/crypto-common-0.1.7/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/crypto-common-0.1.7/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/crypto-common/0.1.7) |

| csv-core 0.1.13 | Unlicense/MIT | [COPYING.txt](third_party/licenses/csv-core-0.1.13/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/csv-core-0.1.13/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/csv-core/0.1.13) |

| csv 1.4.0 | Unlicense/MIT | [COPYING.txt](third_party/licenses/csv-1.4.0/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/csv-1.4.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/csv/1.4.0) |

| ctrlc 3.5.2 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/ctrlc-3.5.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/ctrlc-3.5.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/ctrlc/3.5.2) |

| curve25519-dalek-derive 0.1.1 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/curve25519-dalek-derive-0.1.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/curve25519-dalek-derive-0.1.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/curve25519-dalek-derive/0.1.1) |

| curve25519-dalek 4.1.3 | BSD-3-Clause | [LICENSE.txt](third_party/licenses/curve25519-dalek-4.1.3/LICENSE.txt) | [published crate](https://crates.io/crates/curve25519-dalek/4.1.3) |

| data-encoding 2.10.0 | MIT | [LICENSE.txt](third_party/licenses/data-encoding-2.10.0/LICENSE.txt) | [published crate](https://crates.io/crates/data-encoding/2.10.0) |

| deflate64 0.1.11 | MIT | [LICENSE.txt](third_party/licenses/deflate64-0.1.11/LICENSE.txt) | [published crate](https://crates.io/crates/deflate64/0.1.11) |

| deranged 0.5.8 | MIT OR Apache-2.0 | [LICENSE-Apache.txt](third_party/licenses/deranged-0.5.8/LICENSE-Apache.txt), [LICENSE-MIT.txt](third_party/licenses/deranged-0.5.8/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/deranged/0.5.8) |

| derive_arbitrary 1.4.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/derive_arbitrary-1.4.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/derive_arbitrary-1.4.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/derive_arbitrary/1.4.2) |

| digest 0.10.7 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/digest-0.10.7/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/digest-0.10.7/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/digest/0.10.7) |

| dirs-next 2.0.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/dirs-next-2.0.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/dirs-next-2.0.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/dirs-next/2.0.0) |

| dirs-sys-next 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/dirs-sys-next-0.1.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/dirs-sys-next-0.1.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/dirs-sys-next/0.1.2) |

| dispatch2 0.3.0 | Zlib OR Apache-2.0 OR MIT | [LICENSE.md.txt](third_party/licenses/dispatch2-0.3.0/LICENSE.md.txt), [LICENSE-MIT.txt](third_party/licenses/dispatch2-0.3.0/LICENSE-MIT.txt), [AUTHORS.txt](third_party/licenses/dispatch2-0.3.0/AUTHORS.txt) | [published crate](https://crates.io/crates/dispatch2/0.3.0) |

| displaydoc 0.2.5 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/displaydoc-0.2.5/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/displaydoc-0.2.5/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/displaydoc/0.2.5) |

| dyn-clone 1.0.20 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/dyn-clone-1.0.20/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/dyn-clone-1.0.20/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/dyn-clone/1.0.20) |

| ed25519-dalek 2.2.0 | BSD-3-Clause | [LICENSE.txt](third_party/licenses/ed25519-dalek-2.2.0/LICENSE.txt) | [published crate](https://crates.io/crates/ed25519-dalek/2.2.0) |

| ed25519 2.2.3 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/ed25519-2.2.3/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/ed25519-2.2.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/ed25519/2.2.3) |

| either 1.15.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/either-1.15.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/either-1.15.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/either/1.15.0) |

| encode_unicode 1.0.0 | Apache-2.0 OR MIT | [AUTHORS.md.txt](third_party/licenses/encode_unicode-1.0.0/AUTHORS.md.txt), [LICENSE-APACHE.txt](third_party/licenses/encode_unicode-1.0.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/encode_unicode-1.0.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/encode_unicode/1.0.0) |

| equivalent 1.0.2 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/equivalent-1.0.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/equivalent-1.0.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/equivalent/1.0.2) |

| errno 0.3.14 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/errno-0.3.14/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/errno-0.3.14/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/errno/0.3.14) |

| fastrand 2.3.0 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/fastrand-2.3.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/fastrand-2.3.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/fastrand/2.3.0) |

| fiat-crypto 0.2.9 | MIT OR Apache-2.0 OR BSD-1-Clause | [AUTHORS.txt](third_party/licenses/fiat-crypto-0.2.9/AUTHORS.txt), [COPYRIGHT.txt](third_party/licenses/fiat-crypto-0.2.9/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/fiat-crypto-0.2.9/LICENSE-APACHE.txt), [LICENSE-BSD-1.txt](third_party/licenses/fiat-crypto-0.2.9/LICENSE-BSD-1.txt), [LICENSE-MIT.txt](third_party/licenses/fiat-crypto-0.2.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/fiat-crypto/0.2.9) |

| find-msvc-tools 0.1.9 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/find-msvc-tools-0.1.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/find-msvc-tools-0.1.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/find-msvc-tools/0.1.9) |

| flate2 1.1.9 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/flate2-1.1.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/flate2-1.1.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/flate2/1.1.9) |

| form_urlencoded 1.2.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/form_urlencoded-1.2.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/form_urlencoded-1.2.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/form_urlencoded/1.2.2) |

| fs2 0.4.3 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/fs2-0.4.3/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/fs2-0.4.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/fs2/0.4.3) |

| fuzzy-matcher 0.3.7 | MIT | [LICENSE.txt](third_party/licenses/fuzzy-matcher-0.3.7/LICENSE.txt) | [published crate](https://crates.io/crates/fuzzy-matcher/0.3.7) |

| fxhash 0.2.1 | Apache-2.0/MIT | [SOURCE-ATTRIBUTION.txt](third_party/licenses/fxhash-0.2.1/SOURCE-ATTRIBUTION.txt), [LICENSE-APACHE.txt](third_party/licenses/fxhash-0.2.1/LICENSE-APACHE.txt) | [published crate](https://crates.io/crates/fxhash/0.2.1) |

| generic-array 0.14.7 | MIT | [LICENSE.txt](third_party/licenses/generic-array-0.14.7/LICENSE.txt) | [published crate](https://crates.io/crates/generic-array/0.14.7) |

| getrandom 0.2.17 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/getrandom-0.2.17/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/getrandom-0.2.17/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/getrandom/0.2.17) |

| getrandom 0.3.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/getrandom-0.3.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/getrandom-0.3.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/getrandom/0.3.4) |

| getrandom 0.4.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/getrandom-0.4.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/getrandom-0.4.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/getrandom/0.4.1) |

| git2 0.20.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/git2-0.20.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/git2-0.20.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/git2/0.20.4) |

| globset 0.4.18 | Unlicense OR MIT | [COPYING.txt](third_party/licenses/globset-0.4.18/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/globset-0.4.18/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/globset/0.4.18) |

| hashbrown 0.16.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/hashbrown-0.16.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/hashbrown-0.16.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/hashbrown/0.16.1) |

| heck 0.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/heck-0.5.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/heck-0.5.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/heck/0.5.0) |

| hermit-abi 0.5.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/hermit-abi-0.5.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/hermit-abi-0.5.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/hermit-abi/0.5.2) |

| hex 0.4.3 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/hex-0.4.3/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/hex-0.4.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/hex/0.4.3) |

| hmac 0.12.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/hmac-0.12.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/hmac-0.12.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/hmac/0.12.1) |

| iana-time-zone-haiku 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/iana-time-zone-haiku-0.1.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/iana-time-zone-haiku-0.1.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/iana-time-zone-haiku/0.1.2) |

| iana-time-zone 0.1.65 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/iana-time-zone-0.1.65/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/iana-time-zone-0.1.65/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/iana-time-zone/0.1.65) |

| icu_collections 2.1.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_collections-2.1.1/LICENSE.txt) | [published crate](https://crates.io/crates/icu_collections/2.1.1) |

| icu_locale_core 2.1.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_locale_core-2.1.1/LICENSE.txt) | [published crate](https://crates.io/crates/icu_locale_core/2.1.1) |

| icu_normalizer 2.1.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_normalizer-2.1.1/LICENSE.txt) | [published crate](https://crates.io/crates/icu_normalizer/2.1.1) |

| icu_normalizer_data 2.1.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_normalizer_data-2.1.1/LICENSE.txt) | [published crate](https://crates.io/crates/icu_normalizer_data/2.1.1) |

| icu_properties 2.1.2 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_properties-2.1.2/LICENSE.txt) | [published crate](https://crates.io/crates/icu_properties/2.1.2) |

| icu_properties_data 2.1.2 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_properties_data-2.1.2/LICENSE.txt) | [published crate](https://crates.io/crates/icu_properties_data/2.1.2) |

| icu_provider 2.1.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/icu_provider-2.1.1/LICENSE.txt) | [published crate](https://crates.io/crates/icu_provider/2.1.1) |

| idna 1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/idna-1.1.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/idna-1.1.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/idna/1.1.0) |

| idna_adapter 1.2.1 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/idna_adapter-1.2.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/idna_adapter-1.2.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/idna_adapter/1.2.1) |

| ignore 0.4.25 | Unlicense OR MIT | [COPYING.txt](third_party/licenses/ignore-0.4.25/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/ignore-0.4.25/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/ignore/0.4.25) |

| indexmap 2.13.0 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/indexmap-2.13.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/indexmap-2.13.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/indexmap/2.13.0) |

| indicatif 0.17.11 | MIT | [LICENSE.txt](third_party/licenses/indicatif-0.17.11/LICENSE.txt) | [published crate](https://crates.io/crates/indicatif/0.17.11) |

| inout 0.1.4 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/inout-0.1.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/inout-0.1.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/inout/0.1.4) |

| inquire 0.7.5 | MIT | [LICENSE.txt](third_party/licenses/inquire-0.7.5/LICENSE.txt) | [published crate](https://crates.io/crates/inquire/0.7.5) |

| is-terminal 0.4.17 | MIT | [LICENSE-MIT.txt](third_party/licenses/is-terminal-0.4.17/LICENSE-MIT.txt), [LICENSE-MIT-atty.txt](third_party/licenses/is-terminal-0.4.17/LICENSE-MIT-atty.txt) | [published crate](https://crates.io/crates/is-terminal/0.4.17) |

| is_terminal_polyfill 1.70.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/is_terminal_polyfill-1.70.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/is_terminal_polyfill-1.70.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/is_terminal_polyfill/1.70.2) |

| itoa 1.0.17 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/itoa-1.0.17/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/itoa-1.0.17/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/itoa/1.0.17) |

| jobserver 0.1.34 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/jobserver-0.1.34/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/jobserver-0.1.34/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/jobserver/0.1.34) |

| js-sys 0.3.85 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/js-sys-0.3.85/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/js-sys-0.3.85/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/js-sys/0.3.85) |

| lazy_static 1.5.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/lazy_static-1.5.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/lazy_static-1.5.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/lazy_static/1.5.0) |

| libbz2-rs-sys 0.2.2 | bzip2-1.0.6 | [LICENSE.txt](third_party/licenses/libbz2-rs-sys-0.2.2/LICENSE.txt) | [published crate](https://crates.io/crates/libbz2-rs-sys/0.2.2) |

| libc 0.2.180 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/libc-0.2.180/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/libc-0.2.180/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/libc/0.2.180) |

| libgit2-sys 0.18.8+1.9.7 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/LICENSE-MIT.txt), [AUTHORS.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/AUTHORS.txt), [COPYING.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/deps/llhttp/LICENSE-MIT.txt), [LICENCE.md.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/deps/pcre2/LICENCE.md.txt), [COPYING.GPL.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/deps/winhttp/COPYING.GPL.txt), [COPYING.LGPL.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/deps/winhttp/COPYING.LGPL.txt), [LICENSE.txt](third_party/licenses/libgit2-sys-0.18.8+1.9.7/libgit2/deps/zlib/LICENSE.txt) | [included archive](third_party/sources/libgit2-sys-0.18.8+1.9.7.crate) |

| libredox 0.1.12 | MIT | [LICENSE.txt](third_party/licenses/libredox-0.1.12/LICENSE.txt) | [published crate](https://crates.io/crates/libredox/0.1.12) |

| libz-sys 1.1.23 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/libz-sys-1.1.23/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/libz-sys-1.1.23/LICENSE-MIT.txt), [LICENSE.md.txt](third_party/licenses/libz-sys-1.1.23/src/zlib-ng/LICENSE.md.txt), [LICENSE.txt](third_party/licenses/libz-sys-1.1.23/src/zlib/LICENSE.txt), [LICENSE_1_0.txt.txt](third_party/licenses/libz-sys-1.1.23/src/zlib/contrib/dotzlib/LICENSE_1_0.txt.txt) | [published crate](https://crates.io/crates/libz-sys/1.1.23) |

| linux-raw-sys 0.11.0 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [COPYRIGHT.txt](third_party/licenses/linux-raw-sys-0.11.0/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/linux-raw-sys-0.11.0/LICENSE-APACHE.txt), [LICENSE-Apache-2.0_WITH_LLVM-exception.txt](third_party/licenses/linux-raw-sys-0.11.0/LICENSE-Apache-2.0_WITH_LLVM-exception.txt), [LICENSE-MIT.txt](third_party/licenses/linux-raw-sys-0.11.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/linux-raw-sys/0.11.0) |

| litemap 0.8.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/litemap-0.8.1/LICENSE.txt) | [published crate](https://crates.io/crates/litemap/0.8.1) |

| lock_api 0.4.14 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/lock_api-0.4.14/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/lock_api-0.4.14/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/lock_api/0.4.14) |

| log 0.4.29 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/log-0.4.29/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/log-0.4.29/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/log/0.4.29) |

| lzma-rust2 0.13.0 | Apache-2.0 | [LICENSE.txt](third_party/licenses/lzma-rust2-0.13.0/LICENSE.txt) | [published crate](https://crates.io/crates/lzma-rust2/0.13.0) |

| matchers 0.2.0 | MIT | [LICENSE.txt](third_party/licenses/matchers-0.2.0/LICENSE.txt) | [published crate](https://crates.io/crates/matchers/0.2.0) |

| memchr 2.8.0 | Unlicense OR MIT | [COPYING.txt](third_party/licenses/memchr-2.8.0/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/memchr-2.8.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/memchr/2.8.0) |

| miniz_oxide 0.8.9 | MIT OR Zlib OR Apache-2.0 | [LICENSE.txt](third_party/licenses/miniz_oxide-0.8.9/LICENSE.txt), [LICENSE-APACHE.md.txt](third_party/licenses/miniz_oxide-0.8.9/LICENSE-APACHE.md.txt), [LICENSE-MIT.md.txt](third_party/licenses/miniz_oxide-0.8.9/LICENSE-MIT.md.txt), [LICENSE-ZLIB.md.txt](third_party/licenses/miniz_oxide-0.8.9/LICENSE-ZLIB.md.txt) | [published crate](https://crates.io/crates/miniz_oxide/0.8.9) |

| mio 0.8.11 | MIT | [LICENSE.txt](third_party/licenses/mio-0.8.11/LICENSE.txt) | [published crate](https://crates.io/crates/mio/0.8.11) |

| newline-converter 0.3.0 | MIT | [LICENSE.txt](third_party/licenses/newline-converter-0.3.0/LICENSE.txt) | [published crate](https://crates.io/crates/newline-converter/0.3.0) |

| nix 0.31.1 | MIT | [LICENSE.txt](third_party/licenses/nix-0.31.1/LICENSE.txt) | [published crate](https://crates.io/crates/nix/0.31.1) |

| nu-ansi-term 0.50.3 | MIT | [LICENSE.txt](third_party/licenses/nu-ansi-term-0.50.3/LICENSE.txt) | [published crate](https://crates.io/crates/nu-ansi-term/0.50.3) |

| num-conv 0.2.0 | MIT OR Apache-2.0 | [LICENSE-Apache.txt](third_party/licenses/num-conv-0.2.0/LICENSE-Apache.txt), [LICENSE-MIT.txt](third_party/licenses/num-conv-0.2.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/num-conv/0.2.0) |

| num-traits 0.2.19 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/num-traits-0.2.19/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/num-traits-0.2.19/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/num-traits/0.2.19) |

| number_prefix 0.4.0 | MIT | [LICENCE.txt](third_party/licenses/number_prefix-0.4.0/LICENCE.txt) | [published crate](https://crates.io/crates/number_prefix/0.4.0) |

| objc2-encode 4.1.0 | MIT | [LICENSE.md.txt](third_party/licenses/objc2-encode-4.1.0/LICENSE.md.txt), [LICENSE-MIT.txt](third_party/licenses/objc2-encode-4.1.0/LICENSE-MIT.txt), [AUTHORS.txt](third_party/licenses/objc2-encode-4.1.0/AUTHORS.txt) | [published crate](https://crates.io/crates/objc2-encode/4.1.0) |

| objc2 0.6.3 | MIT | [LICENSE.md.txt](third_party/licenses/objc2-0.6.3/LICENSE.md.txt), [LICENSE-MIT.txt](third_party/licenses/objc2-0.6.3/LICENSE-MIT.txt), [AUTHORS.txt](third_party/licenses/objc2-0.6.3/AUTHORS.txt) | [published crate](https://crates.io/crates/objc2/0.6.3) |

| once_cell 1.21.3 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/once_cell-1.21.3/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/once_cell-1.21.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/once_cell/1.21.3) |

| once_cell_polyfill 1.70.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/once_cell_polyfill-1.70.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/once_cell_polyfill-1.70.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/once_cell_polyfill/1.70.2) |

| parking_lot 0.12.5 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/parking_lot-0.12.5/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/parking_lot-0.12.5/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/parking_lot/0.12.5) |

| parking_lot_core 0.9.12 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/parking_lot_core-0.9.12/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/parking_lot_core-0.9.12/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/parking_lot_core/0.9.12) |

| pbkdf2 0.12.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/pbkdf2-0.12.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/pbkdf2-0.12.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/pbkdf2/0.12.2) |

| percent-encoding 2.3.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/percent-encoding-2.3.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/percent-encoding-2.3.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/percent-encoding/2.3.2) |

| pin-project-lite 0.2.16 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/pin-project-lite-0.2.16/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/pin-project-lite-0.2.16/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/pin-project-lite/0.2.16) |

| pkg-config 0.3.32 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/pkg-config-0.3.32/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/pkg-config-0.3.32/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/pkg-config/0.3.32) |

| portable-atomic 1.13.1 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/portable-atomic-1.13.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/portable-atomic-1.13.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/portable-atomic/1.13.1) |

| potential_utf 0.1.4 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/potential_utf-0.1.4/LICENSE.txt) | [published crate](https://crates.io/crates/potential_utf/0.1.4) |

| powerfmt 0.2.0 | MIT OR Apache-2.0 | [LICENSE-Apache.txt](third_party/licenses/powerfmt-0.2.0/LICENSE-Apache.txt), [LICENSE-MIT.txt](third_party/licenses/powerfmt-0.2.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/powerfmt/0.2.0) |

| ppmd-rust 1.4.0 | CC0-1.0 OR MIT-0 | [LICENSE-CC0.md.txt](third_party/licenses/ppmd-rust-1.4.0/LICENSE-CC0.md.txt), [LICENSE-MIT-0.md.txt](third_party/licenses/ppmd-rust-1.4.0/LICENSE-MIT-0.md.txt) | [published crate](https://crates.io/crates/ppmd-rust/1.4.0) |


| prettytable-rs 0.10.0 | BSD-3-Clause | [LICENSE.txt.txt](third_party/licenses/prettytable-rs-0.10.0/LICENSE.txt.txt) | [published crate](https://crates.io/crates/prettytable-rs/0.10.0) |

| proc-macro2 1.0.106 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/proc-macro2-1.0.106/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/proc-macro2-1.0.106/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/proc-macro2/1.0.106) |

| quote 1.0.44 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/quote-1.0.44/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/quote-1.0.44/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/quote/1.0.44) |

| r-efi 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [AUTHORS.txt](third_party/licenses/r-efi-5.3.0/AUTHORS.txt) | [published crate](https://crates.io/crates/r-efi/5.3.0) |




| rayon-core 1.13.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/rayon-core-1.13.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/rayon-core-1.13.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/rayon-core/1.13.0) |

| rayon 1.11.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/rayon-1.11.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/rayon-1.11.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/rayon/1.11.0) |

| redox_syscall 0.5.18 | MIT | [LICENSE.txt](third_party/licenses/redox_syscall-0.5.18/LICENSE.txt) | [published crate](https://crates.io/crates/redox_syscall/0.5.18) |

| redox_users 0.4.6 | MIT | [LICENSE.txt](third_party/licenses/redox_users-0.4.6/LICENSE.txt) | [published crate](https://crates.io/crates/redox_users/0.4.6) |

| regex-automata 0.4.14 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/regex-automata-0.4.14/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/regex-automata-0.4.14/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/regex-automata/0.4.14) |

| regex-syntax 0.8.9 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/regex-syntax-0.8.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/regex-syntax-0.8.9/LICENSE-MIT.txt), [LICENSE-UNICODE.txt](third_party/licenses/regex-syntax-0.8.9/src/unicode_tables/LICENSE-UNICODE.txt) | [published crate](https://crates.io/crates/regex-syntax/0.8.9) |

| regex 1.12.3 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/regex-1.12.3/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/regex-1.12.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/regex/1.12.3) |

| rust-embed-impl 8.11.0 | MIT | [license.txt](third_party/licenses/rust-embed-impl-8.11.0/license.txt) | [published crate](https://crates.io/crates/rust-embed-impl/8.11.0) |

| rust-embed-utils 8.11.0 | MIT | [license.txt](third_party/licenses/rust-embed-utils-8.11.0/license.txt) | [published crate](https://crates.io/crates/rust-embed-utils/8.11.0) |

| rust-embed 8.11.0 | MIT | [LICENSE.txt](third_party/licenses/rust-embed-8.11.0/LICENSE.txt) | [published crate](https://crates.io/crates/rust-embed/8.11.0) |

| rustc_version 0.4.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/rustc_version-0.4.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/rustc_version-0.4.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/rustc_version/0.4.1) |

| rustix 1.1.3 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [COPYRIGHT.txt](third_party/licenses/rustix-1.1.3/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/rustix-1.1.3/LICENSE-APACHE.txt), [LICENSE-Apache-2.0_WITH_LLVM-exception.txt](third_party/licenses/rustix-1.1.3/LICENSE-Apache-2.0_WITH_LLVM-exception.txt), [LICENSE-MIT.txt](third_party/licenses/rustix-1.1.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/rustix/1.1.3) |

| rustversion 1.0.22 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/rustversion-1.0.22/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/rustversion-1.0.22/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/rustversion/1.0.22) |

| ryu 1.0.23 | Apache-2.0 OR BSL-1.0 | [LICENSE-APACHE.txt](third_party/licenses/ryu-1.0.23/LICENSE-APACHE.txt), [LICENSE-BOOST.txt](third_party/licenses/ryu-1.0.23/LICENSE-BOOST.txt) | [published crate](https://crates.io/crates/ryu/1.0.23) |

| same-file 1.0.6 | Unlicense/MIT | [COPYING.txt](third_party/licenses/same-file-1.0.6/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/same-file-1.0.6/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/same-file/1.0.6) |

| scopeguard 1.2.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/scopeguard-1.2.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/scopeguard-1.2.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/scopeguard/1.2.0) |

| semver 1.0.27 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/semver-1.0.27/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/semver-1.0.27/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/semver/1.0.27) |

| serde 1.0.228 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/serde-1.0.228/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/serde-1.0.228/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/serde/1.0.228) |

| serde_core 1.0.228 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/serde_core-1.0.228/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/serde_core-1.0.228/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/serde_core/1.0.228) |

| serde_derive 1.0.228 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/serde_derive-1.0.228/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/serde_derive-1.0.228/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/serde_derive/1.0.228) |

| serde_json 1.0.149 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/serde_json-1.0.149/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/serde_json-1.0.149/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/serde_json/1.0.149) |

| serde_spanned 0.6.9 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/serde_spanned-0.6.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/serde_spanned-0.6.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/serde_spanned/0.6.9) |

| sha1 0.10.6 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/sha1-0.10.6/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/sha1-0.10.6/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/sha1/0.10.6) |

| sha2 0.10.9 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/sha2-0.10.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/sha2-0.10.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/sha2/0.10.9) |

| sharded-slab 0.1.7 | MIT | [LICENSE.txt](third_party/licenses/sharded-slab-0.1.7/LICENSE.txt) | [published crate](https://crates.io/crates/sharded-slab/0.1.7) |

| shlex 1.3.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/shlex-1.3.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/shlex-1.3.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/shlex/1.3.0) |

| signal-hook-mio 0.2.5 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/signal-hook-mio-0.2.5/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/signal-hook-mio-0.2.5/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/signal-hook-mio/0.2.5) |

| signal-hook-registry 1.4.8 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/signal-hook-registry-1.4.8/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/signal-hook-registry-1.4.8/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/signal-hook-registry/1.4.8) |

| signal-hook 0.3.18 | Apache-2.0/MIT | [LICENSE-APACHE.txt](third_party/licenses/signal-hook-0.3.18/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/signal-hook-0.3.18/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/signal-hook/0.3.18) |

| signature 2.2.0 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/signature-2.2.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/signature-2.2.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/signature/2.2.0) |

| simd-adler32 0.3.8 | MIT | [LICENSE.md.txt](third_party/licenses/simd-adler32-0.3.8/LICENSE.md.txt) | [published crate](https://crates.io/crates/simd-adler32/0.3.8) |

| smallvec 1.15.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/smallvec-1.15.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/smallvec-1.15.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/smallvec/1.15.1) |

| stable_deref_trait 1.2.1 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/stable_deref_trait-1.2.1/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/stable_deref_trait-1.2.1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/stable_deref_trait/1.2.1) |

| strsim 0.11.1 | MIT | [LICENSE.txt](third_party/licenses/strsim-0.11.1/LICENSE.txt) | [published crate](https://crates.io/crates/strsim/0.11.1) |

| subtle 2.6.1 | BSD-3-Clause | [LICENSE.txt](third_party/licenses/subtle-2.6.1/LICENSE.txt) | [published crate](https://crates.io/crates/subtle/2.6.1) |

| syn 2.0.114 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/syn-2.0.114/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/syn-2.0.114/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/syn/2.0.114) |

| synstructure 0.13.2 | MIT | [LICENSE.txt](third_party/licenses/synstructure-0.13.2/LICENSE.txt) | [published crate](https://crates.io/crates/synstructure/0.13.2) |

| tempfile 3.25.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/tempfile-3.25.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/tempfile-3.25.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/tempfile/3.25.0) |

| term 0.7.0 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/term-0.7.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/term-0.7.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/term/0.7.0) |

| thiserror-impl 1.0.69 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/thiserror-impl-1.0.69/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/thiserror-impl-1.0.69/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/thiserror-impl/1.0.69) |

| thiserror 1.0.69 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/thiserror-1.0.69/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/thiserror-1.0.69/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/thiserror/1.0.69) |

| thread_local 1.1.9 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/thread_local-1.1.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/thread_local-1.1.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/thread_local/1.1.9) |

| time-core 0.1.8 | MIT OR Apache-2.0 | [LICENSE-Apache.txt](third_party/licenses/time-core-0.1.8/LICENSE-Apache.txt), [LICENSE-MIT.txt](third_party/licenses/time-core-0.1.8/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/time-core/0.1.8) |

| time 0.3.47 | MIT OR Apache-2.0 | [LICENSE-Apache.txt](third_party/licenses/time-0.3.47/LICENSE-Apache.txt), [LICENSE-MIT.txt](third_party/licenses/time-0.3.47/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/time/0.3.47) |

| tinystr 0.8.2 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/tinystr-0.8.2/LICENSE.txt) | [published crate](https://crates.io/crates/tinystr/0.8.2) |

| toml 0.8.23 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/toml-0.8.23/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/toml-0.8.23/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/toml/0.8.23) |

| toml_datetime 0.6.11 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/toml_datetime-0.6.11/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/toml_datetime-0.6.11/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/toml_datetime/0.6.11) |

| toml_edit 0.22.27 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/toml_edit-0.22.27/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/toml_edit-0.22.27/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/toml_edit/0.22.27) |

| toml_write 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/toml_write-0.1.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/toml_write-0.1.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/toml_write/0.1.2) |

| tracing-attributes 0.1.31 | MIT | [LICENSE.txt](third_party/licenses/tracing-attributes-0.1.31/LICENSE.txt) | [published crate](https://crates.io/crates/tracing-attributes/0.1.31) |

| tracing-core 0.1.36 | MIT | [LICENSE.txt](third_party/licenses/tracing-core-0.1.36/LICENSE.txt), [LICENSE.txt](third_party/licenses/tracing-core-0.1.36/src/spin/LICENSE.txt) | [published crate](https://crates.io/crates/tracing-core/0.1.36) |

| tracing-log 0.2.0 | MIT | [LICENSE.txt](third_party/licenses/tracing-log-0.2.0/LICENSE.txt) | [published crate](https://crates.io/crates/tracing-log/0.2.0) |

| tracing-subscriber 0.3.22 | MIT | [LICENSE.txt](third_party/licenses/tracing-subscriber-0.3.22/LICENSE.txt) | [published crate](https://crates.io/crates/tracing-subscriber/0.3.22) |

| tracing 0.1.44 | MIT | [LICENSE.txt](third_party/licenses/tracing-0.1.44/LICENSE.txt) | [published crate](https://crates.io/crates/tracing/0.1.44) |

| typenum 1.19.0 | MIT OR Apache-2.0 | [LICENSE.txt](third_party/licenses/typenum-1.19.0/LICENSE.txt), [LICENSE-APACHE.txt](third_party/licenses/typenum-1.19.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/typenum-1.19.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/typenum/1.19.0) |

| unicode-ident 1.0.23 | (MIT OR Apache-2.0) AND Unicode-3.0 | [LICENSE-APACHE.txt](third_party/licenses/unicode-ident-1.0.23/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/unicode-ident-1.0.23/LICENSE-MIT.txt), [LICENSE-UNICODE.txt](third_party/licenses/unicode-ident-1.0.23/LICENSE-UNICODE.txt) | [published crate](https://crates.io/crates/unicode-ident/1.0.23) |

| unicode-segmentation 1.12.0 | MIT OR Apache-2.0 | [COPYRIGHT.txt](third_party/licenses/unicode-segmentation-1.12.0/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/unicode-segmentation-1.12.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/unicode-segmentation-1.12.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/unicode-segmentation/1.12.0) |

| unicode-width 0.1.14 | MIT OR Apache-2.0 | [COPYRIGHT.txt](third_party/licenses/unicode-width-0.1.14/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/unicode-width-0.1.14/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/unicode-width-0.1.14/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/unicode-width/0.1.14) |

| unicode-width 0.2.2 | MIT OR Apache-2.0 | [COPYRIGHT.txt](third_party/licenses/unicode-width-0.2.2/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/unicode-width-0.2.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/unicode-width-0.2.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/unicode-width/0.2.2) |

| url 2.5.8 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/url-2.5.8/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/url-2.5.8/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/url/2.5.8) |

| utf8_iter 1.0.4 | Apache-2.0 OR MIT | [COPYRIGHT.txt](third_party/licenses/utf8_iter-1.0.4/COPYRIGHT.txt), [LICENSE-APACHE.txt](third_party/licenses/utf8_iter-1.0.4/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/utf8_iter-1.0.4/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/utf8_iter/1.0.4) |

| utf8parse 0.2.2 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/utf8parse-0.2.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/utf8parse-0.2.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/utf8parse/0.2.2) |

| vcpkg 0.2.15 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/vcpkg-0.2.15/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/vcpkg-0.2.15/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/vcpkg/0.2.15) |

| version_check 0.9.5 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/version_check-0.9.5/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/version_check-0.9.5/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/version_check/0.9.5) |

| walkdir 2.5.0 | Unlicense/MIT | [COPYING.txt](third_party/licenses/walkdir-2.5.0/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/walkdir-2.5.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/walkdir/2.5.0) |

| wasi 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/wasi-0.11.1+wasi-snapshot-preview1/LICENSE-APACHE.txt), [LICENSE-Apache-2.0_WITH_LLVM-exception.txt](third_party/licenses/wasi-0.11.1+wasi-snapshot-preview1/LICENSE-Apache-2.0_WITH_LLVM-exception.txt), [LICENSE-MIT.txt](third_party/licenses/wasi-0.11.1+wasi-snapshot-preview1/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasi/0.11.1+wasi-snapshot-preview1) |

| wasip2 1.0.2+wasi-0.2.9 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/wasip2-1.0.2+wasi-0.2.9/LICENSE-APACHE.txt), [LICENSE-Apache-2.0_WITH_LLVM-exception.txt](third_party/licenses/wasip2-1.0.2+wasi-0.2.9/LICENSE-Apache-2.0_WITH_LLVM-exception.txt), [LICENSE-MIT.txt](third_party/licenses/wasip2-1.0.2+wasi-0.2.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasip2/1.0.2+wasi-0.2.9) |

| wasip3 0.4.0+wasi-0.3.0-rc-2026-01-06 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06/LICENSE-APACHE.txt), [LICENSE-Apache-2.0_WITH_LLVM-exception.txt](third_party/licenses/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06/LICENSE-Apache-2.0_WITH_LLVM-exception.txt), [LICENSE-MIT.txt](third_party/licenses/wasip3-0.4.0+wasi-0.3.0-rc-2026-01-06/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasip3/0.4.0+wasi-0.3.0-rc-2026-01-06) |

| wasm-bindgen-macro-support 0.2.108 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/wasm-bindgen-macro-support-0.2.108/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/wasm-bindgen-macro-support-0.2.108/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasm-bindgen-macro-support/0.2.108) |

| wasm-bindgen-macro 0.2.108 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/wasm-bindgen-macro-0.2.108/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/wasm-bindgen-macro-0.2.108/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasm-bindgen-macro/0.2.108) |

| wasm-bindgen-shared 0.2.108 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/wasm-bindgen-shared-0.2.108/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/wasm-bindgen-shared-0.2.108/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasm-bindgen-shared/0.2.108) |

| wasm-bindgen 0.2.108 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/wasm-bindgen-0.2.108/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/wasm-bindgen-0.2.108/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wasm-bindgen/0.2.108) |

| web-time 1.1.0 | MIT OR Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/web-time-1.1.0/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/web-time-1.1.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/web-time/1.1.0) |

| winapi-i686-pc-windows-gnu 0.4.0 | MIT/Apache-2.0 | [SOURCE-ATTRIBUTION.txt](third_party/licenses/winapi-i686-pc-windows-gnu-0.4.0/SOURCE-ATTRIBUTION.txt), [LICENSE-APACHE.txt](third_party/licenses/winapi-i686-pc-windows-gnu-0.4.0/LICENSE-APACHE.txt) | [published crate](https://crates.io/crates/winapi-i686-pc-windows-gnu/0.4.0) |

| winapi-util 0.1.11 | Unlicense OR MIT | [COPYING.txt](third_party/licenses/winapi-util-0.1.11/COPYING.txt), [LICENSE-MIT.txt](third_party/licenses/winapi-util-0.1.11/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/winapi-util/0.1.11) |

| winapi-x86_64-pc-windows-gnu 0.4.0 | MIT/Apache-2.0 | [SOURCE-ATTRIBUTION.txt](third_party/licenses/winapi-x86_64-pc-windows-gnu-0.4.0/SOURCE-ATTRIBUTION.txt), [LICENSE-APACHE.txt](third_party/licenses/winapi-x86_64-pc-windows-gnu-0.4.0/LICENSE-APACHE.txt) | [published crate](https://crates.io/crates/winapi-x86_64-pc-windows-gnu/0.4.0) |

| winapi 0.3.9 | MIT/Apache-2.0 | [LICENSE-APACHE.txt](third_party/licenses/winapi-0.3.9/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/winapi-0.3.9/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/winapi/0.3.9) |

| windows-core 0.62.2 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-core-0.62.2/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-core-0.62.2/license-mit.txt) | [published crate](https://crates.io/crates/windows-core/0.62.2) |

| windows-implement 0.60.2 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-implement-0.60.2/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-implement-0.60.2/license-mit.txt) | [published crate](https://crates.io/crates/windows-implement/0.60.2) |

| windows-interface 0.59.3 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-interface-0.59.3/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-interface-0.59.3/license-mit.txt) | [published crate](https://crates.io/crates/windows-interface/0.59.3) |

| windows-link 0.2.1 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-link-0.2.1/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-link-0.2.1/license-mit.txt) | [published crate](https://crates.io/crates/windows-link/0.2.1) |

| windows-result 0.4.1 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-result-0.4.1/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-result-0.4.1/license-mit.txt) | [published crate](https://crates.io/crates/windows-result/0.4.1) |

| windows-strings 0.5.1 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-strings-0.5.1/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-strings-0.5.1/license-mit.txt) | [published crate](https://crates.io/crates/windows-strings/0.5.1) |

| windows-sys 0.48.0 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-sys-0.48.0/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-sys-0.48.0/license-mit.txt) | [published crate](https://crates.io/crates/windows-sys/0.48.0) |

| windows-sys 0.59.0 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-sys-0.59.0/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-sys-0.59.0/license-mit.txt) | [published crate](https://crates.io/crates/windows-sys/0.59.0) |

| windows-sys 0.61.2 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-sys-0.61.2/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-sys-0.61.2/license-mit.txt) | [published crate](https://crates.io/crates/windows-sys/0.61.2) |

| windows-targets 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-targets-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-targets-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows-targets/0.48.5) |

| windows-targets 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows-targets-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows-targets-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows-targets/0.52.6) |

| windows_aarch64_gnullvm 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_aarch64_gnullvm-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_aarch64_gnullvm-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_aarch64_gnullvm/0.48.5) |

| windows_aarch64_gnullvm 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_aarch64_gnullvm-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_aarch64_gnullvm-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_aarch64_gnullvm/0.52.6) |

| windows_aarch64_msvc 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_aarch64_msvc-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_aarch64_msvc-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_aarch64_msvc/0.48.5) |

| windows_aarch64_msvc 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_aarch64_msvc-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_aarch64_msvc-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_aarch64_msvc/0.52.6) |

| windows_i686_gnu 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_i686_gnu-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_i686_gnu-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_i686_gnu/0.48.5) |

| windows_i686_gnu 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_i686_gnu-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_i686_gnu-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_i686_gnu/0.52.6) |

| windows_i686_gnullvm 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_i686_gnullvm-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_i686_gnullvm-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_i686_gnullvm/0.52.6) |

| windows_i686_msvc 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_i686_msvc-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_i686_msvc-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_i686_msvc/0.48.5) |

| windows_i686_msvc 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_i686_msvc-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_i686_msvc-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_i686_msvc/0.52.6) |

| windows_x86_64_gnu 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_x86_64_gnu-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_x86_64_gnu-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_x86_64_gnu/0.48.5) |

| windows_x86_64_gnu 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_x86_64_gnu-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_x86_64_gnu-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_x86_64_gnu/0.52.6) |

| windows_x86_64_gnullvm 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_x86_64_gnullvm-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_x86_64_gnullvm-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_x86_64_gnullvm/0.48.5) |

| windows_x86_64_gnullvm 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_x86_64_gnullvm-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_x86_64_gnullvm-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_x86_64_gnullvm/0.52.6) |

| windows_x86_64_msvc 0.48.5 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_x86_64_msvc-0.48.5/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_x86_64_msvc-0.48.5/license-mit.txt) | [published crate](https://crates.io/crates/windows_x86_64_msvc/0.48.5) |

| windows_x86_64_msvc 0.52.6 | MIT OR Apache-2.0 | [license-apache-2.0.txt](third_party/licenses/windows_x86_64_msvc-0.52.6/license-apache-2.0.txt), [license-mit.txt](third_party/licenses/windows_x86_64_msvc-0.52.6/license-mit.txt) | [published crate](https://crates.io/crates/windows_x86_64_msvc/0.52.6) |

| winnow 0.7.14 | MIT | [LICENSE-MIT.txt](third_party/licenses/winnow-0.7.14/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/winnow/0.7.14) |

| wit-bindgen 0.51.0 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/wit-bindgen-0.51.0/LICENSE-APACHE.txt), [LICENSE-Apache-2.0_WITH_LLVM-exception.txt](third_party/licenses/wit-bindgen-0.51.0/LICENSE-Apache-2.0_WITH_LLVM-exception.txt), [LICENSE-MIT.txt](third_party/licenses/wit-bindgen-0.51.0/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/wit-bindgen/0.51.0) |

| writeable 0.6.2 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/writeable-0.6.2/LICENSE.txt) | [published crate](https://crates.io/crates/writeable/0.6.2) |

| yoke-derive 0.8.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/yoke-derive-0.8.1/LICENSE.txt) | [published crate](https://crates.io/crates/yoke-derive/0.8.1) |

| yoke 0.8.1 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/yoke-0.8.1/LICENSE.txt) | [published crate](https://crates.io/crates/yoke/0.8.1) |



| zerofrom-derive 0.1.6 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/zerofrom-derive-0.1.6/LICENSE.txt) | [published crate](https://crates.io/crates/zerofrom-derive/0.1.6) |

| zerofrom 0.1.6 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/zerofrom-0.1.6/LICENSE.txt) | [published crate](https://crates.io/crates/zerofrom/0.1.6) |

| zeroize 1.8.2 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/zeroize-1.8.2/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/zeroize-1.8.2/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/zeroize/1.8.2) |

| zeroize_derive 1.4.3 | Apache-2.0 OR MIT | [LICENSE-APACHE.txt](third_party/licenses/zeroize_derive-1.4.3/LICENSE-APACHE.txt), [LICENSE-MIT.txt](third_party/licenses/zeroize_derive-1.4.3/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/zeroize_derive/1.4.3) |

| zerotrie 0.2.3 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/zerotrie-0.2.3/LICENSE.txt) | [published crate](https://crates.io/crates/zerotrie/0.2.3) |

| zerovec-derive 0.11.2 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/zerovec-derive-0.11.2/LICENSE.txt) | [published crate](https://crates.io/crates/zerovec-derive/0.11.2) |

| zerovec 0.11.5 | Unicode-3.0 | [LICENSE.txt](third_party/licenses/zerovec-0.11.5/LICENSE.txt) | [published crate](https://crates.io/crates/zerovec/0.11.5) |

| zip 5.1.1 | MIT | [LICENSE.txt](third_party/licenses/zip-5.1.1/LICENSE.txt) | [published crate](https://crates.io/crates/zip/5.1.1) |

| zlib-rs 0.6.2 | Zlib | [LICENSE.txt](third_party/licenses/zlib-rs-0.6.2/LICENSE.txt) | [published crate](https://crates.io/crates/zlib-rs/0.6.2) |

| zmij 1.0.20 | MIT | [LICENSE-MIT.txt](third_party/licenses/zmij-1.0.20/LICENSE-MIT.txt) | [published crate](https://crates.io/crates/zmij/1.0.20) |

| zopfli 0.8.3 | Apache-2.0 | [COPYING.txt](third_party/licenses/zopfli-0.8.3/COPYING.txt) | [published crate](https://crates.io/crates/zopfli/0.8.3) |

| zstd-safe 7.2.4 | MIT OR Apache-2.0 | [LICENSE.txt](third_party/licenses/zstd-safe-7.2.4/LICENSE.txt), [LICENSE.Apache-2.0.txt](third_party/licenses/zstd-safe-7.2.4/LICENSE.Apache-2.0.txt), [LICENSE.Mit.txt](third_party/licenses/zstd-safe-7.2.4/LICENSE.Mit.txt) | [published crate](https://crates.io/crates/zstd-safe/7.2.4) |

| zstd-sys 2.0.16+zstd.1.5.7 | MIT/Apache-2.0 | [LICENSE.txt](third_party/licenses/zstd-sys-2.0.16+zstd.1.5.7/LICENSE.txt), [LICENSE.Apache-2.0.txt](third_party/licenses/zstd-sys-2.0.16+zstd.1.5.7/LICENSE.Apache-2.0.txt), [LICENSE.BSD-3-Clause.txt](third_party/licenses/zstd-sys-2.0.16+zstd.1.5.7/LICENSE.BSD-3-Clause.txt), [LICENSE.Mit.txt](third_party/licenses/zstd-sys-2.0.16+zstd.1.5.7/LICENSE.Mit.txt), [COPYING.txt](third_party/licenses/zstd-sys-2.0.16+zstd.1.5.7/zstd/COPYING.txt), [LICENSE.txt](third_party/licenses/zstd-sys-2.0.16+zstd.1.5.7/zstd/LICENSE.txt) | [published crate](https://crates.io/crates/zstd-sys/2.0.16+zstd.1.5.7) |

| zstd 0.13.3 | MIT | [LICENSE.txt](third_party/licenses/zstd-0.13.3/LICENSE.txt) | [published crate](https://crates.io/crates/zstd/0.13.3) |
