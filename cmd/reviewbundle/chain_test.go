package main

import (
	"bytes"
	"strings"
	"testing"
)

// S12-06A: verify chain hardening tests
//
// These tests verify the STDOUT contract invariants across the reviewbundle CLI:
//   - Any failure yields ERROR: + stop=1 (never stop=0 on failure)
//   - Process always exits 0 (stopless)
//   - OK: phase=end stop=<0|1> is always the final line
//
// hermetic: no nix, no cargo, no network required.

// TestChain_ViolationYieldsStop1 verifies that a verify violation
// (nonexistent or corrupt bundle) yields stop=1 and ERROR: on stdout.
// This is the core S12-06A chain test: intentional violation → ERROR: + stop=1.
func TestChainViolationYieldsStop1(t *testing.T) {
	cases := []struct {
		name        string
		args        []string
		wantError   bool
		wantStop    string
		wantErrLine string
	}{
		{
			name:        "nonexistent_bundle",
			args:        []string{"verify", "/nonexistent/does-not-exist.tar.gz"},
			wantError:   true,
			wantStop:    "stop=1",
			wantErrLine: "ERROR: verify_failed",
		},
		{
			name:        "missing_bundle_arg",
			args:        []string{"verify"},
			wantError:   true,
			wantStop:    "stop=1",
			wantErrLine: "ERROR:",
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			var stdout, stderr bytes.Buffer
			rc := run(append([]string{"reviewbundle"}, tc.args...), &stdout, &stderr)

			// INVARIANT 1: process always exits 0 (stopless)
			if rc != 0 {
				t.Errorf("expected rc=0 (stopless), got %d", rc)
			}

			out := stdout.String()

			// INVARIANT 2: OK: phase=end must appear (last line)
			if !strings.Contains(out, "OK: phase=end") {
				t.Errorf("missing OK: phase=end in stdout:\n%s", out)
			}

			// INVARIANT 3: violation must yield stop=1
			if tc.wantError && !strings.Contains(out, tc.wantStop) {
				t.Errorf("expected %q for violation, got stdout:\n%s", tc.wantStop, out)
			}

			// INVARIANT 4: ERROR: prefix on stdout (machine-readable)
			if tc.wantErrLine != "" && !strings.Contains(out, tc.wantErrLine) {
				t.Errorf("expected %q in stdout, got:\n%s", tc.wantErrLine, out)
			}
		})
	}
}

// TestChainStopValuesMutuallyExclusive verifies that stop=0 and stop=1
// never appear in the same output (mutual exclusivity invariant).
func TestChainStopValuesMutuallyExclusive(t *testing.T) {
	// We can't easily create a valid bundle in a unit test,
	// but we can verify that the run() with "create" help path exits 0.
	// For the stop=0 path, we rely on contract_law_test.go TestVerify_StoplessOutput.
	// Here we verify the invariant: stop=0 and stop=1 are mutually exclusive.
	t.Run("stop_values_mutually_exclusive", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		run([]string{"reviewbundle", "verify", "/nonexistent"}, &stdout, &stderr)
		out := stdout.String()

		hasStop0 := strings.Contains(out, "stop=0")
		hasStop1 := strings.Contains(out, "stop=1")

		if hasStop0 && hasStop1 {
			t.Errorf("stdout contains both stop=0 and stop=1 (ambiguous):\n%s", out)
		}
		if !hasStop0 && !hasStop1 {
			t.Errorf("stdout contains neither stop=0 nor stop=1:\n%s", out)
		}
	})
}

// TestChainPhaseEndAlwaysLast verifies that OK: phase=end appears and
// that no machine-readable output follows it (it must be last).
func TestChainPhaseEndAlwaysLast(t *testing.T) {
	cases := [][]string{
		{"reviewbundle", "verify"},
		{"reviewbundle", "verify", "/nonexistent"},
		{"reviewbundle", "bad-unknown-command"},
	}
	for _, args := range cases {
		var stdout, stderr bytes.Buffer
		run(args, &stdout, &stderr)
		out := stdout.String()
		lines := strings.Split(strings.TrimRight(out, "\n"), "\n")
		if len(lines) == 0 {
			t.Errorf("empty stdout for args %v", args)
			continue
		}
		last := lines[len(lines)-1]
		if !strings.HasPrefix(last, "OK: phase=end stop=") {
			t.Errorf("last line must be OK: phase=end stop=, got %q\nfull stdout:\n%s", last, out)
		}
	}
}
