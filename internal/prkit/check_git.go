package prkit

import (
	"fmt"
	"os/exec"
	"strings"
)

func getGitSHA() (string, error) {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("git rev-parse HEAD failed: %v, output: %s", err, strings.TrimSpace(string(out)))
	}
	return strings.TrimSpace(string(out)), nil
}

func checkGitCleanWorktree() CheckResult {
	cmd := exec.Command("git", "status", "--porcelain=v1")
	out, err := cmd.CombinedOutput()

	result := CheckResult{
		Name: "git_clean_worktree",
	}

	output := strings.TrimSpace(string(out))
	if err != nil {
		result.Status = "FAIL"
		result.Details = fmt.Sprintf("failed to run git status: %v, output: %s", err, output)
		return result
	}
	if output == "" {
		result.Status = "PASS"
		result.Details = "worktree is clean"
	} else {
		result.Status = "FAIL"
		result.Details = output
	}

	return result
}
