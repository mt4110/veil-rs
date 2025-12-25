package cockpit

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"
)

const (
	TimezoneTokyo   = "Asia/Tokyo"
	MetricsFilename = "metrics_v1.json"
)

// Dogfood executes the weekly dogfood process.
func Dogfood() (string, error) {
	// 1. Determine Week/Time strict
	dirName := GetWeekID() // e.g. 2025-W52-Tokyo

	// Separate paths per Phase 12 requirement:
	// result/dogfood/<week> -> ignored raw events
	// docs/dogfood/<week>   -> tracked reports
	resultDir := filepath.Join("result", "dogfood", dirName)
	docsDir := filepath.Join("docs", "dogfood", dirName)

	if err := os.MkdirAll(resultDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create result dir %s: %w", resultDir, err)
	}
	if err := os.MkdirAll(docsDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create docs dir %s: %w", docsDir, err)
	}

	// Session state for events
	events := []ReasonEventV1{}

	// Helper to log event
	logEvent := func(code, op, outcome, taxon, detail string, hints []string) {
		e := ReasonEventV1{
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
	prevWeekID, prevMetrics, err := loadPreviousMetrics("docs/dogfood", dirName)
	if err != nil {
		// Log but don't fail, maybe first run
		fmt.Fprintf(os.Stderr, "Warning: could not load previous metrics: %v\n", err)
	}

	// 2. Scorecard (Execution)
	// Output to Docs (it's a report)
	scErr := generateScorecard(docsDir)
	if scErr != nil {
		logEvent(ReasonUnexpected, "audit.scorecard", "fail", "", scErr.Error(), []string{HintRetryLater})
		fmt.Fprintf(os.Stderr, "Scorecard failed: %v\n", scErr)
	}

	// 3. Write Events (result/dogfood/...) - Ignored raw data
	if err := writeEvents(resultDir, events); err != nil {
		return "", fmt.Errorf("failed to write events: %w", err)
	}

	// 4. Aggregate Metrics (metrics_v1.json) -> Docs
	// We read events from resultDir if available, or use local memory events
	if err := generateMetricsV1(docsDir, events, dirName); err != nil {
		return "", fmt.Errorf("metrics generation failed: %w", err)
	}

	// 5. Weekly Report & Worklist -> Docs
	if err := generateWeeklyArtifacts(docsDir, dirName, prevWeekID, prevMetrics); err != nil {
		return "", fmt.Errorf("weekly artifacts generation failed: %w", err)
	}

	return docsDir, nil
}

// GetWeekID returns the strictly formatted current week ID
func GetWeekID() string {
	loc, _ := time.LoadLocation(TimezoneTokyo)
	if loc == nil {
		loc = time.FixedZone(TimezoneTokyo, 9*60*60)
	}
	now := time.Now().In(loc)
	y, w := now.ISOWeek()
	return fmt.Sprintf("%04d-W%02d-Tokyo", y, w)
}

func ensureMetrics(dir string, localEvents []ReasonEventV1) error {
	path := filepath.Join(dir, MetricsFilename)
	if _, err := os.Stat(path); err == nil {
		return nil // Exists
	}
	// Fallback: generate from local events (Dogfooding the dogfooder)
	return generateMetricsV1(dir, localEvents, GetWeekID())

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

func generateMetricsV1(dir string, events []ReasonEventV1, weekID string) error {
	// Aggregate from events
	counts := make(map[string]int)
	hintCounts := make(map[string]int)

	for _, e := range events {
		counts[e.ReasonCode]++
		// Exclude dogfood.* ops from hint aggregation (Top3/worklist input)
		if strings.HasPrefix(e.Op, "dogfood.") { continue }
		for _, h := range e.HintCodes {
			hintCounts[h]++
		}
	}

	// BTreeMap behavior in Go: manually sort keys when printing?
	// Go maps are unordered. We need to rely on the struct definition or custom marshaling if we want strict output?
	// The Rust side guarantees strictness. Go side for fallback:
	// We just marshal; standard library sorts map keys by default when marshaling since Go 1.0??
	// Yes, encoding/json sorts map keys.

	m := MetricsV1{
		V: 1,
		Metrics: MetricsBody{
			CountsByReason: counts,
			CountsByHint:   hintCounts,
		},
		Meta: MetaBody{
			Period:    weekID,
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
	// Append newline
	return os.WriteFile(filepath.Join(dir, MetricsFilename), append(data, '\n'), 0644)
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
	val, found := findScoreInMap(raw)
	if !found {
		return 0, fmt.Errorf("could not find 'score' or 'aggregateScore.score' in scorecard output")
	}
	return val, nil
}

func findScoreInMap(raw map[string]interface{}) (float64, bool) {
	if v, ok := raw["score"]; ok {
		if f, ok := v.(float64); ok {
			return f, true
		}
	}
	if agg, ok := raw["aggregateScore"]; ok {
		if f, ok := agg.(float64); ok {
			return f, true
		} else if aggMap, ok := agg.(map[string]interface{}); ok {
			if s, ok := aggMap["score"]; ok {
				if f, ok := s.(float64); ok {
					return f, true
				}
			}
		}
	}
	return 0, false
}

func loadPreviousMetrics(baseDir, currentWeekID string) (string, *MetricsV1, error) {
	entries, err := os.ReadDir(baseDir)
	if err != nil {
		return "", nil, err
	}

	var weeks []string
	for _, e := range entries {
		if e.IsDir() && strings.HasSuffix(e.Name(), "-Tokyo") {
			weeks = append(weeks, e.Name())
		}
	}
	sort.Strings(weeks)

	// Find the one strictly before current
	var prevID string
	for i := len(weeks) - 1; i >= 0; i-- {
		if weeks[i] < currentWeekID {
			prevID = weeks[i]
			break
		}
	}

	if prevID == "" {
		return "", nil, nil
	}

	path := filepath.Join(baseDir, prevID, MetricsFilename)
	data, err := os.ReadFile(path)
	if err != nil {
		return prevID, nil, err
	}

	var m MetricsV1
	if err := json.Unmarshal(data, &m); err != nil {
		return prevID, nil, err
	}

	return prevID, &m, nil
}

func generateWeeklyArtifacts(dir, weekID, prevWeekID string, prevMetrics *MetricsV1) error {
	// Read Current Metrics
	mPath := filepath.Join(dir, MetricsFilename)
	mData, err := os.ReadFile(mPath)
	if err != nil {
		return err
	}
	var m MetricsV1
	if err := json.Unmarshal(mData, &m); err != nil {
		return err
	}

	// Calculate Top 3 (Worklist)
	worklist, err := generateWorklist(weekID, &m, prevMetrics)
	if err != nil {
		return err
	}

	// Write Worklist V1
	wlData, err := json.MarshalIndent(worklist, "", "  ")
	if err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(dir, "worklist_v1.json"), append(wlData, '\n'), 0644); err != nil {
		return err
	}

	// Read Scorecard (if available)
	scScore := readScorecardFile(dir)

	// Generate and Write Report
	reportMD := generateWeeklyReportMD(weekID, scScore, &m, prevMetrics, worklist, prevWeekID)
	return os.WriteFile(filepath.Join(dir, "weekly.md"), []byte(reportMD), 0644)
}

func readScorecardFile(dir string) string {
	scPath := filepath.Join(dir, "scorecard.txt")
	scContent, _ := os.ReadFile(scPath)
	scScore := "N/A"
	for _, line := range strings.Split(string(scContent), "\n") {
		if strings.HasPrefix(line, "scorecard_score_0_10:") {
			scScore = strings.TrimSpace(strings.TrimPrefix(line, "scorecard_score_0_10:"))
			break
		}
	}
	return scScore
}

func generateWeeklyReportMD(weekID, scScore string, m, prevMetrics *MetricsV1, worklist *WorklistV1, prevWeekID string) string {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("# Weekly dogfood %s\n\n", weekID))

	sb.WriteString("## Summary\n")
	sb.WriteString(fmt.Sprintf("- Scorecard: **%s**\n", scScore))

	totalFailures, deltaMsg := calculateFailureDelta(m, prevMetrics, prevWeekID)
	sb.WriteString(fmt.Sprintf("- Total Failure Events: %d%s\n\n", totalFailures, deltaMsg))

	sb.WriteString("## 改善対象 Top3 (Phase 13 Input)\n")
	sb.WriteString("| Rank | Action ID | Title | Hint Key | Count | Delta | Score | Playbook |\n")
	sb.WriteString("|---|---|---|---|---|---|---|---|\n")

	sb.WriteString(formatWorklistTable(worklist))

	sb.WriteString("\n## Improvement Memo\n- [ ] (Add actionable items here)\n")
	return sb.String()
}

func calculateFailureDelta(m, prevMetrics *MetricsV1, prevWeekID string) (int, string) {
	totalFailures := 0
	for _, v := range m.Metrics.CountsByReason {
		totalFailures += v
	}

	deltaMsg := ""
	if prevMetrics != nil {
		prevTotal := 0
		for _, v := range prevMetrics.Metrics.CountsByReason {
			prevTotal += v
		}
		diff := totalFailures - prevTotal
		sign := "+"
		if diff < 0 {
			sign = ""
		}
		deltaMsg = fmt.Sprintf(" (vs %s: %s%d)", prevWeekID, sign, diff)
	}
	return totalFailures, deltaMsg
}

func formatWorklistTable(worklist *WorklistV1) string {
	var sb strings.Builder
	if len(worklist.Items) == 0 {
		sb.WriteString("| - | - | *(No actions)* | - | - | - | - | - |\n")
	} else {
		for _, item := range worklist.Items {
			if item.Rank > 3 {
				break
			}
			deltaStr := fmt.Sprintf("%d", item.Signals.Delta)
			if item.Signals.Delta > 0 {
				deltaStr = "+" + deltaStr
			}

			pbRef := item.PlaybookRef
			if pbRef == "" {
				pbRef = "-"
			}

			sb.WriteString(fmt.Sprintf("| %d | `%s` | %s | `%s` | %d | %s | **%d** | %s |\n",
				item.Rank, item.ActionId, item.Title, item.Signals.HintKey, item.Signals.CountNow, deltaStr, item.Score, pbRef))
		}
	}
	return sb.String()
}

type WorklistV1 struct {
	V           int            `json:"v"`
	WeekID      string         `json:"week_id"`
	GeneratedAt string         `json:"generated_at"`
	GitSHA      string         `json:"git_sha,omitempty"`
	Items       []WorklistItem `json:"items"`
}

type WorklistItem struct {
	Rank        int         `json:"rank"`
	ActionId    string      `json:"action_id"`
	Title       string      `json:"title"`
	Score       int         `json:"score"`
	PlaybookRef string      `json:"playbook_ref,omitempty"`
	Suggested   []string    `json:"suggested_paths,omitempty"`
	Signals     SignalStats `json:"signals"`
}

type SignalStats struct {
	CountNow int    `json:"count_now"`
	Delta    int    `json:"delta"`
	HintKey  string `json:"hint_key"`
}

func generateWorklist(weekID string, current, prev *MetricsV1) (*WorklistV1, error) {
	// 1. Gather keys
	hints := make(map[string]int)
	for k, v := range current.Metrics.CountsByHint {
		hints[k] = v
	}

	// 2. Score Items
	var items []WorklistItem
	for hintKey, count := range hints {
		items = append(items, scoreWorklistItem(hintKey, count, prev))
	}

	// 3. Sort (Deterministic)
	sortWorklistItems(items)

	// 4. Assign Rank
	for i := range items {
		items[i].Rank = i + 1
	}

	// 5. Deterministic Timestamp
	genTime, err := calculateStrictTimestamp(weekID)
	if err != nil {
		return nil, fmt.Errorf("failed to calculate strict timestamp for %s: %w", weekID, err)
	}

	return &WorklistV1{
		V:           1,
		WeekID:      weekID,
		GeneratedAt: genTime,
		Items:       items,
	}, nil
}

func scoreWorklistItem(hintKey string, count int, prev *MetricsV1) WorklistItem {
	// Calculate Delta
	prevCount := 0
	if prev != nil {
		prevCount = prev.Metrics.CountsByHint[hintKey]
	}
	delta := count - prevCount

	// Score Formula: Count * 10 + Max(Delta, 0) * 25
	wCount := 10
	wDelta := 25
	deltaScore := delta
	if deltaScore < 0 {
		deltaScore = 0
	}
	score := (count * wCount) + (deltaScore * wDelta)

	// Resolve Blueprint
	bp := getActionBlueprint(hintKey)

	// Create Action ID if missing for fallback
	if bp.ActionId == "" {
		bp.ActionId = "Z-UNMAPPED"
		bp.Title = fmt.Sprintf("Unmapped hint: %s", hintKey)
	}

	return WorklistItem{
		Rank:        0, // Fill later
		ActionId:    bp.ActionId,
		Title:       bp.Title,
		Score:       score,
		PlaybookRef: bp.PlaybookRef,
		Suggested:   bp.Suggested,
		Signals: SignalStats{
			CountNow: count,
			Delta:    delta,
			HintKey:  hintKey,
		},
	}
}

func sortWorklistItems(items []WorklistItem) {
	// Sort (Deterministic)
	// Score DESC -> Count DESC -> ActionId ASC -> HintKey ASC
	sort.Slice(items, func(i, j int) bool {
		si, sj := items[i], items[j]
		if si.Score != sj.Score {
			return si.Score > sj.Score
		}
		if si.Signals.CountNow != sj.Signals.CountNow {
			return si.Signals.CountNow > sj.Signals.CountNow
		}
		if si.ActionId != sj.ActionId {
			return si.ActionId < sj.ActionId
		}
		return si.Signals.HintKey < sj.Signals.HintKey
	})
}

func calculateStrictTimestamp(weekID string) (string, error) {
	// Expected format: YYYY-Www-Tokyo
	parts := strings.Split(weekID, "-")
	if len(parts) < 2 {
		return "", fmt.Errorf("invalid weekID format")
	}

	year, err := strconv.Atoi(parts[0])
	if err != nil {
		return "", err
	}

	// parts[1] is "Www" e.g. "W52"
	if len(parts[1]) < 2 || parts[1][0] != 'W' {
		return "", fmt.Errorf("invalid week part")
	}
	week, err := strconv.Atoi(parts[1][1:])
	if err != nil {
		return "", err
	}

	// Calculate Monday of that ISO week
	// Jan 4th is always in ISO Week 1
	jan4 := time.Date(year, time.January, 4, 0, 0, 0, 0, time.UTC)
	isoYear, _ := jan4.ISOWeek()

	// Sanity check: if Jan 4 is in previous ISO year (can happen? no, by definition Jan 4 is Week 1 or greater of 'year')
	// Wait, definition: "The first week of a year is the week that contains the first Thursday of the year (or, equivalently, 4 January)."
	// So isoYear should be 'year'.
	if isoYear != year {
		// This theoretically shouldn't happen for Jan 4
	}

	// Find start of Week 1 (Monday)
	// Go Weekday: 0(Sun), 1(Mon)...6(Sat)
	wd := int(jan4.Weekday())
	if wd == 0 {
		wd = 7
	} // Convert Sun=0 to 7
	// Mon(1) -> offset 0
	// ...
	// Sun(7) -> offset 6
	offset := wd - 1
	week1Mon := jan4.AddDate(0, 0, -offset)

	// Add (week-1) weeks
	targetMon := week1Mon.AddDate(0, 0, (week-1)*7)

	// Convert to JST (UTC+9) and set 00:00:00
	// We want the string to represent 00:00 JST.
	// If we just format targetMon (UTC) as RFC3339, it's UTC.
	// We want to FORCE the timezone to be +09:00 for the output string "YYYY-MM-DDT00:00:00+09:00".
	// So we should construct the time in JST.

	loc := time.FixedZone(TimezoneTokyo, 9*60*60)
	// The targetMon is 00:00 UTC (from jan4 UTC).
	// But "Monday 00:00 JST" is effectively "Sunday 15:00 UTC".
	// We want the resulting object to Print as 00:00+09:00.
	// So we construct a date in loc.

	y, m, d := targetMon.Date()
	mondayJST := time.Date(y, m, d, 0, 0, 0, 0, loc)

	return mondayJST.Format(time.RFC3339), nil
}

type Blueprint struct {
	ActionId    string
	Title       string
	PlaybookRef string
	Suggested   []string
}

func getActionBlueprint(hint string) Blueprint {
	// In production, this might load from a file.
	switch hint {
	case "retry_later":
		return Blueprint{ActionId: "A-WAIT-001", Title: "Retry operation later (Transient)", PlaybookRef: "docs/playbooks/transient.md"}
	case "check_network":
		return Blueprint{ActionId: "A-NET-001", Title: "Verify network connectivity", Suggested: []string{"/etc/hosts"}, PlaybookRef: "docs/playbooks/network.md"}
	case "check_dns":
		return Blueprint{ActionId: "A-NET-002", Title: "Check DNS resolution", Suggested: []string{"/etc/resolv.conf"}}
	case "check_tls_clock":
		return Blueprint{ActionId: "A-SEC-001", Title: "Verify system clock & TLS"}
	case "clear_cache":
		return Blueprint{ActionId: "A-DISK-001", Title: "Clear local cache"}
	case "upgrade_tool":
		return Blueprint{ActionId: "A-UPG-001", Title: "Upgrade veil-rs to latest"}
	case "reduce_parallelism":
		return Blueprint{ActionId: "A-PERF-001", Title: "Reduce concurrency/parallelism"}
	case "check_token_permissions":
		return Blueprint{ActionId: "A-IAM-001", Title: "Verify token permissions"}
	default:
		return Blueprint{ActionId: "", Title: ""} // Will result in Fallback
	}
}
