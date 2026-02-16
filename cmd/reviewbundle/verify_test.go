package main

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
	"time"
)

func TestVerify_FailsOnKnownBadBundle(t *testing.T) {
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
			break
		}
	}

	if bundlePath == "" {
		t.Skip("Skipping known-bad test: no bundle found in .local/review-bundles")
	}

	t.Logf("Testing with bundle: %s", bundlePath)

	_, err = VerifyBundlePath(bundlePath)
	if err == nil {
		t.Fatalf("Expected verify error on known-bad bundle, got success")
	}

	verr, ok := err.(*VError)
	if !ok {
		t.Fatalf("Expected VError, got %T: %v", err, err)
	}

	// E_IDENTITY can also occur (e.g. if uid/gid/uname/gname are set)
	// Accept E_IDENTITY, E_XATTR, or E_PAX as valid failure modes
	if verr.Code != E_XATTR && verr.Code != E_PAX && verr.Code != E_IDENTITY {
		t.Errorf("Expected E_XATTR/E_PAX/E_IDENTITY, got %s", verr.Code)
	}
}

func TestVerify_PassesOnMinimalValidBundle(t *testing.T) {
	bundleBytes, err := ForgeBundle()
	if err != nil {
		t.Fatal(err)
	}

	rep, err := VerifyBundle(bytes.NewReader(bundleBytes))
	if err != nil {
		t.Fatalf("Verify failed on valid bundle: %v", err)
	}

	if !rep.HasContractJSON {
		t.Error("Contract JSON missing in report")
	}
	if !rep.HasSHA256SUMS {
		t.Error("SHA256SUMS missing in report")
	}
}

func ForgeBundle() ([]byte, error) {
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)

	// Set Gzip header mtime to EpochSec
	epoch := int64(1700000000)
	gw.Header.ModTime = time.Unix(epoch, 0)
	gw.Header.OS = 255 // unknown/fixed

	tw := tar.NewWriter(gw)

	// Contract
	contract := Contract{
		ContractVersion: "1.1",
		Mode:            "strict",
		Repo:            "github.com/example/repo",
		EpochSec:        epoch,
		BaseRef:         "main",
		HeadSHA:         "cafebabe00112233445566778899aabbccddeeff",
		Evidence: Evidence{
			Required: true,
		},
		Tool: Tool{Name: "reviewbundle", Version: "0.0.0"},
	}
	contractBytes, _ := json.Marshal(contract)

	// Files to add
	files := map[string][]byte{
		"review/INDEX.md":             []byte("# Index\n"),
		"review/meta/contract.json":   contractBytes,
		"review/patch/series.patch":   []byte("diff --git a/foo b/foo\n"),
		"review/evidence/prverify/ev": []byte("evidence containing head: cafebabe00112233445566778899aabbccddeeff\n"),
	}

	// Calculate SHA256SUMS content
	// Map to list for sorting
	var filenames []string
	for k := range files {
		filenames = append(filenames, k)
	}
	sort.Strings(filenames)

	var sumsBuilder strings.Builder
	for _, name := range filenames {
		hash := sha256.Sum256(files[name])
		sumsBuilder.WriteString(fmt.Sprintf("%x  %s\n", hash, name))
	}
	sha256sumsBytes := []byte(sumsBuilder.String())
	files["review/meta/SHA256SUMS"] = sha256sumsBytes
	filenames = append(filenames, "review/meta/SHA256SUMS")
	sort.Strings(filenames) // Re-sort to include SHA256SUMS

	// Calculate SHA256SUMS.sha256
	sumsHash := sha256.Sum256(sha256sumsBytes)
	sealBytes := []byte(fmt.Sprintf("%x  review/meta/SHA256SUMS\n", sumsHash))
	files["review/meta/SHA256SUMS.sha256"] = sealBytes
	filenames = append(filenames, "review/meta/SHA256SUMS.sha256")
	sort.Strings(filenames) // Re-sort to include Seal

	// Write to tar
	for _, name := range filenames {
		content := files[name]
		hdr := &tar.Header{
			Name:     name,
			Size:     int64(len(content)),
			Mode:     0644,
			ModTime:  time.Unix(epoch, 0),
			Typeflag: tar.TypeReg,
			Uid:      0,
			Gid:      0,
			Uname:    "",
			Gname:    "",
			Format:   tar.FormatPAX,
		}
		if err := tw.WriteHeader(hdr); err != nil {
			return nil, err
		}
		if _, err := tw.Write(content); err != nil {
			return nil, err
		}
	}

	if err := tw.Close(); err != nil {
		return nil, err
	}
	if err := gw.Close(); err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}
