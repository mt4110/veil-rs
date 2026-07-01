# Risk Register

| ID | リスク | 影響 | 対策 |
|---|---|---|---|
| R1 | JP氏名/住所の誤検知 | UX低下 | ラベル/文脈必須、presetで制御 |
| R2 | マイナンバー裸12桁の誤検知 | CIノイズ | Medium以下、context加点、negative fixture |
| R3 | LSPが重い | エディタUX低下 | debounce、サイズ上限、document scan限定 |
| R4 | UIにraw secret漏洩 | B2B致命傷 | SafeFindingのみ、API schema test |
| R5 | Evidenceが改ざん不可と誤表現 | 法務リスク | tamper-evident表現へ統一 |
| R6 | PDF実装が重い | リリース遅延 | HTML+print CSSを先行 |
| R7 | presetが多すぎる | UX迷子 | v1は5種固定。`minimal-ci`はpresetではなくCI modeで表現 |
| R8 | CI遅延 | 導入失敗 | staged scan、default ignore、limits |
| R9 | Config layering不透明 | B2B導入不安 | policy explain UI |
| R10 | SOT版ブレ | GTM事故 | check script / strict bundle |

| R11 | schema生成先/検証スクリプト名の分裂 | PR-0停止 | `schemas/` と `scripts/check_generated_schemas.py` に固定 |
| R12 | 一括実装時の層間rollback不能 | リリース遅延 | `14_bulk_implementation_safety.md` のDAG/rollback条件に従う |
