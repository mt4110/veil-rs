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

# Cargo.lock から sqlx の version を拾う（最初に出てくる sqlx を採用）
SQLX_VERSION="$(
  awk '
    $0=="[[package]]"{in_pkg=1; name=""; ver=""; next}
    in_pkg && $1=="name" && $3=="\"sqlx\"" {name="sqlx"}
    in_pkg && $1=="version" {gsub(/"/,"",$3); ver=$3}
    in_pkg && name=="sqlx" && ver!="" {print ver; exit}
    $0==""{in_pkg=0}
  ' Cargo.lock 2>/dev/null || true
)"

if [[ -n "${SQLX_VERSION}" ]]; then
  echo "sqlx_version_from_lock=${SQLX_VERSION}"
else
  echo "sqlx_version_from_lock=NOT_FOUND"
fi

# 既に入ってるなら情報を出す（キャッシュ確認にもなる）
if command -v sqlx >/dev/null 2>&1; then
  echo "existing_sqlx_cli=$(sqlx --version || true)"
fi

# install コマンド（version拾えたら pin）
if [[ -n "${SQLX_VERSION}" ]]; then
  INSTALL_CMD=(cargo install sqlx-cli --locked --version "${SQLX_VERSION}")
else
  INSTALL_CMD=(cargo install sqlx-cli --locked)
fi

MAX=5
SLEEP=5

for i in $(seq 1 "${MAX}"); do
  echo "--- attempt ${i}/${MAX} ---"
  if "${INSTALL_CMD[@]}"; then
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

echo "installed_sqlx_cli=$(sqlx --version || true)"
echo "OK"
