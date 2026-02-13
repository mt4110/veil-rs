package prkit

import (
	"fmt"
	"os"
	"path/filepath"
	"time"
)

func RunDryRun() (int, error) {
	evidence, err := collectEvidence("dry-run")
	if err != nil {
		return 2, err
	}

	if err := evidence.PrintJSON(); err != nil {
		return 1, err
	}

	if evidence.Status == "FAIL" {
		return 2, nil
	}
	return 0, nil
}

func RunExecuteMode(outPath string, reviewBundle bool) (int, error) {
	evidence, err := collectEvidence("run")
	if err != nil {
		return 2, err
	}

	if reviewBundle {
		bundleInfo, err := generateReviewBundle()
		if err != nil {
			// If review bundle fails, we should probably fail the whole run?
			// The plan says: "error 'review_bundle script missing => STOP'"
			// So yes, fail.
			return 2, fmt.Errorf("failed to generate review bundle: %w", err)
		}
		evidence.ArtifactHashes = append(evidence.ArtifactHashes, bundleInfo)
	}

	if outPath != "" {
		// Validate directory existence
		dir := filepath.Dir(outPath)
		if _, err := os.Stat(dir); os.IsNotExist(err) {
			return 2, fmt.Errorf("output directory does not exist: %s", dir)
		}

		if err := evidence.WriteJSON(outPath); err != nil {
			return 1, fmt.Errorf("failed to write evidence to %s: %w", outPath, err)
		}
	} else {
		if err := evidence.PrintJSON(); err != nil {
			return 1, err
		}
	}

	if evidence.Status == "FAIL" {
		return 2, nil
	}
	return 0, nil
}

func collectEvidence(mode string) (*Evidence, error) {
	// Initialize evidence
	evidence := Evidence{
		SchemaVersion:  1,
		TimestampUTC:   time.Now().UTC().Format("20060102T150405Z"),
		Mode:           mode,
		Status:         "PASS",
		ExitCode:       0,
		ArtifactHashes: []string{},
	}

	// Collect Git SHA
	sha, err := getGitSHA()
	if err != nil {
		// Ensure we still emit failure evidence JSON on Git SHA resolution errors
		evidence.GitSHA = "unknown"
		// We could try to return partial evidence, but for now we follow the existing pattern
		// of failing the execution if we can't contextually bind to a commit.
		return nil, fmt.Errorf("cannot resolve git sha: %w", err)
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
	}

	return &evidence, nil
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
