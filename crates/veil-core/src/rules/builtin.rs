use crate::model::{Rule, Severity};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;
use veil_config::Config;

static DEFAULT_RULES: OnceLock<Vec<Rule>> = OnceLock::new();

pub fn get_default_rules() -> Vec<Rule> {
    DEFAULT_RULES.get_or_init(|| {
        vec![
            // ============================================
            // 1. Auth & Tokens (Kill on Sight)
            // ============================================
            Rule {
                id: "jwt_token".to_string(),
                pattern: Regex::new(r"ey[a-zA-Z0-9_-]{10,}\.ey[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}").unwrap(),
                description: "JSON Web Token".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "github_personal_access_token".to_string(),
                pattern: Regex::new(r"gh[pousr]_[a-zA-Z0-9]{36}").unwrap(),
                description: "GitHub Personal Access Token".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "github_app_private_key".to_string(),
                pattern: Regex::new(r"-----BEGIN RSA PRIVATE KEY-----").unwrap(),
                description: "GitHub App Private Key".to_string(),
                severity: Severity::Critical,
                score: 100,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "github_webhook_secret".to_string(),
                pattern: Regex::new(r"(?i)github_?webhook_?secret.?=.?['\x22]?([a-zA-Z0-9]{40})['\x22]?").unwrap(),
                description: "GitHub Webhook Secret".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "discord_bot_token".to_string(),
                pattern: Regex::new(r"([a-zA-Z0-9]{24}\.[\w-]{6}\.[\w-]{27}|mfa\.[\w-]{84})").unwrap(), 
                description: "Discord Bot Token".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "discord_webhook_url".to_string(),
                pattern: Regex::new(r"https://discord\.com/api/webhooks/\d+/[a-zA-Z0-9_-]+").unwrap(),
                description: "Discord Webhook URL".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "line_channel_access_token".to_string(),
                pattern: Regex::new(r"(?i)channel_access_token.?=.?['\x22]?[a-zA-Z0-9/+=]{43,}['\x22]?").unwrap(),
                description: "LINE Channel Access Token".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "slack_webhook_url".to_string(),
                pattern: Regex::new(r"https://hooks\.slack\.com/services/T[0-9a-zA-Z]{8}/B[0-9a-zA-Z]{8}/[0-9a-zA-Z]{24}").unwrap(),
                description: "Slack Webhook URL".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "sentry_dsn".to_string(),
                pattern: Regex::new(r"https://[a-f0-9]{32}@o\d+\.ingest\.sentry\.io/\d+").unwrap(),
                description: "Sentry DSN".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "sendgrid_api_key".to_string(),
                pattern: Regex::new(r"SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{43}").unwrap(),
                description: "SendGrid API Key".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "env_suspicious_secret".to_string(),
                pattern: Regex::new(r"(?i)(password|secret|api_key|access_key|token)[a-z0-9_]*\s*=\s*['\x22]?[a-zA-Z0-9/+]{20,}").unwrap(),
                description: "Suspicious Secret in Env".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "generic_api_key".to_string(),
                pattern: Regex::new(r"(?i)(sk_live_|sk_test_|pk_live_|pk_test_)[0-9a-zA-Z]{24,}").unwrap(),
                description: "Generic API Key (Stripe-like)".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },

            // ============================================
            // 2. Japan PII (Enhanced)
            // ============================================
            Rule {
                id: "jp_my_number".to_string(),
                pattern: Regex::new(r"\b\d{12}\b").unwrap(),
                description: "Japanese My Number (Individual Number)".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: Some(validate_my_number),
            },
            Rule {
                id: "jp_driver_license_number".to_string(),
                pattern: Regex::new(r"\b\d{12}\b").unwrap(),
                description: "Japanese Driver License Number".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },

            Rule {
                id: "jp_passport_number".to_string(),
                pattern: Regex::new(r"\b[A-Z]{2}\d{7}\b").unwrap(),
                description: "Japanese Passport Number".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "jp_health_insurance_number".to_string(),
                pattern: Regex::new(r"\b\d{4}-\d{6}\b").unwrap(), 
                description: "Japanese Health Insurance Number".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "jp_bank_account".to_string(),
                pattern: Regex::new(r"\b\d{3}-\d{7}\b").unwrap(),
                description: "Japanese Bank Account (Branch-Number)".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "jp_phone".to_string(),
                pattern: Regex::new(r"0\d{1,4}-\d{1,4}-\d{4}").unwrap(),
                description: "Japanese Phone Number".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "jp_address".to_string(),
                pattern: Regex::new(r"(?x)
                    (北海道|東京都|.{2,3}(?:府|県))
                    .{1,10}
                    (?:市|区|町|村)
                ").unwrap(),
                description: "Japanese Address (Prefecture+City)".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },

            // ============================================
            // 3. Infra & Network
            // ============================================
            Rule {
                id: "ipv4_private".to_string(),
                pattern: Regex::new(r"(10\.\d{1,3}\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3}|172\.(1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3})").unwrap(),
                description: "Private IPv4 Address".to_string(),
                severity: Severity::Low,
                score: 30,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "wireguard_config".to_string(),
                pattern: Regex::new(r"(?s)\[Interface\].*?PrivateKey\s*=\s*[a-zA-Z0-9+/=]{44}").unwrap(),
                description: "WireGuard Config PrivateKey".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "openvpn_config".to_string(),
                pattern: Regex::new(r"BEGIN OpenVPN Static key V1").unwrap(),
                description: "OpenVPN Static Key".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "wifi_psk_config".to_string(),
                pattern: Regex::new(r"(?i)psk\s*=\s*['\x22]?[a-f0-9]{64}['\x22]?").unwrap(),
                description: "WPA Supplicant PSK".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "router_admin_credential".to_string(),
                pattern: Regex::new(r"(?i)(192\.168\.[01]\.1).*?(admin|password)").unwrap(),
                description: "Router Admin Credential (Heuristic)".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },

            // ============================================
            // 4. Development & Cloud (Leak Prevention)
            // ============================================
            Rule {
                id: "aws_access_key_id".to_string(),
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                description: "AWS Access Key ID".to_string(),
                severity: Severity::Critical,
                score: 100,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "aws_secret_access_key".to_string(),
                pattern: Regex::new(r"(?i)aws_?secret_?access_?key.?=.?['\x22]?([a-zA-Z0-9/+]{40})['\x22]?").unwrap(),
                description: "AWS Secret Access Key".to_string(),
                severity: Severity::Critical,
                score: 100,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "gcp_service_account_key".to_string(),
                pattern: Regex::new(r#""type":\s*"service_account"#).unwrap(),
                description: "GCP Service Account Key (JSON)".to_string(),
                severity: Severity::Critical,
                score: 100,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "azure_connection_string".to_string(),
                pattern: Regex::new(r"(?i)DefaultEndpointsProtocol=[^;]+;AccountName=[^;]+;AccountKey=[^;]+").unwrap(),
                description: "Azure Connection String".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "db_connection_string".to_string(),
                pattern: Regex::new(r"(?i)(postgres|mysql|sqlserver|oracle|mongodb)://[^:]+:([^@]+)@").unwrap(),
                description: "Database Connection String (Password)".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "k8s_secret_manifest".to_string(),
                pattern: Regex::new(r"(?s)kind:\s*Secret.*?data:").unwrap(),
                description: "Kubernetes Secret Manifest".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "docker_registry_auth".to_string(),
                pattern: Regex::new(r#""auth":\s*"[a-zA-Z0-9+/=]+"#).unwrap(),
                description: "Docker Config Auth".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "firebase_config_secret".to_string(),
                pattern: Regex::new(r#""private_key":\s*"-----BEGIN PRIVATE KEY-----"#).unwrap(),
                description: "Firebase Config Private Key".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "mobile_keystore".to_string(),
                pattern: Regex::new(r"MIvk...").unwrap(),
                description: "Mobile Keystore Header (Placeholder)".to_string(),
                severity: Severity::High,
                score: 80,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },

            // ============================================
            // 5. Document & Data
            // ============================================
            Rule {
                id: "db_dump_file".to_string(),
                pattern: Regex::new(r"(?i)INSERT INTO [`']?users[`']?").unwrap(),
                description: "SQL Dump (Users Table)".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            },
            Rule {
                id: "csv_with_pii_headers".to_string(),
                pattern: Regex::new(r"(?i)^(name|email|phone|address|zip|user_id),").unwrap(),
                description: "CSV Header with PII columns".to_string(),
                severity: Severity::Medium,
                score: 50,
                category: "uncategorized".to_string(),
                tags: vec![],
                validator: None,
            }
        ]
    }).clone()
}

fn validate_my_number(s: &str) -> bool {
    if s.len() != 12 {
        return false;
    }
    let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 12 {
        return false;
    }

    // Weights: Pn for n=1..11 (from left)
    // 6 5 4 3 2 7 6 5 4 3 2
    let weights = [6, 5, 4, 3, 2, 7, 6, 5, 4, 3, 2];
    let mut sum = 0;
    for i in 0..11 {
        sum += digits[i] * weights[i];
    }

    let remainder = sum % 11;
    let check_digit = if remainder <= 1 { 0 } else { 11 - remainder };

    check_digit == digits[11]
}

pub fn get_all_rules(config: &Config) -> Vec<Rule> {
    // defaults is Vec<Rule> (cloned from static)
    let defaults = get_default_rules();
    let mut rule_map: HashMap<String, Rule> =
        defaults.into_iter().map(|r| (r.id.clone(), r)).collect();

    for (id, rule_conf) in &config.rules {
        if let Some(pattern_str) = &rule_conf.pattern {
            // New rule or Overwrite pattern (Pure TOML Rule)
            if let Ok(regex) = Regex::new(pattern_str) {
                let rule = Rule {
                    id: id.clone(),
                    pattern: regex,
                    description: rule_conf
                        .description
                        .clone()
                        .unwrap_or_else(|| "Custom rule".to_string()),
                    severity: rule_conf.severity.as_deref().unwrap_or("medium").into(),
                    score: rule_conf.score.unwrap_or(50) as u32,
                    category: rule_conf.category.clone().unwrap_or("custom".to_string()),
                    tags: rule_conf.tags.clone().unwrap_or_default(),
                    validator: None,
                };
                rule_map.insert(id.clone(), rule);
            } else {
                eprintln!("Invalid Regex pattern for rule {}: {}", id, pattern_str);
            }
        } else {
            // Override existing rule
            if let Some(rule) = rule_map.get_mut(id) {
                if let Some(sev) = &rule_conf.severity {
                    rule.severity = sev.as_str().into();
                }
                if let Some(desc) = &rule_conf.description {
                    rule.description = desc.clone();
                }
                if let Some(score) = rule_conf.score {
                    rule.score = score as u32;
                }
                if let Some(cat) = &rule_conf.category {
                    rule.category = cat.clone();
                }
                if let Some(tags) = &rule_conf.tags {
                    rule.tags = tags.clone();
                }
            }
        }
    }

    rule_map
        .into_values()
        .filter(|r| config.rules.get(&r.id).map(|rc| rc.enabled).unwrap_or(true))
        .collect()
}
