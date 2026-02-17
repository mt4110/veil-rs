#!/usr/bin/env bash
set -euo pipefail

# Shim for the new Go implementation (S11-03)
# Translates legacy environment variables to the new Go tool.

MODE="${MODE:-wip}"
OUT_DIR="${OUT_DIR:-.local/review-bundles}"

# Ensure we use Go 1.24
export GOTOOLCHAIN=local

echo "Shimming to reviewbundle (Go)..."
exec go run ./cmd/reviewbundle create --mode="$MODE" --out-dir="$OUT_DIR"
