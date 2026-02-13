package prkit

import (
	"fmt"
	"os"
	"time"
)

func RunDryRun() error {
	// Initialize evidence
	evidence := Evidence{
		SchemaVersion:  1,
		TimestampUTC:   time.Now().UTC().Format("20060102T150405Z"),
		Mode:           "dry-run",
		Status:         "PASS",
		ExitCode:       0,
		ArtifactHashes: []string{}, // dry-run so empty
	}

	// Collect Git SHA
	sha, err := getGitSHA()
	if err != nil {
		return fmt.Errorf("cannot resolve git sha: %w", err)
	}
	evidence.GitSHA = sha

	// Collect Tool Versions
	evidence.ToolVersions = collectToolVersions()

	// Define Command List (v1 contract)
	evidence.CommandList = []Command{
		{Name: "git_status_porcelain", Cmd: "git status --porcelain=v1"},
	}

	// Run Checks
	// 1. git_clean_worktree
	cleanCheck := checkGitCleanWorktree()
	evidence.Checks = append(evidence.Checks, cleanCheck)

	if cleanCheck.Status == "FAIL" {
		evidence.Status = "FAIL"
		evidence.ExitCode = 2
		if err := evidence.PrintJSON(); err != nil {
			return err
		}
		os.Exit(2)
	}

	return evidence.PrintJSON()
}
