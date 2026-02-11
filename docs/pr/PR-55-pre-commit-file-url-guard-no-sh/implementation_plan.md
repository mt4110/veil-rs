# PR55 Implementation Plan — pre-commit md file-url guard (no .sh)

## 0. Summary

PR55 は staged Markdown に raw `file:` immediately followed by `//`（= `PAT='file:'"//"`）を持ち込ませない
**ローカル pre-commit ガード**を、**`.sh` ファイルを追加せず**に着地させる。

* entry point: `.githooks/pre-commit`（拡張子なし維持）
* worker: `ops/cleanFormatter.sh` を廃止し、拡張子なし `ops/cleanFormatter` として提供
* scope: staged `.md` のみ
* behavior: fail-fast + offending lines（行番号付き表示）

## 1. Constraints (Hard)

* 新規 `*.sh` を追加しない（CI: No Shell Scripts を守る）
* 自動修正はしない（決定論/意味保全のため）
* `.md` 以外は触らない（PR55の範囲外）

## 2. Desired UX

* commit の瞬間に止まる
* 何が悪いかが **1スクロールで分かる**

  * ファイル名
  * 行番号
  * 該当行（raw `file:` immediately followed by `//` を含む行）

## 3. Functional Spec

* Input: `git diff --cached --name-only -- '*.md'`
* Detection: `PAT='file:'"//"; grep -n "$PAT" <file>`
* Output: offending lines + exit 1, clean => exit 0
* Fail-fast: 最初の offending file で停止（そのファイル内は全行表示）

## 4. Implementation Outline

* rename: `git mv ops/cleanFormatter.sh ops/cleanFormatter`
* pre-commit update: `.githooks/pre-commit` → `ops/cleanFormatter`
* chmod: `chmod +x ops/cleanFormatter .githooks/pre-commit`

## 5. Test Plan

* Negative: temp md を stage → hook直叩きで block
* Positive: クリーン md で pass
* Policy: `git ls-files '*.sh' || true`

## 6. Verification Contract

* `cargo test --workspace`
* `nix run .#prverify` PASS
* evidence: `docs/evidence/prverify/prverify_<UTC>_<sha7>.md`
  