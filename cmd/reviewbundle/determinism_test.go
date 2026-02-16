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
	epoch := int64(1700000000)
	os.Setenv("SOURCE_DATE_EPOCH", strconv.FormatInt(epoch, 10))
	defer os.Unsetenv("SOURCE_DATE_EPOCH")

	headSHA, _ := getGitHeadSHA()
	c := &Contract{
		ContractVersion: "1.1",
		Mode:            "wip",
		Repo:            "veil-rs",
		EpochSec:        epoch,
		BaseRef:         "main",
		HeadSHA:         headSHA,
		Tool:            Tool{Name: "test", Version: "0.0.0"},
	}

	outDir := t.TempDir()
	path1, err := CreateBundle(c, outDir)
	if err != nil {
		t.Fatalf("First create failed: %v", err)
	}
	b1, _ := os.ReadFile(path1)

	// Sleep to ensure time doesn't leak if we had a bug
	time.Sleep(10 * time.Millisecond)

	path2, err := CreateBundle(c, outDir)
	if err != nil {
		t.Fatalf("Second create failed: %v", err)
	}
	b2, _ := os.ReadFile(path2)

	if !bytes.Equal(b1, b2) {
		t.Error("Bundles are not byte-identical")
	}
}
