package signal

import (
	"fmt"
	"strings"

	"veil-rs/internal/types"
)

// Normalize converts a list of raw ReasonEventV1 into Normalized Signals.
// It handles grouping, severity mapping, and source merging.
func Normalize(events []types.ReasonEventV1, weekID, commitSHA string) []Signal {
	// Group by Category + CauseTag
	// In ReasonEventV1:
	// - Category ~ Taxon or implicitly derived from Op? 
	// - Phase 13 used Op like "audit.scorecard" and ReasonCode "config_invalid".
	// - Spec says "CauseTag (based on rule_id)".
	// Mapping Strategy:
	// Category = Top-level of Op (e.g. "audit", "entropy")
	// CauseTag = ReasonCode (e.g. "config_invalid") OR Op specific rule
	
	sigMap := make(map[string]*Signal)

	for _, e := range events {
		// 1. Derive Category/Cause
		category, cause := deriveCategoryCause(e)
		key := category + ":" + cause

		// 2. Derive Severity
		severity := "info"
		if e.Outcome == "fail" {
			severity = "warn"
		} else if e.Outcome == "pass" {
			severity = "improve" // optimistically? or "info"? Spec example says "improve". 
			// Actually "pass" usually entails improvement if it was failing. 
			// For now, mapping non-fail to info unless we explicitly detect improvement.
			severity = "info"
		}

		// 3. Merge or Create
		if s, exists := sigMap[key]; exists {
			// Merge source
			if !contains(s.Source, "dogfood-json") {
				s.Source = append(s.Source, "dogfood-json")
			}
			// Update severity logic (escalate warn)
			if severity == "warn" {
				s.Severity = "warn"
			}
		} else {
			sigMap[key] = &Signal{
				WeekID:      weekID,
				CommitSHA:   commitSHA,
				Category:    category,
				CauseTag:    cause,
				Severity:    severity,
				Source:      []string{"dogfood-json"}, // Assuming mostly events come from here
				ArtifactRef: fmt.Sprintf("docs/dogfood/%s-Tokyo/metrics_v1.json", weekID),
			}
		}
	}

	// Convert map to slice
	var result []Signal
	for _, s := range sigMap {
		result = append(result, *s)
	}

	return result
}

func deriveCategoryCause(e types.ReasonEventV1) (string, string) {
	if e.ReasonCode == "unexpected" {
		return "Unexpected", e.Op
	}
	
	// Example: "audit.scorecard" -> "Audit"
	parts := strings.Split(e.Op, ".")
	if len(parts) > 0 && parts[0] != "" {
		// Capitalize first letter
		cat := strings.Title(parts[0])
		return cat, e.ReasonCode
	}
	
	return e.Op, e.ReasonCode
}

func contains(slice []string, item string) bool {
	for _, s := range slice {
		if s == item {
			return true
		}
	}
	return false
}
