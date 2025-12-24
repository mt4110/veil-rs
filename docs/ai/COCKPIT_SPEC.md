# COCKPIT_SPEC_CANONICAL (v1.2)

**Status**: Canonical / Active  
**Objective**: **“Zero Hesitation”** — CI と Local の挙動差を根絶する。  
**Scope**: `dist` 生成・検査・状態確認（リリース本文の“生成コックピット”）。  
**Engine**: **Go (`veil-aiux`)**  
**Parity Layer**: **Nix (`nix run .#gen|.#check|.#status`)**

---

## 0. この聖典の読み方（脱線防止）
- ここに書いてある “契約” が最優先。  
- 迷ったら **止まる（Exit > 0）**。勝手に直さない。  
- 仕様追加は「ここに追記 → 実装 → テスト → CI」の順番。逆は禁止。

---

## 1. Iron Laws（絶対に破ってはいけない）
違反は **Red Lamp**（即やり直し）。

1) **Single Entry Point**  
   生成・検査ロジックは **`veil-aiux` にのみ存在**する。  
   - CI / Local / Wrapper にロジックを複製しない。

2) **No Auto-Repair（自動修復しない）**  
   `veil-aiux` は **変更を加えない**。  
   - 出力は「Reason / Fix」を必ず出す。  
   - 例外なし（“気が利く修正” は将来の地雷）。

3) **Dist Contract（出力契約）**  
   出力先は常に `dist/publish/<VERSION>/`。  
   その中は **必ず 4ファイルだけ**:
   - `PUBLISH_<VER>.md`
   - `RELEASE_BODY_<VER>.md`
   - `X_<VER>.md`
   - `AI_PACK_<VER>.txt`（**.txt 固定**）

4) **AI_PACK Safety（漏洩防止）**
   - Local では `.txt` 生成のみ許可。
   - `dist/publish/**/AI_PACK*.md` が存在したら **check は必ず FAIL**。
   - CI の artifact upload は **md のみ**（`**/*.md`）。AI_PACK は絶対にアップロードしない。

5) **Exit Codes（標準）**
   - `0`: OK
   - `2`: USAGE（引数不正 / 安全装置発動）
   - `3`: CHECK（契約違反 / ガードレール違反）
   - `4`: GEN（生成実行失敗）
   - `5`: IO（ファイルアクセス/権限/パス問題）

6) **Bash-Free Core**
   - `scripts/ai/*.sh` は **Wrapperのみ**。  
   - 目安: **最大 10行**。条件分岐・解析・検査ロジック禁止。  
   - できることは `exec nix run ... -- "$@"` だけ。  
   - Wrapper は **1リリースサイクル後に削除**。

---

## 2. CLI Spec（`veil-aiux`）
**Location**: `tools/veil-aiux`（Go module）  
**Binary**: `veil-aiux`

### 2.1 `gen`
```bash
veil-aiux gen --version v0.14.0 [--clean] [--base-ref origin/main]
```
- `--version` 必須（`vX.Y.Z` のみ許可）
- `--clean` は明示時のみ削除（黙って上書きしない）
- `--base-ref` は Git diff の基準（default: `origin/main`）

**禁則**: `--out` を追加しない。どうしても必要なら「契約パス以外は即 E_USAGE」。

**生成の流れ（必須）**
1. 引数検証（fail closed）
2. `dist/publish/` を用意
3. `--clean` が指定された時だけ `dist/publish/<VER>/` を削除
4. `dist/publish/.tmp-<ver>-<pid>/` に 4ファイルを作る（途中は契約外）
5. tmp 内で「4ファイルちょうど」を検証（漏洩チェック含む）
6. `rename(tmp -> dist/publish/<VER>)`（原子的な確定）

### 2.2 `check`
```bash
veil-aiux check [--version v0.14.0]
```
- `--version` なし: リポ内ルール（テンプレ存在・H1数・Fence数など）
- `--version` あり: 上記 + `dist/publish/<VER>` の契約検証

**出力**: 失敗時は必ず
- `ERROR[code]`
- `Reason: ...`
- `Fix: ...`

### 2.3 `status`
```bash
veil-aiux status --version v0.14.0
```
- Read-only
- 4ファイルの存在/サイズ
- AI_PACK の統計（可能なら diff stats）

---

## 3. Nix Parity（CI/Local 完全一致）
**絶対条件**: `nix run .#app` が CI/Local で同一挙動。

- Platforms: `x86_64-linux`（CI）, `aarch64-darwin`（Local）
- Runtime deps: `git`（AI_PACK 生成に必要）
- CI は **fetch-depth: 0**（`origin/main` diff を成立させるため）

Apps:
- `.#gen` -> `veil-aiux gen`
- `.#check` -> `veil-aiux check`
- `.#status` -> `veil-aiux status`

---

## 4. Drift Guard（“脱線したらメス入れる” ルール）
次のどれかが起きたら **即停止して修正**（Red Lamp候補）。

- Wrapper に条件分岐や検査ロジックが増えた
- CI と Local で別コマンドを実行している
- dist の 4ファイル契約が増減した
- AI_PACK が `.md` で生成されるルートが生えた
- spec と実装がズレたのに “まあいいか” で放置した

---

## 5. Acceptance（合格条件）
### Local
```bash
nix run .#gen -- --version v0.14.0 --clean
nix run .#check
nix run .#check -- --version v0.14.0
nix run .#status -- --version v0.14.0
```

### Leak Attempt
```bash
touch dist/publish/v0.14.0/AI_PACK_v0.14.0.md
nix run .#check -- --version v0.14.0
# => Exit 3（CHECK）になること
```
