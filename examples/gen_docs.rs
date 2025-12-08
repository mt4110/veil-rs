use std::collections::HashMap;
use veil_core::model::Rule;
use veil_core::rules::builtin::get_default_rules;

fn main() {
    let rules = get_default_rules();

    // Group by category (or just sectioning logic)
    // Currently builtin rules don't have distinct categories in code ("uncategorized"),
    // but we can infer or simpler: just list them all or group by ID prefix?
    // Let's grouping manually or just list them.
    // The existing rules.md has sections:
    // 1. Auth & Tokens
    // 2. Japan PII
    // 3. Infra & Network
    // 4. Development & Cloud
    // 5. Document & Data

    // To replicate this, we need mapping logic.
    // Ideally Rule struct should have meaningful category.
    // But since it is "uncategorized", we can map by ID prefix or known IDs.

    let mut categorized: HashMap<&str, Vec<&Rule>> = HashMap::new();

    for rule in &rules {
        let cat = match rule.id.as_str() {
            id if id.starts_with("jp_") => "2. Japan PII (個人情報)",
            "jwt_token"
            | "github_personal_access_token"
            | "github_app_private_key"
            | "github_webhook_secret"
            | "discord_bot_token"
            | "discord_webhook_url"
            | "line_channel_access_token"
            | "slack_webhook_url"
            | "sentry_dsn"
            | "sendgrid_api_key"
            | "env_suspicious_secret"
            | "generic_api_key" => "1. Auth & Tokens (Credentials)",
            "ipv4_private"
            | "wireguard_config"
            | "openvpn_config"
            | "wifi_psk_config"
            | "router_admin_credential" => "3. Infra & Network",
            "aws_access_key_id"
            | "aws_secret_access_key"
            | "gcp_service_account_key"
            | "azure_connection_string"
            | "db_connection_string"
            | "k8s_secret_manifest"
            | "docker_registry_auth"
            | "firebase_config_secret"
            | "mobile_keystore" => "4. Development & Cloud (Leak Prevention)",
            "db_dump_file" | "csv_with_pii_headers" => "5. Document & Data",
            _ => "99. Other",
        };
        categorized.entry(cat).or_default().push(rule);
    }

    println!("# veil-rs 検出ルール一覧 (Built-in Rules)\n");
    println!("veil-rs に標準搭載されている検出ルールの一覧です。");
    println!("これらのルールは `veil.toml` で無効化したり、Severity を上書きしたりできます。\n");

    let mut keys: Vec<_> = categorized.keys().cloned().collect();
    keys.sort();

    for key in keys {
        println!("## {}\n", key);
        println!("| ID | Description | Severity | Score |");
        println!("| :--- | :--- | :--- | :--- |");

        let mut rules = categorized[key].clone();
        // Sort by Score desc, then ID
        rules.sort_by(|a, b| b.score.cmp(&a.score).then(a.id.cmp(&b.id)));

        for rule in rules {
            println!(
                "| `{}` | {} | {} | {} |",
                rule.id, rule.description, rule.severity, rule.score
            );
        }
        println!();
    }
}
