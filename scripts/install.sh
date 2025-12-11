#!/usr/bin/env sh
set -eu

REPO_URL="https://github.com/mt4110/veil-rs.git"
CRATE_NAME="veil-cli"   # ← crate 名
BIN_NAME="veil"         # ← 実際に入るバイナリ名

echo "🔧 Installing ${BIN_NAME} from ${REPO_URL} ..."

# 1. cargo があるかチェック
if ! command -v cargo >/dev/null 2>&1; then
  echo "❌ Rust (cargo) が見つかりません。まず Rust をインストールしてください。" >&2
  echo "   https://www.rust-lang.org/tools/install" >&2
  exit 1
fi

# 2. インストール（既に入っている場合は上書きインストールされる）
#    ※ HEAD の main を入れる運用。安定版を固定したいなら --tag v0.7.5 も検討。
echo "➡ cargo install --git ${REPO_URL} ${CRATE_NAME}"
cargo install --locked --git "${REPO_URL}" "${CRATE_NAME}"

# 3. 確認
if command -v "${BIN_NAME}" >/dev/null 2>&1; then
  echo
  echo "✅ Install complete!"
  "${BIN_NAME}" --version || true
else
  echo "⚠ インストールは完了したはずですが、PATH に ${BIN_NAME} が見つかりません。" >&2
  echo "   ~/.cargo/bin が PATH に入っているか確認してください。" >&2
fi
