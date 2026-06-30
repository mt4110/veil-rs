# Repo Hygiene Notes

## 必須ignore

`.gitignore` に以下を追加する。

```gitignore
.codex/
.design/
.private/
```

- `.codex/`: Codex/agent local state。AGENTS指示に従いgit管理しない。
- `.design/`: ローカル設計SOT・実験ログ。共有する場合はreviewbundleに含める。
- `.private/`: 営業/PoC資料。公開リポジトリに入れない。

差分案は `implementation/repo_hygiene.patch` を参照。
