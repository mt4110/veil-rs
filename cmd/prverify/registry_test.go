package main

import (
	"fmt"
	"strings"
	"testing"
	"testing/fstest"
	"time"

	"veil-rs/internal/registry"
)

func TestValidateRegistry_Valid(t *testing.T) {
	reg := &registry.Registry{
		Exceptions: []registry.Exception{
			{
				ID:        "EX-20260208-001",
				Rule:      "rule-1",
				Scope:     "path:src/**",
				Reason:    "Valid reason",
				Owner:     "@owner",
				CreatedAt: "2026-02-08",
				Audit:     []string{"http://audit"},
			},
		},
	}
	today := parseDate(t, "2026-02-08")
	errs := registry.Validate(reg, today)
	if len(errs) != 0 {
		t.Errorf("Expected valid registry, got errors: %v", errs)
	}
}

func TestValidateRegistry_GoldenOutput(t *testing.T) {
	// Verify formatting of driftError in validateRegistryFile
	today := parseDate(t, "2026-02-08")

	// Create 12 expired entries
	var exceptions []registry.Exception
	for i := 11; i >= 0; i-- {
		id := fmt.Sprintf("EX-%02d", i)
		exceptions = append(exceptions, registry.Exception{
			ID:        id,
			Rule:      "r",
			Scope:     "path:a",
			Reason:    "r",
			Owner:     "o",
			CreatedAt: "2026-01-01",
			Audit:     []string{"a"},
			ExpiresAt: "2026-02-07",
		})
	}

	// Mock FS
	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: generateTOML(exceptions)},
	}

	err := validateRegistryFile(fsys, today)
	if err == nil {
		t.Fatal("Expected error from validateRegistryFile")
	}

	driftErr, ok := err.(*driftError)
	if !ok {
		t.Fatalf("Expected *driftError, got %T", err)
	}

	// 1. Header
	if !strings.Contains(driftErr.reason, "Registry validation failed (12 errors): (utc_today=2026-02-08)") {
		t.Errorf("Header mismatch. Got:\n%s", driftErr.reason)
	}

	// 2. Body (Top 10 sorted)
	for i := 0; i < 10; i++ {
		key := fmt.Sprintf("EX-%02d", i)
		if !strings.Contains(driftErr.reason, "- "+key) {
			t.Errorf("Missing expected item %s in output", key)
		}
	}

	// 3. Capping (EX-10, EX-11 hidden)
	if strings.Contains(driftErr.reason, "EX-10") || strings.Contains(driftErr.reason, "EX-11") {
		t.Errorf("Output should not contain capped items EX-10 or EX-11")
	}

	// 4. "and N more"
	if !strings.Contains(driftErr.reason, "... and 2 more") {
		t.Errorf("Missing capping message '... and 2 more'")
	}
	
	// 5. Check FixCmd (New UX)
	expectedFix := "Renew expiry or remove exception (see runbook: docs/runbook/exception-registry.md)"
	if driftErr.fixCmd != expectedFix {
		t.Errorf("FixCmd mismatch.\nExpected: %s\nGot:      %s", expectedFix, driftErr.fixCmd)
	}

	// 6. Check NextCmd (New UX - Contextual)
	expectedNext := "nix run .#veil -- exceptions list --status expired"
	if driftErr.nextCmd != expectedNext {
		t.Errorf("NextCmd mismatch.\nExpected: %s\nGot:      %s", expectedNext, driftErr.nextCmd)
	}
}

func TestValidateRegistry_ValidationFailure(t *testing.T) {
	// Verify non-expiry validation failure
	today := parseDate(t, "2026-02-08")
	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: []byte(`
[[exception]]
id = "MISSING-FIELD-01"
# missing rule, scope, etc.
`)},
	}

	err := validateRegistryFile(fsys, today)
	if err == nil {
		t.Fatal("Expected error")
	}

	driftErr, ok := err.(*driftError)
	if !ok {
		t.Fatalf("Expected *driftError, got %T", err)
	}

	// Ensure NextCmd is generic list
	expectedNext := "nix run .#veil -- exceptions list"
	if driftErr.nextCmd != expectedNext {
		t.Errorf("NextCmd mismatch for validation error.\nExpected: %s\nGot:      %s", expectedNext, driftErr.nextCmd)
	}
	
	// Ensure FixCmd is generic fix
	if strings.Contains(driftErr.fixCmd, "Renew expiry") {
		t.Errorf("FixCmd should not suggest key renewal for validation error. Got: %s", driftErr.fixCmd)
	}
}

func TestValidateRegistry_MissingFile(t *testing.T) {
	today := parseDate(t, "2026-02-08")
	fsys := fstest.MapFS{} // Empty FS

	err := validateRegistryFile(fsys, today)
	if err == nil {
		t.Fatal("Expected error")
	}
	
	driftErr, ok := err.(*driftError)
	if !ok {
		t.Fatalf("Expected *driftError, got %T", err)
	}

	expectedNext := "nix run .#veil -- exceptions list"
	if driftErr.nextCmd != expectedNext {
		t.Errorf("NextCmd mismatch for missing file.\nExpected: %s\nGot:      %s", expectedNext, driftErr.nextCmd)
	}
}

// Helper to generate TOML content for MapFS
func generateTOML(exs []registry.Exception) []byte {
	var sb strings.Builder
	for _, e := range exs {
		sb.WriteString("[[exception]]\n")
		sb.WriteString(fmt.Sprintf("id = \"%s\"\n", e.ID))
		sb.WriteString(fmt.Sprintf("rule = \"%s\"\n", e.Rule))
		sb.WriteString(fmt.Sprintf("scope = \"%s\"\n", e.Scope))
		sb.WriteString(fmt.Sprintf("reason = \"%s\"\n", e.Reason))
		sb.WriteString(fmt.Sprintf("owner = \"%s\"\n", e.Owner))
		sb.WriteString(fmt.Sprintf("created_at = \"%s\"\n", e.CreatedAt))
		sb.WriteString("audit = [\"a\"]\n")
		if e.ExpiresAt != "" {
			sb.WriteString(fmt.Sprintf("expires_at = \"%s\"\n", e.ExpiresAt))
		}
		sb.WriteString("\n")
	}
	return []byte(sb.String())
}

const (
	dateFmt = "2006-01-02"
)

func parseDate(t *testing.T, s string) time.Time {
	d, err := time.Parse(dateFmt, s)
	if err != nil {
		t.Fatalf("failed to parse date %s: %v", s, err)
	}
	return d
}
