package main

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
	"testing/fstest"
	"time"
)

func TestExceptionsList_Golden(t *testing.T) {
	// Mock FS with all status types
	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: []byte(`
[[exception]]
id = "ACTIVE-01"
rule = "r"
scope = "s"
reason = "Active exception"
owner = "o"
created_at = "2026-01-01"
audit = ["a"]
expires_at = "2026-03-01"

[[exception]]
id = "SOON-01"
rule = "r"
scope = "s"
reason = "Expiring soon"
owner = "o"
created_at = "2026-01-01"
audit = ["a"]
expires_at = "2026-02-10"

[[exception]]
id = "EXPIRED-01"
rule = "r"
scope = "s"
reason = "Expired exception"
owner = "o"
created_at = "2026-01-01"
audit = ["a"]
expires_at = "2026-02-07"

[[exception]]
id = "PERPETUAL-01"
rule = "r"
scope = "s"
reason = "Perpetual exception"
owner = "o"
created_at = "2026-01-01"
audit = ["a"]
`)},
	}

	// Fixed Now: 2026-02-08
	now := time.Date(2026, 2, 8, 0, 0, 0, 0, time.UTC)
	
	// 1. Table Output Check
	{
		var stdout, stderr bytes.Buffer
		ctx := &AppContext{Stdout: &stdout, Stderr: &stderr, FS: fsys, Now: now}
		if code := runExceptionsList(ctx, []string{}); code != 0 {
			t.Fatalf("Table list failed: %v", stderr.String())
		}
		output := stdout.String()
		
		// Verifying full exact lines for key rows
		// Note: The order should be ACTIVE, EXPIRED, PERPETUAL, SOON (Alphabetical ID)
		// Expected IDs: ACTIVE-01, EXPIRED-01, PERPETUAL-01, SOON-01
		
		lines := strings.Split(strings.TrimSpace(output), "\n")
		// Header + 4 rows = 5 lines
		if len(lines) < 5 {
			t.Fatalf("Expected at least 5 lines, got %d", len(lines))
		}
		
		// Check Header
		if !strings.Contains(lines[0], "ID") || !strings.Contains(lines[0], "STATUS") {
			t.Errorf("Header mismatch: %s", lines[0])
		}
		
		// Check Sorted Order (ID)
		expectedIDs := []string{"ACTIVE-01", "EXPIRED-01", "PERPETUAL-01", "SOON-01"}
		for i, expected := range expectedIDs {
			line := lines[i+1] // skip header
			if !strings.HasPrefix(line, expected) {
				t.Errorf("Row %d: expected ID %s..., got %s", i, expected, line)
			}
		}
	}

	// 2. JSON Output Check
	{
		var stdout, stderr bytes.Buffer
		ctx := &AppContext{Stdout: &stdout, Stderr: &stderr, FS: fsys, Now: now}
		if code := runExceptionsList(ctx, []string{"--format", "json"}); code != 0 {
			t.Fatalf("JSON list failed: %v", stderr.String())
		}
		
		output := stdout.String()
		type jsonEntry struct {
			ID      string `json:"id"`
			Status  string `json:"status"`
			Expires string `json:"expires"`
			Owner   string `json:"owner"`
			Reason  string `json:"reason"`
		}
		var entries []jsonEntry
		if err := json.Unmarshal([]byte(output), &entries); err != nil {
			t.Fatalf("Failed to unmarshal JSON: %v\nOutput: %s", err, output)
		}
		
		if len(entries) != 4 {
			t.Fatalf("Expected 4 entries in JSON, got %d", len(entries))
		}
		
		// Check Order & Content
		expectedIDs := []string{"ACTIVE-01", "EXPIRED-01", "PERPETUAL-01", "SOON-01"}
		for i, e := range entries {
			if e.ID != expectedIDs[i] {
				t.Errorf("JSON index %d: expected ID %s, got %s", i, expectedIDs[i], e.ID)
			}
			// Check required keys presence (implicit in struct unmarshal, checking values)
			if e.Status == "" || e.Owner == "" || e.Reason == "" {
				t.Errorf("JSON entry %s missing fields: %+v", e.ID, e)
			}
		}
		
		// Check specific fields correctness
		if entries[0].Status != "active" { t.Errorf("Active status mismatch: %s", entries[0].Status) }
		if entries[1].Status != "expired" { t.Errorf("Expired status mismatch: %s", entries[1].Status) }
		if entries[2].Expires != "" { t.Errorf("Perpetual expires should be empty, got %s", entries[2].Expires) }
	}

	// 3. Status Filter (Invalid)
	{
		var stdout, stderr bytes.Buffer
		ctx := &AppContext{Stdout: &stdout, Stderr: &stderr, FS: fsys, Now: now}
		if code := runExceptionsList(ctx, []string{"--status", "invalid"}); code != 1 {
			t.Errorf("Expected exit code 1 for invalid status, got %d", code)
		}
	}
}

func TestExceptionsShow_Golden(t *testing.T) {
	fsys := fstest.MapFS{
		"ops/exceptions.toml": {Data: []byte(`
[[exception]]
id = "TEST-01"
rule = "rule-1"
scope = "path:src/**"
reason = "Test Reason"
owner = "@owner"
created_at = "2026-01-01"
audit = ["http://audit"]
`)},
	}
	now := time.Date(2026, 2, 8, 0, 0, 0, 0, time.UTC)
	
	var stdout, stderr bytes.Buffer
	ctx := &AppContext{
		Stdout: &stdout,
		Stderr: &stderr,
		FS:     fsys,
		Now:    now,
	}

	code := runExceptionsShow(ctx, "TEST-01")
	if code != 0 {
		t.Fatalf("Show failed. Stderr: %s", stderr.String())
	}

	output := stdout.String()
	expected := []string{
		"ID:        TEST-01",
		"Status:    active",
		"Rule:      rule-1",
		"Scope:     path:src/**",
		"Created:   2026-01-01",
		"Owner:     @owner",
		"Reason:    Test Reason",
		"Audit:",
		"  - http://audit",
	}

	for _, line := range expected {
		if !strings.Contains(output, line) {
			t.Errorf("Output missing line: %q", line)
		}
	}
}
