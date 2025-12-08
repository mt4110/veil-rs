#!/bin/bash
# scripts/doc.sh - Generate updated rules documentation
set -e

echo "Generating docs/rules.md from built-in rules..."
cargo run --example gen_docs --quiet > docs/rules.md

echo "Done. Please verify docs/rules.md changes."
