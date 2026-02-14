package prkit

import (
	"context"
	"fmt"

	"strings"
)

func getGitSHA() (string, error) {
	spec := ExecSpec{
		Argv: []string{"git", "rev-parse", "HEAD"},
	}
	res := Runner.Run(context.Background(), spec)

	if res.ExitCode != 0 {
		return "", fmt.Errorf("git rev-parse HEAD failed: %s, stderr: %s", res.ErrorKind, strings.TrimSpace(res.Stderr))
	}
	return strings.TrimSpace(res.Stdout), nil
}

func checkGitCleanWorktree() CheckResult {
	spec := ExecSpec{
		Argv: []string{"git", "status", "--porcelain=v1"},
	}
	res := Runner.Run(context.Background(), spec)

	result := CheckResult{
		Name: "git_clean_worktree",
	}

	if res.ExitCode != 0 {
		result.Status = "FAIL"
		result.Details = fmt.Sprintf("failed to run git status: %s, stderr: %s", res.ErrorKind, strings.TrimSpace(res.Stderr))
		return result
	}

	output := strings.TrimSpace(res.Stdout)
	if output == "" {
		result.Status = "PASS"
		result.Details = "worktree is clean"
	} else {
		result.Status = "FAIL"
		result.Details = output
	}

	return result
}
