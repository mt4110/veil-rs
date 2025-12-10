# Benchmarks v0.7.0

## Overview
Performance baseline for `veil-rs` v0.7.0.
Focus on scanning throughput and resilience against large files/binaries.

## Environment
- Machine: Local Dev
- Check logic: Regex-based pattern matching (re2/rust-regex).

## Results

| Benchmark | Latency | Throughput | Notes |
| Str                     | Str | Str | Str                  |
| ----------------------- | --- | --- | -------------------- |
| `scan_content_small`    |     |     | Simple config file   |
| `scan_content_medium`   |     |     | 1MB text file        |
| `scan_content_large`    |     |     | 5MB text file        |
| `scan_data_binary_skip` |     |     | 5MB binary (skipped) |
| `scan_data_size_skip`   |     |     | 10MB file (skipped)  |

## Observations
- Binary detection is efficient (only first 8KB read).
- Large files larger than `max_file_size` are skipped almost instantly.
- Content scanning scales linearly with file size.
