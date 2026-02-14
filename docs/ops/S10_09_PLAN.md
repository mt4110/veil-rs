Goal

prkit の 外部コマンド実行を長期運用で破綻しない形に硬化

shell禁止（sh -c / bash -c / 文字列コマンド禁止）

cmd/args/cwd/env/stdout/stderr/exit を portable evidence に決定論で記録

失敗時の evidence も 決定論（順序・改行・JSONキー・フィールド有無）

テストは 実プロセス実行禁止（fake/stubで determinism 担保）

Non-goals

hardening以外の機能増（やるなら別Sへ）

ネットワーク依存の統合テスト

実プロセスをspawnするテスト（flake温床）

Definitions / Invariants（壊れないための契約）

Invariant-1: shellを使わない

exec.Command(name, args...) / CommandContext のみ

文字列1本のコマンド（クォート含む）を受け取らない

Invariant-2: evidenceに 非決定性を入れない

時刻・絶対パス・ホスト名・ユーザ名・環境差（巨大env）を混入させない

Invariant-3: 出力正規化は 入口で固定

改行：\r\n/\r → \n

サイズ上限：MAX_BYTES（必要なら）＋ truncated を明示

バイト→文字列は UTF-8で確定（不正byteは \uFFFD で確定）

Invariant-4: JSON determinism

mapを evidence に持ち込まない（順序問題を殺す）

env は [{k,v}] の ソート済みsliceで持つ

command_list は append順が契約（テストで固定）

PHASE 0: Clean rail（汚れた状態で設計しない）

IF git status --porcelain not empty:

IF 意図した変更 → commit（別コミットに切る）

ELSE → restore（git restore -SW . など）
RECHECK
IF still dirty → error STOP

PHASE 1: 対象箇所の実パス確定（作り話禁止・ここが心臓）

目的：「編集対象ファイルの絶対リスト（repo相対パス）」を確定し、以降の設計・実装・テストの参照先を固定する。

1-A) prkitの実体（ディレクトリ）を確定

CANDIDATES = rg -n "prkit" -S cmd internal scripts .github の結果から、

“prkit本体っぽい”ディレクトリを列挙

IF candidates == 0 → error STOP（このSは成立しない）
IF candidates > 1:

判定規則（上から優先）

cmd/prkit/ が存在 →採用

internal/prkit/ が存在 →採用

package prkit の定義が最も集中しているdir →採用
IF still ambiguous → error STOP（確定できないまま進むの禁止）

LOCK: PRKIT_DIR=cmd/prkit を PLAN/TASKに貼る

1-B) exec利用点の実パス確定（全列挙してロック）

SEARCH_KEYS = {os/exec, exec.Command, exec.CommandContext, CombinedOutput, Output, Run, command_list, RunCmd}
RESULT = rg -n "<SEARCH_KEYS>" -S <PRKIT_DIR> cmd internal

IF RESULT empty → error STOP
LOCK: 以下を固定

EXEC_CALL_SITES = [
  "internal/prkit/check_git.go",
  "internal/prkit/review_bundle.go",
  "internal/prkit/sot.go",
  "internal/prkit/tools.go"
]（重複なし、ソート済み）

EVIDENCE_SCHEMA_FILES = [
  "internal/prkit/portable_evidence.go"
]

STOP条件：

exec呼び出しが prkit外に多数散ってる（>2ディレクトリに波及）→ STOP
（まず「中央集約だけ」を終える。記録追加は次コミット/次Sへ）

PHASE 2: “実行”の中央集約（単一入口の確立）

前提：PHASE 1 の LOCK が埋まっていること（未埋めならSTOP）

2-A) インターフェースを導入（テストでspawn禁止にするため）

DEFINE:

type ExecSpec struct {

Argv []string（= [cmd, arg1, ...]。cmdとargsを分離せず“argv”で固定）

CwdRel string（repo相対 / 空なら “process cwd”）

EnvKV []EnvKV（sorted slice）

TimeoutMs int（0=無制限）

// NOTE: 時刻は持たない

type ExecResult struct {

Stdout string（normalized）

Stderr string（normalized）

ExitCode int（成功=0 / 失敗=非0 / 起動失敗=-1 など 規約化）

TruncatedStdout bool

TruncatedStderr bool

ErrorKind string（"", "exit", "timeout", "spawn", "canceled" のように 列挙で固定）

type ExecRunner interface { Run(ctx context.Context, spec ExecSpec) ExecResult }

errorを返さない（errorはResultに畳む：失敗でも決定論で記録するため）

STOP条件：

既存構造に合わせられず、広範囲の設計変更が必要 → STOP（S10-09は中央集約までに縮退）

2-B) 本番runner（shell無し）実装

exec.CommandContext(spec.Argv[0], spec.Argv[1:]...)

Cmd.Dir は repo root + CwdRel（or 空）

Cmd.Env は spec.EnvKV から構築（順序固定）

stdout/stderr は 別バッファ

ctx timeout/cancel を ErrorKind に正規化

出力正規化：

改行統一：CRLF/CR → LF

サイズ上限：MAX_BYTES（定数）で切り、TruncatedX=true

trailing newline の強制は しない（勝手に変えると意味が変わる）

PHASE 3: evidenceへ command_list を決定論で積む

前提：PHASE 1 の EVIDENCE_SCHEMA_FILES を読んで “既存schema” を把握していること

最小フィールド（例：既存に合わせる。増やしすぎ禁止）：

argv（[]string）

cwd_rel（string / "" 可）

env（[] {k,v} sorted）

stdout（string）

stderr（string）

exit_code（int）

error_kind（string）

truncated_stdout（bool）

truncated_stderr（bool）

順序契約：

command_list は 呼び出し順にappend

呼び出し順がmap/並列で揺れる場合：

先に実行順を固定（slice化＋sort）

それが広範囲なら STOP → 中央集約のみに縮退

PHASE 4: テスト（execしない / determinism固定）

fakeRunner を実装し、spec→result を固定で返す

success case: stdout/stderr/exit=0

failure case: stderr + exit!=0（ErrorKind="exit"）

timeout/cancel: ErrorKind="timeout" or "canceled"（可能なら）

contract:

同一入力（spec列 + 呼び出し順）→ 同一JSON（byte一致）

command_list の順序が揺れない

STOP条件：

テストが実プロセスspawnを要求してくる → STOP（テスト設計からやり直し）

PHASE 5: Gates（証拠を残して閉じる）

go test ./... -count=1 PASS

nix run .#prverify PASS

docs/evidence/prverify/prverify_<UTC>_<sha>.md 保存

SOTとTASKの該当行を更新
