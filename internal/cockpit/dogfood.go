package cockpit

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"veil-rs/internal/cockpit/ui"
	"veil-rs/internal/types"
)

const (
	TimezoneTokyo   = "Asia/Tokyo"
	MetricsFilename = "metrics_v1.json"
	// DefaultRepoName is the fallback repository name if GITHUB_REPOSITORY is unset.
	DefaultRepoName = "mt4110/veil-rs"
)

// Dogfood executes the weekly dogfood process.
// Returns output directory, exit code, and error.
func Dogfood(overrideWeekID string) (string, int, error) {
	// 1. Determine Week/Time strict
	weekID := overrideWeekID
	if weekID == "" {
		weekID = GetWeekID() // e.g. 2025-W52
	}

	// Validate WeekID format strictly (YYYY-Www)
	if !isValidWeekID(weekID) {
		return "", 3, fmt.Errorf("invalid WEEK_ID format: %q", weekID)
	}

	// Directory name is derived from WEEK_ID (Tokyo)
	dirName := weekID + "-Tokyo"
	resultDir := filepath.Join("result", "dogfood", dirName)
	docsDir := filepath.Join("docs", "dogfood", dirName)

	if err := os.MkdirAll(resultDir, 0755); err != nil {
		return "", 10, fmt.Errorf("failed to create result dir %s: %w", resultDir, err)
	}
	if err := os.MkdirAll(docsDir, 0755); err != nil {
		return "", 10, fmt.Errorf("failed to create docs dir %s: %w", docsDir, err)
	}

	// Session state for events
	events := []types.ReasonEventV1{}

	// Helper to log event
	logEvent := func(code, op, outcome, taxon, detail string, hints []string) {
		e := types.ReasonEventV1{
			V:          1,
			Ts:         time.Now().Format(time.RFC3339),
			ReasonCode: code,
			Op:         op,
			Outcome:    outcome,
			Taxon:      taxon,
			Detail:     detail,
			HintCodes:  hints,
		}
		events = append(events, e)
	}

	// 2. Resolve Previous Week
	prevWeekDir, prevMetrics, err := loadPreviousMetrics("docs/dogfood", dirName)
	if err != nil {
		// Log but don't fail, maybe first run
		fmt.Fprintf(os.Stderr, "Warning: could not load previous metrics: %v\n", err)
	}

	// prevWeekDir is a directory name (YYYY-Www-Tokyo). Derive WEEK_ID (YYYY-Www) for display/contract.
	prevWeekID := strings.TrimSuffix(prevWeekDir, "-Tokyo")

	// 2. Scorecard (Execution)
	// Output to Docs (it's a report)
	spSc := ui.NewSpinner("running scorecard audit")
	spSc.Start()

	scorecardPath := filepath.Join(docsDir, "scorecard.txt")
	scErr := generateScorecard(docsDir)
	if scErr != nil {
		spSc.StopWarn(fmt.Sprintf("scorecard failed: %v", scErr))

		logEvent(types.ReasonUnexpected, "audit.scorecard", "fail", "", scErr.Error(), []string{types.HintRetryLater})
		
		// Fallback: write error to file (User Request)
		msg := fmt.Sprintf("scorecard unavailable: %v\n", scErr)
		_ = os.WriteFile(scorecardPath, []byte(msg), 0644)
		
		// Removed persistent stderr log to keep UI clean, StopWarn handled it.
	} else {
		spSc.StopOK("scorecard done")

		// Validation: ensure file exists
		if _, statErr := os.Stat(scorecardPath); statErr != nil {
			_ = os.WriteFile(scorecardPath, []byte("scorecard: ok (no output captured)\n"), 0644)
		}
	}

	// 3. Write Events (result/dogfood/...) - Ignored raw data
	if err := writeEvents(resultDir, events); err != nil {
		return "", 10, fmt.Errorf("failed to write events: %w", err)
	}

	// 4. Aggregate Metrics (metrics_v1.json) -> Docs
	spAn := ui.NewSpinner("aggregating metrics & generating worklist")
	spAn.Start()

	// We read events from resultDir if available, or use local memory events
	if err := generateMetricsV1(docsDir, events, weekID); err != nil {
		spAn.StopWarn("metrics gen failed")
		return "", 10, fmt.Errorf("metrics generation failed: %w", err)
	}

	// 5. Weekly Report & Worklist -> Docs (Phase 16: Signal Processing)
	if err := generateWeeklyArtifacts(docsDir, weekID, prevWeekID, prevMetrics, events); err != nil {
		spAn.StopWarn("artifact gen failed")
		return "", 10, fmt.Errorf("weekly artifacts generation failed: %w", err)
	}
	spAn.StopOK("dogfood loop complete")

	return docsDir, 0, nil
}

// GetWeekID returns the strictly formatted current week ID (Tokyo, ISO week)
// Format: YYYY-W## (e.g. 2025-W52)
func GetWeekID() string {
	loc, _ := time.LoadLocation(TimezoneTokyo)
	if loc == nil {
		loc = time.FixedZone(TimezoneTokyo, 9*60*60)
	}
	now := time.Now().In(loc)
	y, w := now.ISOWeek()
	return fmt.Sprintf("%04d-W%02d", y, w)
}

func ensureMetrics(dir string, localEvents []types.ReasonEventV1) error {
	path := filepath.Join(dir, MetricsFilename)
	if _, err := os.Stat(path); err == nil {
		return nil // Exists
	}
	// Fallback: generate from local events (Dogfooding the dogfooder)
	return generateMetricsV1(dir, localEvents, GetWeekID())

}
