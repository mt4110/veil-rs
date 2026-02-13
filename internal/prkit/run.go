package prkit

import (
	"fmt"
	"time"
)

func RunDryRun() (int, error) {
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
		// Ensure we still emit failure evidence JSON on Git SHA resolution errors
		if genErr := GenerateFailureEvidence(fmt.Errorf("cannot resolve git sha: %w", err)); genErr != nil {
			// If we cannot even generate failure evidence, propagate that as an internal error
			return 1, genErr
		}
		// Evidence was emitted with a failure status; signal failure via exit code
		return 2, nil
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
			return 2, err
		}
		return 2, nil
	}

	if err := evidence.PrintJSON(); err != nil {
		return 1, err
	}
	return 0, nil
}

func GenerateFailureEvidence(failureErr error) error {
	// Initialize evidence with basic defaults
	evidence := Evidence{
		SchemaVersion:  1,
		TimestampUTC:   time.Now().UTC().Format("20060102T150405Z"),
		Mode:           "dry-run",
		Status:         "FAIL",
		ExitCode:       2,
		ArtifactHashes: []string{},
	}

	// Try to collect Git SHA (best effort)
	if sha, err := getGitSHA(); err == nil {
		evidence.GitSHA = sha
	} else {
		evidence.GitSHA = "unknown"
	}

	// Collect Tool Versions (for consistency)
	evidence.ToolVersions = collectToolVersions()

	// Add the failure as a specific check result
	evidence.Checks = []CheckResult{
		{
			Name:    "cli_bootstrap",
			Status:  "FAIL",
			Details: failureErr.Error(),
		},
	}

	// Command list might be empty or partial in this case
	evidence.CommandList = []Command{}

	return evidence.PrintJSON()
}
