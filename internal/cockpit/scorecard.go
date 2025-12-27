package cockpit

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

func generateScorecard(dir string) error {
	repo := os.Getenv("GITHUB_REPOSITORY")
	if repo == "" {
		repo = DefaultRepoName
	}
	repoURL := "https://github.com/" + repo
	
	// Security: Validate repo format to prevent command injection
	if strings.ContainsAny(repo, " ;&|`$()<>") {
		return fmt.Errorf("invalid repository name: %q", repo)
	}

	// Timeout: ensure scorecard never hangs forever
	timeout := 10 * time.Minute
	if os.Getenv("CI") != "" || os.Getenv("GITHUB_ACTIONS") == "true" {
		timeout = 4 * time.Minute
	}
	if v := os.Getenv("VEIL_SCORECARD_TIMEOUT"); v != "" {
		if d, err := time.ParseDuration(v); err == nil && d > 0 {
			timeout = d
		}
	}
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	cmd := exec.CommandContext(ctx, "scorecard", "--repo="+repoURL, "--format=json")

	// Prevent “waiting for credential prompt” hangs
	env := append(os.Environ(),
		"GIT_TERMINAL_PROMPT=0",
		"GIT_ASKPASS=/usr/bin/false",
		"SSH_ASKPASS=/usr/bin/false",
	)
	if token := os.Getenv("GITHUB_AUTH_TOKEN"); token != "" {
		env = append(env, "GITHUB_AUTH_TOKEN="+token)
	} else if token := os.Getenv("GITHUB_TOKEN"); token != "" {
		env = append(env, "GITHUB_AUTH_TOKEN="+token)
	}
	cmd.Env = env

	out, err := cmd.CombinedOutput()
	if ctx.Err() == context.DeadlineExceeded {
		return fmt.Errorf("scorecard timed out after %s (repo=%s)", timeout, repoURL)
	}
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
