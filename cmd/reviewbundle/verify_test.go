package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestVerify_FailsOnKnownBadBundle(t *testing.T) {
	// Locate the known-bad bundle in .local/review-bundles
	// We assume verify_test.go is in cmd/reviewbundle/
	// So repo root is ../../
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	repoRoot := filepath.Dir(filepath.Dir(wd))
	bundleDir := filepath.Join(repoRoot, ".local", "review-bundles")

	entries, err := os.ReadDir(bundleDir)
	if err != nil {
		t.Skipf("Skipping known-bad test: cannot read %s: %v", bundleDir, err)
	}

	var bundlePath string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".tar.gz") {
			bundlePath = filepath.Join(bundleDir, e.Name())
			break // Pick first one
		}
	}

	if bundlePath == "" {
		t.Skip("Skipping known-bad test: no bundle found in .local/review-bundles")
	}

	t.Logf("Testing with bundle: %s", bundlePath)

	rep, err := VerifyBundlePath(bundlePath)
	if err == nil {
		t.Fatalf("Expected verify error on known-bad bundle, got success (report: %+v)", rep)
	}

	// We expect E_XATTR or E_PAX
	verr, ok := err.(*VError)
	if !ok {
		t.Fatalf("Expected VError, got %T: %v", err, err)
	}

	t.Logf("Got expected error: %v", verr)

	if verr.Code != E_XATTR && verr.Code != E_PAX {
		t.Errorf("Expected E_XATTR or E_PAX, got %s", verr.Code)
	}
}

func TestVerify_PassesOnMinimalValidBundle(t *testing.T) {
	// TODO: implement in C5.2
}

func TestVerify_DeterministicResultForSameInput(t *testing.T) {
	// TODO: implement in determinism_test.go
}
