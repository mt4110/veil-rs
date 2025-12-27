package signal

// Signal represents a Normalized Signal (Phase 16 Spec).
// It unifies observations from multiple sources into a trackable entity.
type Signal struct {
	WeekID     string   `json:"week_id"`     // e.g. "2025-W12"
	CommitSHA  string   `json:"commit_sha"`  // The commit where this signal was observed
	Category   string   `json:"category"`    // e.g. "entropy", "scorecard", "test"
	CauseTag   string   `json:"cause_tag"`   // e.g. "R001", "audit.fail" (Rule ID based)
	Severity   string   `json:"severity"`    // "improve", "warn", "info"
	Source     []string `json:"source"`      // ["dogfood-json", "ci-log", "fixlog"]
	ArtifactRef string  `json:"artifact_ref"` // Path to detailed evidence
}

// Recurrence represents the state of a recurring signal in the global ledger.
// It tracks persistence over time based on WEEK_ID logic.
type Recurrence struct {
	Category     string   `json:"category"`
	CauseTag     string   `json:"cause_tag"`
	Status       string   `json:"status"`         // "active", "inactive" (resolved is implicit if inactive for long?) - Spec says "active" | "inactive"
	FirstSeenWeek string   `json:"first_seen_week"`
	LastSeenWeek  string   `json:"last_seen_week"`
	HitCount      int      `json:"hit_count"`      // Total occurrences
	RecentWeeks   []string `json:"recent_weeks"`   // Sliding window for detection (last 4-8 weeks)
}

// Ledger is the top-level structure for recurring_signals.json
type Ledger struct {
	V       int          `json:"v"` // Schema version 1
	Signals []Recurrence `json:"signals"`
}
