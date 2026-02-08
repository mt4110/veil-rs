package main

import (
	"strings"
	"testing"
	"testing/fstest"
)

func TestValidateRegistry_Valid(t *testing.T) {
	reg := &Registry{
		Exceptions: []Exception{
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
	errs := validateRegistry(reg)
	if len(errs) != 0 {
		t.Errorf("Expected valid registry, got errors: %v", errs)
	}
}

func TestValidateRegistry_MissingFields(t *testing.T) {
	// Empty exception
	reg := &Registry{
		Exceptions: []Exception{{ID: ""}}, // Missing ID -> key "idx 0"
	}
	errs := validateRegistry(reg)
	if len(errs) == 0 {
		t.Error("Expected errors for missing fields, got none")
	}
	// Check for specific missing field errors
	foundRule := false
	for _, e := range errs {
		if e.Error() == "idx 0: missing rule" {
			foundRule = true
		}
	}
	if !foundRule {
		t.Errorf("Expected 'idx 0: missing rule' error, got: %v", errs)
	}
}

func TestValidateRegistry_DuplicateID(t *testing.T) {
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "DUP-01", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
			{ID: "DUP-01", Rule: "r", Scope: "path:b", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
		},
	}
	errs := validateRegistry(reg)
	foundDup := false
	for _, e := range errs {
		if e.Error() == "duplicate id: DUP-01" {
			foundDup = true
		}
	}
	if !foundDup {
		t.Error("Expected duplicate ID error")
	}
}

func TestValidateRegistry_InvalidScope(t *testing.T) {
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "BAD-SCOPE", Rule: "r", Scope: "invalid:format", Reason: "r", Owner: "o", CreatedAt: "2026-01-01", Audit: []string{"a"}},
		},
	}
	errs := validateRegistry(reg)
	foundScope := false
	expected := "BAD-SCOPE: invalid scope format 'invalid:format' (must start with path: or fingerprint:)"
	for _, e := range errs {
		if e.Error() == expected {
			foundScope = true
		}
	}
	if !foundScope {
		t.Errorf("Expected error '%s', got: %v", expected, errs)
	}
}

func TestValidateRegistry_InvalidDates(t *testing.T) {
	reg := &Registry{
		Exceptions: []Exception{
			{ID: "BAD-DATE", Rule: "r", Scope: "path:a", Reason: "r", Owner: "o", CreatedAt: "2026/01/01", Audit: []string{"a"}},
		},
	}
	errs := validateRegistry(reg)
	foundDate := false
	expected := "BAD-DATE: invalid created_at '2026/01/01' (must be YYYY-MM-DD)"
	for _, e := range errs {
		if e.Error() == expected {
			foundDate = true
		}
	}
	if !foundDate {
		t.Errorf("Expected error '%s', got: %v", expected, errs)
	}
}

func TestValidateRegistryFile_Missing(t *testing.T) {
	// Empty FS -> Missing file
	fsys := fstest.MapFS{}
	err := validateRegistryFile(fsys)
	if err == nil {
		t.Fatal("Expected error for missing file, got nil")
	}
	// Check error text/type if possible, or just existence of driftError
	if !strings.Contains(err.Error(), "ops/exceptions.toml is missing") {
		t.Errorf("Unexpected error message: %v", err)
	}
}

func TestValidateRegistryFile_Valid(t *testing.T) {
	content := `
[[exception]]
id = "TEST-FS"
rule = "r1"
scope = "path:*"
reason = "fs test"
owner = "@me"
created_at = "2026-02-08"
audit = ["link"]
`
	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: []byte(content)},
	}
	err := validateRegistryFile(fsys)
	if err != nil {
		t.Errorf("Expected valid registry, got error: %v", err)
	}
}

func TestValidateRegistryFile_InvalidTOML(t *testing.T) {
	content := `INVALID TOML`
	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: []byte(content)},
	}
	err := validateRegistryFile(fsys)
	if err == nil {
		t.Fatal("Expected error for invalid TOML")
	}
	if !strings.Contains(err.Error(), "TOML parse error") {
		t.Errorf("Unexpected error: %v", err)
	}
}
