# Docker ベースイメージのダイジェスト固定

Status: `archive`

## SOT

対象は `feat/scorecard-docker-pins`。ベースは `main` の `6546be8`。

## What

- Dockerfile の Rust ビルダーと Debian ランタイムを、Docker Hub 公式イメージのマルチアーキテクチャのダイジェストに固定した。
- Rust はワークスペースの最低対応版に合わせて 1.83 から 1.92.0 に変更した。クリーンな checkout に存在せず、ビルドでも使わない `veil.toml` のコピーを外した。Debian は `bookworm-slim` を維持した。

## Verification

- クリーンなコンテキストから `docker build --pull --platform linux/arm64` が成功。コンテナの `veil --version` は `veil 0.22.1` を返し、`scan --help` も成功した。
- レジストリで両イメージに `linux/arm64` と `linux/amd64` が含まれることを確認した。amd64 の実ビルドは未実施。
- OpenSSF Scorecard v5.6.0 のローカル比較で Pinned-Dependencies は 8 から 10 に上がり、ベースイメージ2件とも固定済みと判定された。

## Evidence

- [Dockerfile](../../Dockerfile)
- [Docker のイメージダイジェスト仕様](https://docs.docker.com/dhi/explore/security-concepts/digests)

## Rollback

Dockerfile の変更とこの記録を PR 単位で戻す。
