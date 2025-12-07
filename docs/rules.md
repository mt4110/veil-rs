# veil-rs 検出ルール一覧 (Built-in Rules) v0.1.0

veil-rs に標準搭載されている検出ルールの一覧です。
これらのルールは `veil.toml` で無効化したり、Severity を上書きしたりできます。

## 1. Auth & Tokens (Credentials)
不正流出すると即座にリスクとなる認証情報。

| ID                             | Description                   | Severity | Score |
| :----------------------------- | :---------------------------- | :------- | :---- |
| `jwt_token`                    | JSON Web Token                | High     | 80    |
| `github_personal_access_token` | GitHub Personal Access Token  | High     | 80    |
| `github_app_private_key`       | GitHub App Private Key        | Critical | 100   |
| `github_webhook_secret`        | GitHub Webhook Secret         | High     | 80    |
| `discord_bot_token`            | Discord Bot Token             | High     | 80    |
| `discord_webhook_url`          | Discord Webhook URL           | High     | 80    |
| `line_channel_access_token`    | LINE Channel Access Token     | High     | 80    |
| `slack_webhook_url`            | Slack Webhook URL             | High     | 80    |
| `sentry_dsn`                   | Sentry DSN                    | Medium   | 50    |
| `sendgrid_api_key`             | SendGrid API Key              | High     | 80    |
| `env_suspicious_secret`        | Suspicious Secret in Env      | High     | 80    |
| `generic_api_key`              | Generic API Key (Stripe-like) | High     | 80    |

## 2. Japan PII (個人情報)
日本国内の個人情報保護法やガイドラインに関連する重要情報。

| ID                           | Description                    | Severity | Score | Validator    |
| :--------------------------- | :----------------------------- | :------- | :---- | :----------- |
| `jp_my_number`               | 日本のマイナンバー（12桁）     | High     | 80    | あり (Mod11) |
| `jp_driver_license_number`   | 運転免許証番号                 | High     | 80    | なし         |
| `jp_passport_number`         | パスポート番号                 | High     | 80    | なし         |
| `jp_health_insurance_number` | 被保険者番号                   | High     | 80    | なし         |
| `jp_bank_account`            | 銀行口座番号 (支店-口座)       | Medium   | 50    | なし         |
| `jp_phone`                   | 日本の電話番号                 | Medium   | 50    | なし         |
| `jp_address`                 | 日本の住所 (都道府県+市区町村) | Medium   | 50    | なし         |

## 3. Infra & Network
インフラ構成ミスや攻撃の手がかりとなる情報。

| ID                        | Description                         | Severity | Score |
| :------------------------ | :---------------------------------- | :------- | :---- |
| `ipv4_private`            | Private IPv4 Address                | Low      | 30    |
| `wireguard_config`        | WireGuard Config PrivateKey         | High     | 80    |
| `openvpn_config`          | OpenVPN Static Key                  | High     | 80    |
| `wifi_psk_config`         | WPA Supplicant PSK                  | High     | 80    |
| `router_admin_credential` | Router Admin Credential (Heuristic) | Medium   | 50    |

## 4. Development & Cloud (Leak Prevention)
クラウド設定や開発環境に関する秘匿情報。

| ID                        | Description                    | Severity | Score |
| :------------------------ | :----------------------------- | :------- | :---- |
| `aws_access_key_id`       | AWS Access Key ID              | Critical | 100   |
| `aws_secret_access_key`   | AWS Secret Access Key          | Critical | 100   |
| `gcp_service_account_key` | GCP Service Account Key (JSON) | Critical | 100   |
| `azure_connection_string` | Azure Connection String        | High     | 80    |
| `db_connection_string`    | Database Connection String     | High     | 80    |
| `k8s_secret_manifest`     | Kubernetes Secret Manifest     | High     | 80    |
| `docker_registry_auth`    | Docker Config Auth             | High     | 80    |
| `firebase_config_secret`  | Firebase Config Private Key    | High     | 80    |
| `mobile_keystore`         | Mobile Keystore Header         | High     | 80    |

## 5. Document & Data
内部文書やデータの流出リスク。

| ID                     | Description                 | Severity | Score |
| :--------------------- | :-------------------------- | :------- | :---- |
| `db_dump_file`         | SQL Dump (Users Table)      | Medium   | 50    |
| `csv_with_pii_headers` | CSV Header with PII columns | Medium   | 50    |
