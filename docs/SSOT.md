# Veil v2 SSOT

Status: `active`
Owner: product + engineering
Purpose: Veil v2 の製品定義、用語、境界、一次参照を 1 本に固定する

---

## 1. 製品定義

Veil v2 は **Audit Trail OS** である。

主語は次の 4 つに固定する。

- `Detect`: findings を検知する
- `Prove`: evidence を生成する
- `Decide`: evidence と Case Ledger から verdict を出す
- `Govern`: Case Ledger で承認・期限・責任・PatchGate 接続状態を運用する

---

## 2. 一次導線

公開面の一次導線は以下。

1. `veil scan`
2. `veil verify`
3. `veil guardian`
4. `veil fix`
5. `veil-pro`

`Detect` / `Prove` / `Decide` / `Govern` は製品概念の正本であり、現時点では standalone CLI command 名を意味しない。
`veil exceptions` は **legacy/ops compat** とする。

---

## 3. Source Of Truth

### Governance

`Case Ledger` が唯一の source of truth。

- file: `.veil/case_ledger.json`
- owner / reason / expires_at / approved_by を持つ
- `Accept Risk` は case 属性で管理する
- UI もこの ledger を読む

### Baseline

Baseline は historical debt suppression であり、ガバナンス判断ではない。

- canonical path: `veil.baseline.json`
- compat path: `.veil-baseline.json`
- 用途: 既存検知の抑制
- 非用途: 承認、例外、PatchGate 連携

### Exceptions

`ops/exceptions.toml` は legacy/ops reference。

- `prverify` など旧運用では使う
- v2 governance の一次ソースではない

---

## 4. PatchGate 境界

Veil:

- Detect
- Prove
- Decide
- Govern
- remediation task export
- patch result import

PatchGate:

- Patch
- Gate
- Merge
- GitHub publish
- waiver

Veil は patch を生成しない。
Veil は PR gate を実装しない。
Veil は waiver を持ち込まない。

---

## 5. Docs Hierarchy

文書の読み順は次で固定する。

1. この文書
2. [README.md](../README.md)
3. [docs/README.md](README.md)
4. reference docs
5. archive docs

過去の `SSOT`、`STATUS`、`PR SOT`、`dogfood`、`ops phase` 文書は historical/reference として読む。
この文書と衝突した場合は **この文書を正** とする。

---

## 6. 人手最終チェック

人間の最終レビューはローカル private checklist で行う。private 文書は public SSOT から直接リンクしない。
