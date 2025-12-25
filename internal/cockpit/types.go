package cockpit

// ReasonEventV1 represents a single log line in reason_events_v1.jsonl
// strictly matching schemas/reason_event_v1.schema.json
type ReasonEventV1 struct {
	V          int      `json:"v"`                     // Const: 1
	Ts         string   `json:"ts"`                    // ISO8601/RFC3339
	ReasonCode string   `json:"reason_code"`           // "snake_case" enum
	Op         string   `json:"op"`                    // e.g. "dogfood.scorecard"
	Outcome    string   `json:"outcome"`               // "fail" | "skip"
	Taxon      string   `json:"taxon,omitempty"`       // "domain.key=value"
	Detail     string   `json:"detail,omitempty"`      // Free text
	HintCodes  []string `json:"hint_codes,omitempty"`  // ["retry_later", ...]
}

// MetricsV1 represents the snapshot in metrics_v1.json
// strictly matching schemas/metrics_v1.schema.json
type MetricsV1 struct {
	V       int         `json:"v"`       // Const: 1
	Metrics MetricsBody `json:"metrics"`
	Meta    MetaBody    `json:"meta,omitempty"`
}

type MetricsBody struct {
	CountsByReason map[string]int `json:"counts_by_reason"` // Generated from events
}

type MetaBody struct {
	Period    string `json:"period"`
	Repo      string `json:"repo"`
	GitCommit string `json:"git_commit"`
	Toolchain string `json:"toolchain"`
}

// Known ReasonCodes (matching Rust/Schema)
const (
	ReasonConfigInvalid        = "config_invalid"
	ReasonConfigMissing        = "config_missing_required"
	ReasonSchemaViolation      = "schema_violation"
	ReasonIoReadFailed         = "io_read_failed"
	ReasonIoWriteFailed        = "io_write_failed"
	ReasonCacheCorrupt         = "cache_corrupt"
	ReasonCacheVersionMismatch = "cache_version_mismatch"
	ReasonLockTimeout          = "lock_timeout"
	ReasonAtomicRenameFailed   = "atomic_rename_failed"
	ReasonOffline              = "offline"
	ReasonDnsFailed            = "dns_failed"
	ReasonTlsFailed            = "tls_failed"
	ReasonTimeout              = "timeout"
	ReasonRateLimited          = "rate_limited"
	ReasonHttp4xx              = "http_4xx"
	ReasonHttp5xx              = "http_5xx"
	ReasonJsonParseFailed      = "json_parse_failed"
	ReasonDecodeFailed         = "decode_failed"
	ReasonNotSupported         = "not_supported"
	ReasonInvariantBroken      = "internal_invariant_broken"
	ReasonUnexpected           = "unexpected"
)

// Known Hints
const (
	HintRetryLater   = "retry_later"
	HintCheckNetwork = "check_network"
	HintClearCache   = "clear_cache"
	HintUpgradeTool  = "upgrade_tool"
)
