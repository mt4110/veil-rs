package ux_test

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"
	"veil-rs/internal/ux"
)

func TestRFv1Output(t *testing.T) {
	// 1. Setup deterministic input
	out := ux.Output{
		Step:   "check",
		Status: "FAIL",
	}
	// Add unordered items to verify sorting
	out.Add("GIT_MISSING", "git is required but not found", "Install git, or enter nix develop (git is runtimeInput)", "docs/ai/SSOT.md#git-required")
	out.Add("DIRTY_TREE", "uncommitted changes found", "Commit your changes or use a clean checkout", "docs/ai/SSOT.md#clean-tree")

	// 2. Capture output
	var buf bytes.Buffer
	out.PrintTo(&buf)
	got := buf.String()

	// 3. Compare with golden file
	goldenPath := filepath.Join("testdata", "rfv1.golden")

	// If UPDATE_GOLDEN env var is set, update the file
	if os.Getenv("UPDATE_GOLDEN") == "1" {
		err := os.MkdirAll(filepath.Dir(goldenPath), 0755)
		if err != nil {
			t.Fatalf("failed to create testdata dir: %v", err)
		}
		err = os.WriteFile(goldenPath, buf.Bytes(), 0644)
		if err != nil {
			t.Fatalf("failed to update golden file: %v", err)
		}
	}

	wantBytes, err := os.ReadFile(goldenPath)
	if err != nil {
		t.Fatalf("failed to read golden file: %v. Run with UPDATE_GOLDEN=1 to create it.", err)
	}
	want := string(wantBytes)

	if got != want {
		t.Errorf("output mismatch.\nGOT:\n%s\nWANT:\n%s", got, want)
	}
}
