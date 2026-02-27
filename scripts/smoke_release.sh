#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Find the veil binary (either built locally via cargo run, or in PATH)
if [ -f "target/release/veil" ]; then
  VEIL="target/release/veil"
  echo "Using local release build: $VEIL"
else
  VEIL="veil"
  echo "Using veil from PATH"
fi

echo "==> Running veil --version"
$VEIL --version

echo "==> Running veil doctor"
$VEIL doctor

echo "==> Running veil scan . --format json > /tmp/veil.json"
$VEIL scan . --format json > /tmp/veil.json
if ! grep -q 'schemaVersion' /tmp/veil.json; then
  echo "❌ JSON output failed validation"
  exit 1
fi

echo "==> Running veil scan . --format html > /tmp/veil.html"
$VEIL scan . --format html > /tmp/veil.html
if ! grep -q '<html' /tmp/veil.html; then
  echo "❌ HTML output failed validation"
  exit 1
fi

echo "==> Running with strict limits to verify exit code 2 and stdout purity"
set +e
$VEIL scan . --max-file-count 1 --format json > /tmp/veil_limit.json 2> /tmp/veil_limit.err
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -ne 2 ]; then
    echo "❌ Expected exit code 2, got $EXIT_CODE"
    exit 1
fi
if ! grep -q 'schemaVersion' /tmp/veil_limit.json; then
  echo "❌ JSON output broken on limit failure (stdout contaminated)"
  exit 1
fi

echo "==> Smoke test passed! 🎉"
