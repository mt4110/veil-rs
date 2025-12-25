package cockpit

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// Dogfood executes the weekly dogfood process.
func Dogfood() (string, error) {
	// 1. Determine Week/Time
	loc, err := time.LoadLocation("Asia/Tokyo")
	if err != nil {
		loc = time.FixedZone("Asia/Tokyo", 9*60*60)
	}
	now := time.Now().In(loc)
	y, w := now.ISOWeek()

	dirName := fmt.Sprintf("%04d-W%02d-Tokyo", y, w)
	outDir := filepath.Join("docs", "dogfood", dirName)

	if err := os.MkdirAll(outDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create dir %s: %w", outDir, err)
	}

	// Session state for events
	events := []ReasonEventV1{}

	// Helper to log event
	logEvent := func(code, op, outcome, taxon, detail string, hints []string) {
		e := ReasonEventV1{
			V:          1,
			Ts:         now.Format(time.RFC3339),
			ReasonCode: code,
			Op:         op,
			Outcome:    outcome,
			Taxon:      taxon,
			Detail:     detail,
			HintCodes:  hints,
		}
		events = append(events, e)
	}

	// 2. Scorecard (Execution)
	// We run this first or as a main activity. If it fails, log event.
	scErr := generateScorecard(outDir)
	if scErr != nil {
		// Log specific failure based on error content if possible
		logEvent(ReasonUnexpected, "dogfood.scorecard", "fail", "", scErr.Error(), []string{HintRetryLater})
		// We don't return immediately, we try to finish what we can?
		// User requirement: "dogfood の各ステップ...でエラーを捕まえたら..."
		// But if scorecard fails essential file missing?
		// Scorecard generation might just fail to produce score, but we should proceed to metrics/weekly.
		fmt.Fprintf(os.Stderr, "Scorecard failed: %v\n", scErr)
	}

	// 3. Write Events (reason_events_v1.jsonl)
	// Even if empty? User says "eventsは基本 fail/skip だけ". If we have no failures, file might be empty.
	// That's fine.
	if err := writeEvents(outDir, events); err != nil {
		return "", fmt.Errorf("failed to write events: %w", err)
	}

	// 4. Aggregate Metrics (metrics_v1.json)
	if err := generateMetricsV1(outDir, events, y, w); err != nil {
		return "", fmt.Errorf("metrics generation failed: %w", err)
	}

	// 5. Weekly Markdown
	// Reads the generated files (or we pass data)
	if err := generateWeekly(outDir, y, w); err != nil {
		return "", fmt.Errorf("weekly.md generation failed: %w", err)
	}

	return outDir, nil
}

func writeEvents(dir string, events []ReasonEventV1) error {
	f, err := os.Create(filepath.Join(dir, "reason_events_v1.jsonl"))
	if err != nil {
		return err
	}
	defer f.Close()

	enc := json.NewEncoder(f)
	for _, e := range events {
		if err := enc.Encode(e); err != nil {
			return err
		}
	}
	return nil
}

