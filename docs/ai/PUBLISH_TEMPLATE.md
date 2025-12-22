# Publish Template (PR / Release / X)

このテンプレは「コピーして埋めるだけ」で公開物のブレを消すための型。

---

## PR

### Title
vX.Y.Z — <短い要約>

### Body (GitHub PR)
#### What
- <何をしたかを箇条書き>
- <箇条書き>

#### Why
- <なぜ必要か（運用/安全/UX）>

#### How
- <どうやって（設計ポイント）>

#### Safety / Compatibility
- [ ] Breaking: <Yes/No>（Yesなら移行手順を書く）
- [ ] Defaults changed: <Yes/No>
- [ ] Data / Cache schema: <changed? どう変わる?>

#### Tests
- [ ] cargo test --workspace
- [ ] (optional) cargo clippy --workspace --all-targets
- [ ] (optional) CI green

#### Notes for reviewers
- <見てほしいポイント1>
- <見てほしいポイント2>

---

## Release (GitHub Releases)

### Title
vX.Y.Z — <短い要約>

### Highlights
- <1行ハイライト>
- <1行ハイライト>
- <1行ハイライト>

### Details
#### ✅ Added
- <追加点>

#### 🔧 Changed
- <変更点>

#### 🛡️ Fixed
- <修正点>

#### 🧪 Tests
- <どこでどう確認したか>

#### ⚠️ Breaking / Migration
- <必要なら手順を短く>

---

## X (Tweet)

<最初の1行: 何が嬉しいか>
vX.Y.Z: <短い要約>

✅ <箇条書き1>
✅ <箇条書き2>
✅ <箇条書き3>

Repo: <GitHub URL>
Release: <Release URL>
#Rust #OSS #Security
