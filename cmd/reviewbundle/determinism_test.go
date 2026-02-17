package main

import (
	"bytes"
	"os"
	"strconv"
	"testing"
	"time"
)

func TestVerify_DeterministicResultForSameInput(t *testing.T) {
	// TODO: implement in C3/C5
}

func TestCreate_Determinism(t *testing.T) {
	// C2: Hermetic environment
	repoDir, baseSHA := forgeHermeticRepo(t)

	// Fixed epoch
	epoch := int64(1700000000)
	epochStr := strconv.FormatInt(epoch, 10)
	os.Setenv("SOURCE_DATE_EPOCH", epochStr)
	defer os.Unsetenv("SOURCE_DATE_EPOCH")

	// Resolve HeadSHA from the hermetic repo
	headSHA, err := getGitHeadSHA(repoDir)
	if err != nil {
		t.Fatalf("Failed to get head SHA: %v", err)
	}

	// Create contract template
	c := &Contract{
		ContractVersion: "1.1",
		Mode:            "wip",
		Repo:            "veil-rs",
		EpochSec:        epoch,
		BaseRef:         baseSHA,
		HeadSHA:         headSHA,
		Tool:            Tool{Name: "test", Version: "0.0.0"},
	}

	outDir := t.TempDir()

	// Run 1
	path1, err := CreateBundle(c, outDir, repoDir)
	if err != nil {
		t.Fatalf("First create failed: %v", err)
	}
	b1, err := os.ReadFile(path1)
	if err != nil {
		t.Fatal(err)
	}

	// Sleep to ensure real time doesn't leak if we had a bug
	time.Sleep(10 * time.Millisecond)

	// Run 2
	path2, err := CreateBundle(c, outDir, repoDir)
	if err != nil {
		t.Fatalf("Second create failed: %v", err)
	}
	b2, err := os.ReadFile(path2)
	if err != nil {
		t.Fatal(err)
	}

	// Compare
	if !bytes.Equal(b1, b2) {
		t.Error("Bundles are not byte-identical")
	}

	// Self-audit: Verify the generated bundle
	report, err := VerifyBundlePath(path1)
	if err != nil {
		t.Fatalf("VerifyBundlePath failed on hermetic bundle: %v", err)
	}
	if report.Contract.EpochSec != epoch {
		t.Errorf("Epoch mismatch: got %d, want %d", report.Contract.EpochSec, epoch)
	}
	// Verify manifest content
	// We expect INDEX.md, meta/contract.json, patch/series.patch, meta/SHA256SUMS*
}
