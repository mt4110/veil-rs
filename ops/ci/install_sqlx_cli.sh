#!/usr/bin/env bash
set -euo pipefail

mkdir -p .local/ci
LOG=".local/ci/sqlx_cli_install.log"
: > "$LOG"

# 以降のstdout/stderrを全部ログへ（CIで原因が即見える）
exec > >(tee -a "$LOG") 2>&1

echo "== sqlx-cli install =="
date -u +"timestamp_utc=%Y%m%dT%H%M%SZ"
echo "pwd=$(pwd)"
echo "cargo=$(cargo --version)"
echo "rustc=$(rustc --version)"

# Version logic: env override > pinned default
# Cargo.lock is unreliable for CLI versioning, so we default to explicit pin.
DEFAULT_VERSION="0.8.6"
TARGET_VERSION="${SQLX_CLI_VERSION:-$DEFAULT_VERSION}"
echo "target_version=${TARGET_VERSION}"

# 既に入ってるならバージョン確認
if command -v sqlx >/dev/null 2>&1; then
  # Try checking 'cargo sqlx --version' first as it's the most reliable
  if ! CURRENT_VERSION=$(cargo sqlx --version 2>/dev/null | awk '{print $2}'); then
     # Fallback to just checking the binary version if cargo subcommand fails (unlikely if installed via cargo)
     CURRENT_VERSION=$(sqlx --version | awk '{print $2}')
  fi
  echo "existing_sqlx_cli=${CURRENT_VERSION}"

  if [[ "${CURRENT_VERSION}" == "${TARGET_VERSION}" ]]; then
    echo "Version match. Skipping install."
    exit 0
  fi
  echo "Version mismatch or force reinstall needed."
fi

echo "Installing sqlx-cli v${TARGET_VERSION}..."

INSTALL_CMD=(
  cargo install sqlx-cli
  --locked
  --version "${TARGET_VERSION}"
  --no-default-features
  --features postgres,rustls
)

MAX=3
SLEEP=10

for i in $(seq 1 "${MAX}"); do
  echo "--- attempt ${i}/${MAX} ---"
  if "${INSTALL_CMD[@]}"; then
    echo "Install successful."
    break
  fi
  code=$?
  echo "exit_code=${code}"
  if [[ "${i}" -eq "${MAX}" ]]; then
    echo "install_failed=1"
    exit "${code}"
  fi
  echo "sleep_seconds=${SLEEP}"
  sleep "${SLEEP}"
  SLEEP=$((SLEEP * 2))
done

INSTALLED_VERSION=$(cargo sqlx --version)
echo "installed_sqlx_cli=${INSTALLED_VERSION}"
echo "OK"
