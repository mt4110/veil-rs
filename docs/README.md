# Docs Portal

Status: `active`
Owner: product + engineering
Purpose: public docs の入口、分類規約、SSOT 導線

---

## 1. まず読む順番

1. [SSOT.md](SSOT.md)
2. [README.md](../README.md)
3. 必要な reference docs

この順番を崩すと、historical docs を先に読んで迷いやすい。

---

## 2. ステータス定義

- `active`: 今の製品判断に直接効く文書。仕様や導線の一次参照。
- `reference`: 現行運用や補助説明には使うが、SSOT ではない文書。
- `archive`: 歴史的記録。過去の設計・証跡・進捗・PR 単位メモ。

---

## 3. 全 docs の分類ルール

この repo の Markdown は path 単位で全件分類する。

### `active`

- [README.md](README.md) (この文書)
- [SSOT.md](SSOT.md)
- [DEVELOPMENT.md](DEVELOPMENT.md)
- [DEVELOPMENT_EN.md](DEVELOPMENT_EN.md)
- [TESTING_SECRETS.md](TESTING_SECRETS.md)

### `reference`

- `docs/baseline/**`
- `docs/ci/**`
- `docs/cli/**`
- `docs/guardian/**`
- `docs/guardrails/**`
- `docs/integrations/**`
- `docs/runbook/**`
- `docs/templates/**`
- `docs/sales/**`
- `docs/rules.md`
- `docs/rules/**`
- `docs/json-schema.md`
- `docs/prompts/**`
- `docs/security/threat_model.md`
- `docs/pr/README.md`
- `docs/pr/sot_template.md`
- `docs/ops/*CONTRACT*.md`
- `docs/ops/REVIEW_BUNDLE.md`
- `docs/ops/LOCAL_STORAGE_POLICY_v1.md`
- `docs/dogfood/README.md`

### `archive`

- `docs/ai/**`
- `docs/design/**`
- `docs/dogfood/**` except `README.md`
- `docs/evidence/**`
- `docs/epics/**`
- `docs/ops/**` except contract/reference docs
- `docs/perf/**`
- `docs/phase16/**`
- `docs/phase17/**`
- `docs/pr/PR-*.md`
- `docs/pr/PR-TBD-*.md`
- `docs/pr/*/{plan,task,implementation_plan}.md`
- `docs/pr/_archive/**`
- `docs/pr/evidence/**`
- `docs/roadmap/**`
- `docs/security/dependabot/**`
- `docs/security_review.md`
- `docs/v0.20.0-planning/**`

---

## 4. よく使う reference

- CLI: [README.md](cli/README.md)
- Baseline: [usage.md](baseline/usage.md)
- Integrations: [github-actions.md](integrations/github-actions.md)
- Runbook: [review-bundle.md](runbook/review-bundle.md)
- Product boundary: `docs/sales/PRD_V1_NONOVERLAP_PATCHGATE_JA.md`

---

## 5. 機械チェック

全 Markdown の分類は次で検証する。

```bash
python3 scripts/check_docs_taxonomy.py
```

全件の分類を一覧で見たい場合:

```bash
python3 scripts/check_docs_taxonomy.py --list
```

---

## 6. 人手確認

最終の人間レビューはローカル private checklist で行う。private 文書は public docs から直接リンクしない。
