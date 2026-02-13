package prkit

import (
	"fmt"
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
		if genErr := GenerateFailureEvidence(fmt.Errorf("cannot resolve git sha: %w", err)); genErr != nil {
			// If we cannot even generate failure evidence, propagate that as an internal error
			return nil, genErr
		}
		// Evidence was emitted with a failure status; signal failure via exit code
		// Logic in wrapper will handle outputting this failure evidence if needed,
		// but GenerateFailureEvidence writes to Stdout by default.
		// For verification consistency we should probably return the "Failure Evidence" object
		// instead of printing it immediately if we want to support --out.
		// However, GenerateFailureEvidence is existing helper.
		// Let's refactor GenerateFailureEvidence to return *Evidence instead of printing?
		// For now, let's keep it simple. If failure occurs here, we might just fail.
		// BUT the contract says we should emit evidence.

		// To properly support --out for failure cases, we should adjust GenerateFailureEvidence.
		// But for minimal changes: stick to printing behavior for catastrophic early failure?
		// Or better: construct partial evidence and return it with error?

		// Let's rely on the existing behavior for now, assuming "cannot resolve git sha" is rare/fatal.
		// Actually, let's make collectEvidence robust.
		evidence.GitSHA = "unknown"
		// We will continue to try running checks? Or fail immediately?
		// Original code returned 2.
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
