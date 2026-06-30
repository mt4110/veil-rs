#!/usr/bin/env bash
set -euo pipefail

cargo test --workspace
npm --prefix crates/veil-pro/frontend run build
cargo run -p veil-pro --bin export_local_api_schema -- --out-dir schemas
python scripts/check_generated_schemas.py
cargo run -p veil-cli -- verify tests/fixtures/evidence/golden.zip --require-complete
