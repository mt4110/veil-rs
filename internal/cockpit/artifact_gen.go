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

	"veil-rs/internal/cockpit/signal"
	"veil-rs/internal/types"
)

// WorklistV1 represents the worklist.json artifact
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

type Blueprint struct {
	ActionId    string
	Title       string
	PlaybookRef string
	Suggested   []string
}

func loadPreviousMetrics(baseDir, currentWeekID string) (string, *types.MetricsV1, error) {
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

	var m types.MetricsV1
	if err := json.Unmarshal(data, &m); err != nil {
		return prevID, nil, err
	}

	return prevID, &m, nil
}

func generateWeeklyArtifacts(dir, weekID, prevWeekID string, prevMetrics *types.MetricsV1, events []types.ReasonEventV1) error {
	// Read Current Metrics
	mPath := filepath.Join(dir, MetricsFilename)
	mData, err := os.ReadFile(mPath)
	if err != nil {
		return err
	}
	var m types.MetricsV1
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
	if err := os.WriteFile(filepath.Join(dir, "worklist.json"), append(wlData, '\n'), 0644); err != nil {
		return err
	}

	// Read Scorecard (if available)
	scScore := readScorecardFile(dir)
	
	// Phase 16 #M1: Signal Processing
	// 1. Normalize Signals
	commitSHA, _ := getGitSHA() // Best effort
	signals := signal.Normalize(events, weekID, commitSHA)
	
	// 2. Write Signals V1
	if err := signal.SaveSignalsV1(filepath.Join(dir, "signals_v1.json"), signals); err != nil {
		return fmt.Errorf("failed to save signals: %w", err)
	}

	// 3. Recurrence Detection
	ledgerPath := filepath.Join("docs/dogfood", "recurring_signals.json")
	ledger, err := signal.LoadLedger(ledgerPath)
	if err != nil {
		// If failed to load, log but don't crash main loop? Spec says "Storage".
		// We'll init new one if error (LoadLedger already handles NotExist)
		// If real error, warn.
		fmt.Printf("Warning: failed to load ledger: %v\n", err)
		ledger = &signal.Ledger{V: 1, Signals: []signal.Recurrence{}}
	}
	
	signal.DetectRecurrence(ledger, signals, weekID)
	
	if err := signal.SaveLedger(ledgerPath, ledger); err != nil {
		return fmt.Errorf("failed to save ledger: %w", err)
	}

	// 4. Stability Tracking (#M2)
	stabilityPath := filepath.Join("docs/dogfood", "stability_ledger.json")
	stabLedger, err := signal.LoadStabilityLedger(stabilityPath)
	if err != nil {
		fmt.Printf("Warning: failed to load stability ledger: %v\n", err)
		stabLedger = &signal.StabilityLedger{V: 1, Runs: []signal.RunEntry{}}
	}

	var prevSignals []signal.Signal
	if prevWeekID != "" {
		// Attempt to load previous signals
		prevDir := prevWeekID + "-Tokyo"
		// Assuming docs/dogfood structure. baseDir 'dir' is e.g. docs/dogfood/2025-W02-Tokyo
		// We can step up .. or construct absolute?
		// Better to use filepath.Dir(dir) to get docs/dogfood
		baseDogfood := filepath.Dir(dir)
		prevSigPath := filepath.Join(baseDogfood, prevDir, "signals_v1.json")
		
		if data, err := os.ReadFile(prevSigPath); err == nil {
			_ = json.Unmarshal(data, &prevSignals)
		}
	}

	signal.DetectStability(stabLedger, signals, prevSignals, weekID)

	if err := signal.SaveStabilityLedger(stabilityPath, stabLedger); err != nil {
		return fmt.Errorf("failed to save stability ledger: %w", err)
	}

	// Generate and Write Report
	reportMD := generateWeeklyReportMD(weekID, scScore, &m, prevMetrics, worklist, prevWeekID)
	return os.WriteFile(filepath.Join(dir, "report.md"), []byte(reportMD), 0644)
}

