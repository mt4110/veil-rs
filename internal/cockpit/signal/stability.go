package signal

import (
	"encoding/json"
	"os"
	"sort"
)

// StabilityLedger tracks the history of NOOP/Changed states.
type StabilityLedger struct {
	V             int        `json:"v"`
	CurrentStreak int        `json:"current_streak"`
	Runs          []RunEntry `json:"runs"`
}

type RunEntry struct {
	WeekID           string `json:"week_id"`
	Result           string `json:"result"` // NOOP or CHANGED
	IsPure           bool   `json:"is_pure"`
	ConsecutiveCount int    `json:"consecutive_count"`
	DeltaSummary     string `json:"delta_summary,omitempty"`
}

// SemanticSignal is a subset of Signal used for equality comparison.
// We ignore volatile fields like WeekID, CommitSHA, ArtifactRef.
type SemanticSignal struct {
	Category string
	CauseTag string
	Severity string
}

// DetectStability updates the ledger based on comparison between current and previous signals.
func DetectStability(ledger *StabilityLedger, current, prev []Signal, weekID string) {
	isNoop, isPure := checkNoop(current, prev)
	
	result := "CHANGED"
	if isNoop {
		result = "NOOP"
	}

	// Update Streak
	streak := 0
	if isNoop {
		streak = ledger.CurrentStreak + 1
	}
	
	entry := RunEntry{
		WeekID:           weekID,
		Result:           result,
		IsPure:           isPure,
		ConsecutiveCount: streak,
	}
	
	ledger.Runs = append(ledger.Runs, entry)
	ledger.CurrentStreak = streak
}

func checkNoop(current, prev []Signal) (bool, bool) {
	// Pure NOOP: Empty set
	if len(current) == 0 {
		// If current is empty, it is Pure NOOP.
		// Does it match previous? 
		// If previous was also empty, then No Delta.
		// If previous had signals, then it IS a Delta (Change to Empty).
		// Spec: "A week is considered NOOP if there is no delta...". 
		// Spec also says: "An empty signal set represents a pure NOOP." -> This might mean "State of being empty" is Pure NOOP state.
		// But strictly: NOOP means Invariant.
		// If Prev=[A], Current=[], that is a CHANGE (Recovery).
		// If Prev=[], Current=[], that is NOOP (Invariant Empty).
		
		// Wait, user spec: "Option C: No delta... Option A (0 Signals) as secondary."
		// "signals_v1.json == [] の週は 強い NOOP (pure NOOP) として別フラグ"
		// This implies if Current is Empty, it is Pure NOOP *regardless of delta*?
		// No, "Observed Invariant" is the core.
		// If I recover from Failure to Empty, that's a CHANGE.
		// But the resulting state is "Pure".
		// Let's interpret:
		// NOOP = (SemanticCurrent == SemanticPrev)
		// Pure = (len(Current) == 0)
		
		// If Prev=[A], Current=[], result is CHANGED. (IsPure=true).
		// If Prev=[], Current=[], result is NOOP. (IsPure=true).
		
		// Let's implement this strict interpretation.
		
		isPure := true
		isNoop := sameSet(current, prev)
		return isNoop, isPure
	}

	isPure := false
	isNoop := sameSet(current, prev)
	return isNoop, isPure
}

func sameSet(a, b []Signal) bool {
	if len(a) != len(b) {
		return false
	}
	
	// Convert to semantic and sort
	semA := toSemantic(a)
	semB := toSemantic(b)
	
	// Sort by Key (Cat:Cause)
	sort.Slice(semA, func(i, j int) bool {
		return semA[i].Category+semA[i].CauseTag < semA[j].Category+semA[j].CauseTag
	})
	sort.Slice(semB, func(i, j int) bool {
		return semB[i].Category+semB[i].CauseTag < semB[j].Category+semB[j].CauseTag
	})
	
	for i := range semA {
		if semA[i] != semB[i] {
			return false
		}
	}
	return true
}

func toSemantic(sigs []Signal) []SemanticSignal {
	out := make([]SemanticSignal, len(sigs))
	for i, s := range sigs {
		out[i] = SemanticSignal{
			Category: s.Category,
			CauseTag: s.CauseTag,
			Severity: s.Severity,
		}
	}
	return out
}

// IO Helpers for Stability Ledger (similar to Ledger)
func LoadStabilityLedger(path string) (*StabilityLedger, error) {
	data, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return &StabilityLedger{V: 1, Runs: []RunEntry{}}, nil
	}
	if err != nil {
		return nil, err
	}
	var l StabilityLedger
	if err := json.Unmarshal(data, &l); err != nil {
		return nil, err
	}
	return &l, nil
}

func SaveStabilityLedger(path string, l *StabilityLedger) error {
	data, err := json.MarshalIndent(l, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, append(data, '\n'), 0644)
}
