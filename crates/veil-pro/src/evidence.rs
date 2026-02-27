use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunMeta {
    pub schema_version: String,
    pub run_id: String,
    pub generated_at_utc: String,

    pub product: ProductMeta,
    pub engine: EngineMeta,
    pub project: ProjectMeta,
    pub invocation: InvocationMeta,
    pub limits: LimitsMeta,
    pub scope: ScopeMeta,
    pub result: ResultMeta,
    pub artifacts: ArtifactsMeta,
    pub privacy: PrivacyMeta,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductMeta {
    pub name: String,
    pub version: String,
    pub commit: String,
    pub build_profile: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EngineMeta {
    pub name: String,
    pub schema_version: String,
    pub rule_packs: Vec<RulePackMeta>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RulePackMeta {
    pub name: String,
    pub source: String,
    pub content_sha256: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub display_name: String,
    pub root_hint: String,
    pub git: ProjectGitMeta,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectGitMeta {
    pub head: String,
    pub dirty: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvocationMeta {
    pub mode: String,
    pub targets: Vec<String>,
    pub config: InvocationConfigMeta,
    pub baseline: InvocationBaselineMeta,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvocationConfigMeta {
    pub repo_config: String,
    pub ci_config: String,
    pub org_config_env: String,
    pub org_config_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvocationBaselineMeta {
    pub enabled: bool,
    pub file: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LimitsMeta {
    pub max_file_count: usize,
    pub max_findings: usize,
    pub max_file_size_bytes: usize,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScopeMeta {
    pub gitignore_respected: bool,
    pub built_in_excluded_dirs: Vec<String>,
    pub binary_skip: bool,
    pub oversize_skip: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResultMeta {
    pub status: String,
    pub exit_code: i32,
    pub limit_reached: bool,
    pub limit_reasons: Vec<String>,
    pub summary: ResultSummaryMeta,
    pub timing: ResultTimingMeta,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResultSummaryMeta {
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub findings_count: usize,
    pub severity_counts: std::collections::HashMap<String, usize>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResultTimingMeta {
    pub started_at_utc: String,
    pub finished_at_utc: String,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactsMeta {
    pub report_html: ArtifactFileMeta,
    pub report_json: ArtifactFileMeta,
    pub effective_config: ArtifactFileMeta,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<ArtifactFileMeta>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactFileMeta {
    pub path: String,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyMeta {
    pub telemetry: String,
    pub network: String,
    pub bind: String,
}

#[derive(Clone)]
pub struct CachedRun {
    pub meta: RunMeta,
    pub report_html: String,
    pub report_json: String,
    pub effective_config: String,
    pub baseline_json: Option<String>,
    pub timestamp: Instant,
}

impl CachedRun {
    pub fn size_bytes(&self) -> usize {
        let mut size =
            self.report_html.len() + self.report_json.len() + self.effective_config.len();
        if let Some(b) = &self.baseline_json {
            size += b.len();
        }
        size
    }
}

pub struct RunCache {
    items: HashMap<String, CachedRun>,
    order: VecDeque<String>,
    max_runs_kept: usize,
    max_bytes_kept: usize,
    current_bytes: usize,
    ttl: Duration,
}

impl RunCache {
    pub fn new(max_runs_kept: usize, max_bytes_kept: usize, ttl_minutes: u64) -> Self {
        Self {
            items: HashMap::new(),
            order: VecDeque::new(),
            max_runs_kept,
            max_bytes_kept,
            current_bytes: 0,
            ttl: Duration::from_secs(ttl_minutes * 60),
        }
    }

    pub fn insert(&mut self, run_id: String, run: CachedRun) {
        self.evict_stale();

        let incoming_size = run.size_bytes();
        // If a single run is larger than the total capacity, do not cache it.
        if incoming_size > self.max_bytes_kept {
            return;
        }

        while self.items.len() >= self.max_runs_kept
            || (self.current_bytes + incoming_size) > self.max_bytes_kept
        {
            if let Some(oldest) = self.order.pop_front() {
                if let Some(old_run) = self.items.remove(&oldest) {
                    self.current_bytes = self.current_bytes.saturating_sub(old_run.size_bytes());
                }
            } else {
                break;
            }
        }

        self.order.push_back(run_id.clone());
        self.current_bytes += incoming_size;
        self.items.insert(run_id, run);
    }

    pub fn get(&mut self, run_id: &str) -> Option<CachedRun> {
        self.evict_stale();
        self.items.get(run_id).cloned()
    }

    fn evict_stale(&mut self) {
        let now = Instant::now();
        while let Some(oldest_id) = self.order.front() {
            if let Some(run) = self.items.get(oldest_id) {
                if now.duration_since(run.timestamp) > self.ttl {
                    let id = self.order.pop_front().unwrap();
                    if let Some(old_run) = self.items.remove(&id) {
                        self.current_bytes =
                            self.current_bytes.saturating_sub(old_run.size_bytes());
                    }
                } else {
                    break;
                }
            } else {
                self.order.pop_front();
            }
        }
    }
}
