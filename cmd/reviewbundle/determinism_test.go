package main

import (
	"bytes"
	"os"
	"testing"
)

func TestCreate_Determinism(t *testing.T) {
	// Hermetic test: create a synthetic repo in TempDir
	repoDir := t.TempDir()
	InitRepo(t, repoDir)

	// Create 'main' branch content (base)
	CommitFile(t, repoDir, "README.md", "init", "Initial commit")

	// Make feature branch
	MakeBranch(t, repoDir, "feature/foo")
	mustRunGit(t, repoDir, "checkout", "feature/foo")
	CommitFile(t, repoDir, "foo.txt", "foo content", "Add foo feature")

	headSHA := GetHeadSHA(t, repoDir)

	// We need 2 separate runs to verify determinism
	// Run 1
	outDir1 := t.TempDir()
	epoch := int64(1000000000) // Fixed epoch for determinism

	c1 := &Contract{
		ContractVersion: "1.1",
		Mode:            ModeWIP, // Use WIP to allow dirty state if needed, though this repo is clean
		Repo:            "veil-rs",
		EpochSec:        epoch,
		BaseRef:         "main",
		HeadSHA:         headSHA,
		// Evidence is optional in WIP
		Evidence: Evidence{
			Required:   false,
			PathPrefix: DirEvidence,
		},
		Tool: Tool{
			Name:    "reviewbundle",
			Version: "test",
		},
	}

	path1, err := CreateBundle(c1, outDir1, repoDir, "")
	if err != nil {
		t.Fatalf("CreateBundle(1) failed: %v", err)
	}

	// Run 2 (same inputs)
	outDir2 := t.TempDir()
	c2 := &Contract{
		ContractVersion: "1.1",
		Mode:            ModeWIP,
		Repo:            "veil-rs",
		EpochSec:        epoch,
		BaseRef:         "main",
		HeadSHA:         headSHA,
		Evidence: Evidence{
			Required:   false,
			PathPrefix: DirEvidence,
		},
		Tool: Tool{
			Name:    "reviewbundle",
			Version: "test",
		},
	}

	path2, err := CreateBundle(c2, outDir2, repoDir, "")
	if err != nil {
		t.Fatalf("CreateBundle(2) failed: %v", err)
	}

	// Compare binary content
	b1, err := os.ReadFile(path1)
	if err != nil {
		t.Fatal(err)
	}
	b2, err := os.ReadFile(path2)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(b1, b2) {
		t.Errorf("Bundle binary content differs between runs with identical input")
	}
}