func generateWeeklyReportMD(weekID, scScore string, m, prevMetrics *types.MetricsV1, worklist *WorklistV1, prevWeekID string) string {
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

func calculateFailureDelta(m, prevMetrics *types.MetricsV1, prevWeekID string) (int, string) {
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

func generateWorklist(weekID string, current, prev *types.MetricsV1) (*WorklistV1, error) {
	hints := make(map[string]int)
	for k, v := range current.Metrics.CountsByHint {
		hints[k] = v
	}

	var items []WorklistItem
	for hintKey, count := range hints {
		items = append(items, scoreWorklistItem(hintKey, count, prev))
	}

	sortWorklistItems(items)

	for i := range items {
		items[i].Rank = i + 1
	}

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

func scoreWorklistItem(hintKey string, count int, prev *types.MetricsV1) WorklistItem {
	prevCount := 0
	if prev != nil {
		prevCount = prev.Metrics.CountsByHint[hintKey]
	}
	delta := count - prevCount

	wCount := 10
	wDelta := 25
	deltaScore := delta
	if deltaScore < 0 {
		deltaScore = 0
	}
	score := (count * wCount) + (deltaScore * wDelta)

	bp := getActionBlueprint(hintKey)

	if bp.ActionId == "" {
		bp.ActionId = "Z-UNMAPPED"
		bp.Title = fmt.Sprintf("Unmapped hint: %s", hintKey)
	}

	return WorklistItem{
		Rank:        0,
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
	parts := strings.Split(weekID, "-")
	if len(parts) < 2 {
		return "", fmt.Errorf("invalid weekID format")
	}

	year, err := strconv.Atoi(parts[0])
	if err != nil {
		return "", err
	}

	if len(parts[1]) < 2 || parts[1][0] != 'W' {
		return "", fmt.Errorf("invalid week part")
	}
	week, err := strconv.Atoi(parts[1][1:])
	if err != nil {
		return "", err
	}

	jan4 := time.Date(year, time.January, 4, 0, 0, 0, 0, time.UTC)
	
	wd := int(jan4.Weekday())
	if wd == 0 {
		wd = 7
	} 
	offset := wd - 1
	week1Mon := jan4.AddDate(0, 0, -offset)

	targetMon := week1Mon.AddDate(0, 0, (week-1)*7)

	loc := time.FixedZone(TimezoneTokyo, 9*60*60)
	y, m, d := targetMon.Date()
	mondayJST := time.Date(y, m, d, 0, 0, 0, 0, loc)

	return mondayJST.Format(time.RFC3339), nil
}

func getActionBlueprint(hint string) Blueprint {
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
		return Blueprint{ActionId: "", Title: ""} 
	}
}

func writeEvents(dir string, events []types.ReasonEventV1) error {
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

func generateMetricsV1(dir string, events []types.ReasonEventV1, weekID string) error {
	counts := make(map[string]int)
	hintCounts := make(map[string]int)

	for _, e := range events {
		counts[e.ReasonCode]++
		if strings.HasPrefix(e.Op, "dogfood.") { continue }
		for _, h := range e.HintCodes {
			hintCounts[h]++
		}
	}

	m := types.MetricsV1{
		V: 1,
		Metrics: types.MetricsBody{
			CountsByReason: counts,
			CountsByHint:   hintCounts,
		},
		Meta: types.MetaBody{
			Period:    weekID,
			Toolchain: "nix",
			Repo:      "github.com/" + DefaultRepoName,
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
	return os.WriteFile(filepath.Join(dir, MetricsFilename), append(data, '\n'), 0644)
}

func getGitSHA() (string, error) {
	if _, err := exec.LookPath("git"); err != nil {
		return "", fmt.Errorf("git executable not found in PATH: %w", err)
	}
	cmd := exec.Command("git", "rev-parse", "HEAD")
	out, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}
