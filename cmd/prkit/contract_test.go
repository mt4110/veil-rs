package main

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
	"time"

	"veil-rs/internal/prkit"
)

// ensure stable output for tests
func init() {
	// Mock time for deterministic JSON output in tests
	prkit.Now = func() time.Time {
		// 2024-01-01 12:00:00 UTC
		return time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC)
	}
}

func TestContractHelp(t *testing.T) {
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)
	args := []string{"--help"}

	exitCode := Run(args, stdout, stderr)

	if exitCode != 0 {
		t.Errorf("expected exit code 0, got %d", exitCode)
	}

	if stdout.Len() > 0 {
		t.Errorf("expected empty stdout for --help, got: %q", stdout.String())
	}

	usage := stderr.String()
	if !strings.Contains(usage, "Usage of prkit:") {
		t.Errorf("expected stderr to contain usage, got: %q", usage)
	}
}

func TestContractUnknownFlag(t *testing.T) {
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)
	args := []string{"--unknown-flag-for-contract-test"}

	exitCode := Run(args, stdout, stderr)

	if exitCode != 2 {
		t.Errorf("expected exit code 2, got %d", exitCode)
	}

	// Stdout must be valid portable-json
	var result map[string]interface{}
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Errorf("stdout is not valid JSON: %v. Content: %q", err, stdout.String())
	} else {
		// Check for specific error structure if needed, but at least ensure it's failure evidence
		if status, ok := result["status"]; !ok || status != "FAIL" {
			t.Errorf("expected status=FAIL in JSON, got %v", result)
		}
	}

	// Stderr must contain flag error AND usage
	errOutput := stderr.String()
	flagErrorMsg := "flag provided but not defined"
	if strings.Count(errOutput, flagErrorMsg) != 1 {
		t.Errorf("expected stderr to contain error EXACTLY ONCE, got count %d. Content:\n%s", strings.Count(errOutput, flagErrorMsg), errOutput)
	}
	if !strings.Contains(errOutput, "Usage of prkit:") {
		t.Errorf("expected stderr to contain usage, got: %q", errOutput)
	}
}

func TestContractSOTMissingArgs(t *testing.T) {
	// SOT mode requires epic/slug. Test that failure is clean/deterministic.
	stdout := new(bytes.Buffer)
	stderr := new(bytes.Buffer)
	args := []string{"--sot-new"} // Missing epic/slug

	exitCode := Run(args, stdout, stderr)

	if exitCode != 2 {
		t.Errorf("expected exit code 2 for missing args, got %d", exitCode)
	}

	// Check JSON output (audit log)
	var result map[string]interface{}
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Errorf("stdout is not valid JSON: %v", err)
	}

	// Check human output
	if !strings.Contains(stderr.String(), "Error: --sot-new requires --epic and --slug") {
		t.Errorf("expected stderr to contain specific error, got: %q", stderr.String())
	}
}
