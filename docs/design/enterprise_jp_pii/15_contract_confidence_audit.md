# 15. Contract Confidence Audit v4.4

## 目的

本章は「100% confidentか」という問いに対する自己監査である。v4.4時点の設計は、既知の契約矛盾を実装前にSOTとacceptance gateへ落とすことを目的とする。

## 結論

v4.4時点で、設計契約としての既知の実装停止要因は Contract SOT に落とし込んだ。実装の正しさは `PR-0 Contract Alignment` の acceptance gate で実証する。したがって「100%保証」ではなく、**既知の契約矛盾を機械検証可能なゲートに落とした状態**を完了条件とする。

## 既知loopholeと処置

| loophole | 処置 |
|---|---|
| schema検証script名の分裂 | `scripts/check_generated_schemas.py` に固定 |
| schema出力先の分裂 | repo root `schemas/` に固定 |
| JSON Schema `$ref` 構文だけOKで未解決refを見逃す | PR-0で resolver-backed schema validation を必須化 |
| `report.json` `$ref` 不安定 | `$defs.SafeFindingApiV1` 埋め込み |
| `SeverityCounts` key省略 | `Low` / `Medium` / `High` / `Critical` 必須 |
| `suppressedSeverityCounts` 任意/必須ブレ | 全surfaceで必須 + zero-filled |
| `coverageComplete` DTO例抜け | Local API DTO / OpenAPI / schemas に必須化 |
| legacy `severity` migration未定 | Low=20, Medium=40, High=70, Critical=90 |
| `score` と `severity` 併存 | `base_score`優先。`score + severity` のみなら `score`優先 |
| oversized file skip/incomplete混同 | binaryはexpected skip、text/log/source oversizeはincomplete |
| acceptance gateの二重化 | 具体コマンドは `14.4 Acceptance Gate` のみを正本化 |
| acceptance gateのPATH依存 | `cargo run -p veil-cli -- verify ...` に固定 |
| Evidence report findingsの範囲不明 | raw-free all findings（new + suppressed）に固定 |
| RunMeta self hash | 内部に持たず、外部anchorのみ |
| `RunMetaResponse` subset化リスク | full `RunMetaV1` のみを返す契約に固定 |
| `RunMeta.result.limitReasons` optional化リスク | requiredに固定。理由なし時は `[]` |
| `RunMeta.result` の自由拡張 | `additionalProperties=false`; 拡張は top-level `extensions` のみ |
| `baselineFingerprint` と `findingId` の混同 | 別物として維持し、baseline照合は `baselineFingerprint` に固定 |
| stale design artifact混入 | versioned stale summaryをパックから削除し、README noteを単一化 |

## PR-0で必ず機械化すること

- Rust DTOからOpenAPI/JSON Schemaを生成する。
- 生成物とtracked schemaの差分を `scripts/check_generated_schemas.py` で検出する。
- JSON Schema resolverで未解決 `$ref` を検出する。
- OpenAPI内部 `$ref` を検出する。
- `python scripts/check_contract_acceptance.py` で `14.4 Acceptance Gate` のコマンド列を直列実行する。
- acceptance gate失敗時は `14_bulk_implementation_safety.md` のDAG層までrollbackする。

## Confidence boundary

v4.4は実装開始可能な設計契約である。ただし、事実としての100% confidenceは `PR-0 Contract Alignment` が生成schema差分ゼロとacceptance gate通過を示した後にのみ主張できる。
