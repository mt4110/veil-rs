#!/usr/bin/env bash
set -euo pipefail

target="$(rustc --print host-tuple)"
target_dir="$WORK/veil-fuzz"
cargo fetch --manifest-path fuzz/Cargo.toml --locked
CARGO_NET_OFFLINE=true cargo fuzz build --release --debug-assertions \
  --sanitizer address --target "$target" --target-dir "$target_dir" rulepack_toml

install -m 0755 "$target_dir/$target/release/rulepack_toml" "$OUT/rulepack_toml"
python3 -m zipfile -c "$OUT/rulepack_toml_seed_corpus.zip" fuzz/seeds/rulepack_toml/*
