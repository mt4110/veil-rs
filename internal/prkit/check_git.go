package prkit

import (
	"fmt"
	"os/exec"
	"strings"
)

func getGitSHA() (string, error) {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	out, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(out)), nil
}

func checkGitCleanWorktree() CheckResult {
	cmd := exec.Command("git", "status", "--porcelain=v1")
	out, err := cmd.CombinedOutput()

	result := CheckResult{
		Name: "git_clean_worktree",
	}

	if err != nil {
		result.Status = "FAIL"
		result.Details = fmt.Sprintf("failed to run git status: %v, output: %s", err, string(out))
		return result
	}

	output := strings.TrimSpace(string(out))
	if output == "" {
		result.Status = "PASS"
		result.Details = "worktree is clean"
	} else {
		result.Status = "FAIL"
		result.Details = output
	}

	return result
}
