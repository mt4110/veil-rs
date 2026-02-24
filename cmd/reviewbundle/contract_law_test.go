package main

import (
	"bytes"
	"strings"
	"testing"
)

// TestVerify_StoplessOutput verifies that the verify subcommand emits
// OK: phase=end stop=<0|1> and never calls os.Exit.
// This is the S12-06B contract: verify CLI is stopless.
func TestVerify_StoplessOutput(t *testing.T) {
	t.Run("missing_path_arg", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		rc := run([]string{"reviewbundle", "verify"}, &stdout, &stderr)
		if rc != 0 {
			t.Errorf("expected rc=0 (stopless), got %d", rc)
		}
		out := stdout.String()
		if !strings.Contains(out, "OK: phase=end stop=1") {
			t.Errorf("expected OK: phase=end stop=1 in stdout, got:\n%s", out)
		}
		if !strings.Contains(out, "ERROR:") {
			t.Errorf("expected ERROR: in stdout, got:\n%s", out)
		}
	})

	t.Run("nonexistent_bundle", func(t *testing.T) {
		var stdout, stderr bytes.Buffer
		rc := run([]string{"reviewbundle", "verify", "/nonexistent/bundle.tar.gz"}, &stdout, &stderr)
		if rc != 0 {
			t.Errorf("expected rc=0 (stopless), got %d", rc)
		}
		out := stdout.String()
		if !strings.Contains(out, "OK: phase=end stop=1") {
			t.Errorf("expected OK: phase=end stop=1 in stdout, got:\n%s", out)
		}
		if !strings.Contains(out, "ERROR: verify_failed") {
			t.Errorf("expected ERROR: verify_failed in stdout, got:\n%s", out)
		}
	})
}

// TestVerify_ContractV11Compat verifies that a bundle produced with
// ContractVersion "1.1" is accepted by the current verifier.
// This is the S12-06B backward-compatibility test.
// It uses testdata/manifest fields only — no tar.gz stored in repo.
func TestVerify_ContractV11Compat(t *testing.T) {
	// Verify that ValidateContractV11 accepts "1.1" and rejects other versions.
	cases := []struct {
		version string
		wantErr bool
	}{
		{"1.1", false},
		{"1.0", true},
		{"2.0", true},
		{"", true},
	}
	for _, tc := range cases {
		c := &Contract{
			ContractVersion: tc.version,
			Mode:            "wip",
			EpochSec:        1700000000,
			HeadSHA:         strings.Repeat("a", 40),
		}
		err := ValidateContractV11(c)
		if tc.wantErr && err == nil {
			t.Errorf("version %q: expected error, got nil", tc.version)
		}
		if !tc.wantErr && err != nil {
			t.Errorf("version %q: expected no error, got %v", tc.version, err)
		}
	}
}

// TestVerify_ContractVersionSingleSource verifies that contract_version
// is read from contract.json only (single source of truth per S12-06B).
func TestVerify_ContractVersionSingleSource(t *testing.T) {
	// ParseContractJSON must extract contract_version from JSON.
	jsonBytes := []byte(`{
		"contract_version": "1.1",
		"mode": "wip",
		"epoch_sec": 1700000000,
		"head_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
		"base_ref": "main",
		"repo": "veil-rs",
		"warnings_count": 0,
		"evidence": {"required": false, "present": false, "bound_to_head": false, "path_prefix": "review/evidence/"},
		"tool": {"name": "reviewbundle", "version": "1.0.0"}
	}`)
	c, err := ParseContractJSON(jsonBytes)
	if err != nil {
		t.Fatalf("ParseContractJSON failed: %v", err)
	}
	if c.ContractVersion != "1.1" {
		t.Errorf("expected ContractVersion=1.1, got %q", c.ContractVersion)
	}
	// Validate passes
	if err := ValidateContractV11(c); err != nil {
		t.Errorf("ValidateContractV11 failed for valid contract: %v", err)
	}
}