func generateMetricsV1(dir string, events []ReasonEventV1, y, w int) error {
	// Aggregate from events
	counts := make(map[string]int)
	for _, e := range events {
		counts[e.ReasonCode]++
	}

	m := MetricsV1{
		V: 1,
		Metrics: MetricsBody{
			CountsByReason: counts,
		},
		Meta: MetaBody{
			Period:    fmt.Sprintf("%04d-W%02d", y, w),
			Toolchain: "nix",
			Repo:      "github.com/mt4110/veil-rs", // Default
		},
	}

	if sha, err := getGitSHA(); err == nil {
		m.Meta.GitCommit = sha
	}
	if r := os.Getenv("GITHUB_REPOSITORY"); r != "" {
		m.Meta.Repo = "github.com/" + r
	}

	data, err := json.MarshalIndent(m, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(filepath.Join(dir, "metrics_v1.json"), append(data, '\n'), 0644)
}

func getGitSHA() (string, error) {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	out, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

func generateScorecard(dir string) error {
	repo := os.Getenv("GITHUB_REPOSITORY")
	if repo == "" {
		repo = "mt4110/veil-rs"
	}
	repoURL := "github.com/" + repo

	cmd := exec.Command("scorecard", "--repo="+repoURL, "--format=json")
	if token := os.Getenv("GITHUB_AUTH_TOKEN"); token != "" {
		cmd.Env = append(os.Environ(), "GITHUB_AUTH_TOKEN="+token)
	} else if token := os.Getenv("GITHUB_TOKEN"); token != "" {
		cmd.Env = append(os.Environ(), "GITHUB_AUTH_TOKEN="+token)
	}

	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("scorecard cli failed: %v\nOutput:\n%s", err, string(out))
	}

	scoreVal, err := parseScorecardScore(out)
	if err != nil {
		return err
	}

	score100 := int(scoreVal*10 + 0.5)
	lines := []string{
		fmt.Sprintf("scorecard_score_0_100: %d", score100),
		fmt.Sprintf("scorecard_score_0_10: %.1f", scoreVal),
		fmt.Sprintf("scorecard_repo: %s", repoURL),
		"scorecard_source: ossf/scorecard CLI",
		"",
	}

	return os.WriteFile(filepath.Join(dir, "scorecard.txt"), []byte(strings.Join(lines, "\n")), 0644)
}

func parseScorecardScore(jsonOutput []byte) (float64, error) {
	var raw map[string]interface{}
	if err := json.Unmarshal(jsonOutput, &raw); err != nil {
		return 0, fmt.Errorf("failed to parse scorecard json: %w", err)
	}

	var scoreVal float64
	found := false

	if v, ok := raw["score"]; ok {
		if f, ok := v.(float64); ok {
			scoreVal = f
			found = true
		}
	}

	if !found {
		if agg, ok := raw["aggregateScore"]; ok {
			if f, ok := agg.(float64); ok {
				scoreVal = f
				found = true
			} else if aggMap, ok := agg.(map[string]interface{}); ok {
				if s, ok := aggMap["score"]; ok {
					if f, ok := s.(float64); ok {
						scoreVal = f
						found = true
					}
				}
			}
		}
	}

	if !found {
		return 0, fmt.Errorf("could not find 'score' or 'aggregateScore.score' in scorecard output")
	}
	return scoreVal, nil
}

func generateWeekly(dir string, year, week int) error {
	// Read Metrics
	mPath := filepath.Join(dir, "metrics_v1.json")
	mData, err := os.ReadFile(mPath)
	if err != nil {
		return err
	}
	var m MetricsV1
	if err := json.Unmarshal(mData, &m); err != nil {
		return err
	}

	// Read Events (Aggregated Hints)
	ePath := filepath.Join(dir, "reason_events_v1.jsonl")
	eData, err := os.ReadFile(ePath)
	hintCounts := make(map[string]int)
	if err == nil {
		lines := strings.Split(string(eData), "\n")
		for _, line := range lines {
			if strings.TrimSpace(line) == "" {
				continue
			}
			var e ReasonEventV1
			if json.Unmarshal([]byte(line), &e) == nil {
				for _, h := range e.HintCodes {
					hintCounts[h]++
				}
			}
		}
	}

	// Sort Top Reasons
	type kv struct {
		K string
		V int
	}
	var reasons []kv
	for k, v := range m.Metrics.CountsByReason {
		reasons = append(reasons, kv{k, v})
	}
	sort.Slice(reasons, func(i, j int) bool {
		return reasons[i].V > reasons[j].V // Descending
	})

	// Sort Top Hints
	var hints []kv
	for k, v := range hintCounts {
		hints = append(hints, kv{k, v})
	}
	sort.Slice(hints, func(i, j int) bool {
		return hints[i].V > hints[j].V // Descending
	})

	// Read Scorecard for Summary
	scPath := filepath.Join(dir, "scorecard.txt")
	scContent, _ := os.ReadFile(scPath)
	scScore := "N/A"
	for _, line := range strings.Split(string(scContent), "\n") {
		if strings.HasPrefix(line, "scorecard_score_0_10:") {
			scScore = strings.TrimSpace(strings.TrimPrefix(line, "scorecard_score_0_10:"))
			break
		}
	}

	// Build Markdown
	var sb strings.Builder
	title := fmt.Sprintf("# Weekly dogfood %04d-W%02d (Tokyo)\n\n", year, week)
	sb.WriteString(title)

	sb.WriteString("## Summary\n")
	sb.WriteString(fmt.Sprintf("- Scorecard: **%s**\n", scScore))
	sb.WriteString(fmt.Sprintf("- Total Incidents: %d\n\n", len(reasons))) // Roughly distinct reasons count? Or total count?
	// Actually reasons is list of pair. Calculate total?
	totalOps := 0
	for _, r := range reasons {
		totalOps += r.V
	}
	sb.WriteString(fmt.Sprintf("- Total Failure Events: %d\n\n", totalOps))

	sb.WriteString("## Top Reasons\n")
	if len(reasons) == 0 {
		sb.WriteString("*(No failures reported)*\n")
	} else {
		for i, r := range reasons {
			if i >= 5 {
				break
			}
			sb.WriteString(fmt.Sprintf("1. `%s`: %d\n", r.K, r.V))
		}
	}
	sb.WriteString("\n")

	sb.WriteString("## Top Hints\n")
	if len(hints) == 0 {
		sb.WriteString("*(No hints)*\n")
	} else {
		for i, h := range hints {
			if i >= 5 {
				break
			}
			sb.WriteString(fmt.Sprintf("1. `%s`: %d\n", h.K, h.V))
		}
	}
	sb.WriteString("\n")

	sb.WriteString("## Improvement Memo\n")
	sb.WriteString("- [ ] (Add actionable items here)\n")

	return os.WriteFile(filepath.Join(dir, "weekly.md"), []byte(sb.String()), 0644)
}

