package cockpit

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

type MetricsV1 struct {
	Version       string `json:"version"` // "metrics_v1"
	Period        string `json:"period"`  // "YYYY-Www"
	ScannedFiles  int    `json:"scanned_files"`
	FindingsTotal int    `json:"findings_total"`
	MaskedTotal   int    `json:"masked_total"`
	IgnoredTotal  int    `json:"ignored_total"`
	DurationMs    int64  `json:"duration_ms"` // 0 if not measured
	CacheHit      int    `json:"cache_hit"`
	CacheMiss     int    `json:"cache_miss"`
	Repo          string `json:"repo,omitempty"`
	GitCommit     string `json:"git_commit,omitempty"`
	Toolchain     string `json:"toolchain,omitempty"`
}

type ScorecardResult struct {
	Score float64 `json:"score"`
}

func Dogfood() (string, error) {
	// 1. Determine Week/Time
	loc, err := time.LoadLocation("Asia/Tokyo")
	if err != nil {
		// Fallback to Fixed Zone if system database is missing (e.g. some minimal containers)
		loc = time.FixedZone("Asia/Tokyo", 9*60*60)
	}
	now := time.Now().In(loc)
	y, w := now.ISOWeek()

	dirName := fmt.Sprintf("%04d-W%02d-Tokyo", y, w)
	outDir := filepath.Join("docs", "dogfood", dirName)

	if err := os.MkdirAll(outDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create dir %s: %w", outDir, err)
	}

	// 2. Metrics
	if err := generateMetrics(outDir, y, w); err != nil {
		return "", fmt.Errorf("metrics generation failed: %w", err)
	}

	// 3. Scorecard
	if err := generateScorecard(outDir); err != nil {
		return "", fmt.Errorf("scorecard generation failed: %w", err)
	}

	// 4. Weekly Markdown
	if err := generateWeekly(outDir, y, w); err != nil {
		return "", fmt.Errorf("weekly.md generation failed: %w", err)
	}

	return outDir, nil
}

func generateMetrics(dir string, y, w int) error {
	m := MetricsV1{
		Version:       "metrics_v1",
		Period:        fmt.Sprintf("%04d-W%02d", y, w),
		ScannedFiles:  0, // Placeholder
		FindingsTotal: 0, // Placeholder
		MaskedTotal:   0, // Placeholder
		IgnoredTotal:  0, // Placeholder
		DurationMs:    0, // Placeholder
		CacheHit:      0,
		CacheMiss:     0,
		Toolchain:     "nix",
	}

	if sha, err := getGitSHA(); err == nil {
		m.GitCommit = sha
	}
	// Try to get repo from env
	if r := os.Getenv("GITHUB_REPOSITORY"); r != "" {
		m.Repo = "github.com/" + r
	} else {
		// Reasonable default or empty
		m.Repo = "github.com/mt4110/veil-rs"
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
	// Require repo name.
	repo := os.Getenv("GITHUB_REPOSITORY")
	if repo == "" {
		repo = "mt4110/veil-rs" // Default for local if not set
	}
	repoURL := "github.com/" + repo

	// Run scorecard CLI
	// Requires GITHUB_AUTH_TOKEN explicitly from requirements
	cmd := exec.Command("scorecard", "--repo="+repoURL, "--format=json")

	// Pass GITHUB_AUTH_TOKEN if setup, otherwise trust env
	if token := os.Getenv("GITHUB_AUTH_TOKEN"); token != "" {
		cmd.Env = append(os.Environ(), "GITHUB_AUTH_TOKEN="+token)
	} else if token := os.Getenv("GITHUB_TOKEN"); token != "" {
		// Fallback compat
		cmd.Env = append(os.Environ(), "GITHUB_AUTH_TOKEN="+token)
	}

	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("scorecard cli failed: %v\nOutput:\n%s", err, string(out))
	}

	// Robust JSON parsing
	var raw map[string]interface{}
	if err := json.Unmarshal(out, &raw); err != nil {
		return fmt.Errorf("failed to parse scorecard json: %w", err)
	}

	var scoreVal float64
	found := false

	// Strategy: look for 'score' float, or 'aggregateScore' which might be object or float
	if v, ok := raw["score"]; ok {
		if f, ok := v.(float64); ok {
			scoreVal = f
			found = true
		}
	}

	if !found {
		// Try aggregateScore.score or just aggregateScore
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
		return fmt.Errorf("could not find 'score' or 'aggregateScore.score' in scorecard output")
	}

	// Format output
	score100 := int(scoreVal*10 + 0.5) // round

	lines := []string{
		fmt.Sprintf("scorecard_score_0_100: %d", score100),
		fmt.Sprintf("scorecard_score_0_10: %.1f", scoreVal),
		fmt.Sprintf("scorecard_repo: %s", repoURL),
		"scorecard_source: ossf/scorecard CLI",
		"", // Trailing newline
	}

	content := strings.Join(lines, "\n")
	return os.WriteFile(filepath.Join(dir, "scorecard.txt"), []byte(content), 0644)
}

func generateWeekly(dir string, year, week int) error {
	content := fmt.Sprintf(`# Weekly dogfood %04d-W%02d (Tokyo)

- metrics_v1.json generated
- scorecard.txt generated
- next: review deltas / tune thresholds
`, year, week)

	return os.WriteFile(filepath.Join(dir, "weekly.md"), []byte(content), 0644)
}
