# Default Rules Reference

Total rules: 35

| ID | Severity | Description |
|----|----------|-------------|
| `jwt_token` | **HIGH** | JSON Web Token |
| `github_personal_access_token` | **HIGH** | GitHub Personal Access Token |
| `github_app_private_key` | **CRITICAL** | GitHub App Private Key |
| `github_webhook_secret` | **HIGH** | GitHub Webhook Secret |
| `discord_bot_token` | **HIGH** | Discord Bot Token |
| `discord_webhook_url` | **HIGH** | Discord Webhook URL |
| `line_channel_access_token` | **HIGH** | LINE Channel Access Token |
| `slack_webhook_url` | **HIGH** | Slack Webhook URL |
| `sentry_dsn` | **MEDIUM** | Sentry DSN |
| `sendgrid_api_key` | **HIGH** | SendGrid API Key |
| `env_suspicious_secret` | **HIGH** | Suspicious Secret in Env |
| `generic_api_key` | **HIGH** | Generic API Key (Stripe-like) |
| `jp_my_number` | **HIGH** | Japanese My Number (Individual Number) |
| `jp_driver_license_number` | **HIGH** | Japanese Driver License Number |
| `jp_passport_number` | **HIGH** | Japanese Passport Number |
| `jp_health_insurance_number` | **HIGH** | Japanese Health Insurance Number |
| `jp_bank_account` | **MEDIUM** | Japanese Bank Account (Branch-Number) |
| `jp_phone` | **MEDIUM** | Japanese Phone Number |
| `jp_address` | **MEDIUM** | Japanese Address (Prefecture+City) |
| `ipv4_private` | **LOW** | Private IPv4 Address |
| `wireguard_config` | **HIGH** | WireGuard Config PrivateKey |
| `openvpn_config` | **HIGH** | OpenVPN Static Key |
| `wifi_psk_config` | **HIGH** | WPA Supplicant PSK |
| `router_admin_credential` | **MEDIUM** | Router Admin Credential (Heuristic) |
| `aws_access_key_id` | **CRITICAL** | AWS Access Key ID |
| `aws_secret_access_key` | **CRITICAL** | AWS Secret Access Key |
| `gcp_service_account_key` | **CRITICAL** | GCP Service Account Key (JSON) |
| `azure_connection_string` | **HIGH** | Azure Connection String |
| `db_connection_string` | **HIGH** | Database Connection String (Password) |
| `k8s_secret_manifest` | **HIGH** | Kubernetes Secret Manifest |
| `docker_registry_auth` | **HIGH** | Docker Config Auth |
| `firebase_config_secret` | **HIGH** | Firebase Config Private Key |
| `mobile_keystore` | **HIGH** | Mobile Keystore Header (Placeholder) |
| `db_dump_file` | **MEDIUM** | SQL Dump (Users Table) |
| `csv_with_pii_headers` | **MEDIUM** | CSV Header with PII columns |
