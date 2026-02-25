package main

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strings"
	"testing"
	"time"
)

func TestVerify_FailsOnKnownBadBundle(t *testing.T) {
	tests := []struct {
		name   string
		mutate func(hdr *tar.Header)
		expect ErrorCode
	}{
		{
			"Forbidden PAX key",
			func(hdr *tar.Header) {
				if strings.HasSuffix(hdr.Name, "INDEX.md") {
					hdr.PAXRecords = map[string]string{"foo": "bar"}
				}
			},
			E_PAX,
		},
		{
			"Provenance leak",
			func(hdr *tar.Header) {
				if hdr.Name == "review/INDEX.md" {
					hdr.PAXRecords = map[string]string{"LIBARCHIVE.xattr.com.apple.provenance": "leak"}
				}
			},
			E_XATTR,
		},
		{
			"Non-zero UID",
			func(hdr *tar.Header) {
				if hdr.Name == "review/INDEX.md" {
					hdr.Uid = 1000
				}
			},
			E_IDENTITY,
		},
		{
			"Non-empty Gname",
			func(hdr *tar.Header) {
				if hdr.Name == "review/INDEX.md" {
					hdr.Gname = "staff"
				}
			},
			E_IDENTITY,
		},
		{
			"Non-zero nanoseconds",
			func(hdr *tar.Header) {
				if hdr.Name == "review/INDEX.md" {
					hdr.ModTime = hdr.ModTime.Add(time.Nanosecond)
				}
			},
			E_TIME,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			bundleBytes, err := ForgeBundle(tt.mutate)
			if err != nil {
				t.Fatal(err)
			}

			_, err = VerifyBundle(bytes.NewReader(bundleBytes), DefaultVerifyOptions)
			if err == nil {
				t.Fatalf("Expected error %v, got success", tt.expect)
			}

			verr, ok := err.(*VError)
			if !ok {
				t.Fatalf("Expected VError, got %T: %v", err, err)
			}

			if verr.Code != tt.expect {
				t.Errorf("Expected code %v, got %v (detail: %s)", tt.expect, verr.Code, verr.Detail)
			}
		})
	}
}

func TestVerify_PassesOnMinimalValidBundle(t *testing.T) {
	bundleBytes, err := ForgeBundle(nil)
	if err != nil {
		t.Fatal(err)
	}

	rep, err := VerifyBundle(bytes.NewReader(bundleBytes), DefaultVerifyOptions)
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

func TestVerify_FileMissing_Stopless(t *testing.T) {
	bundle, _ := ForgeBundleEx(nil, "review/dummy.txt", "", nil, nil)
	_, err := VerifyBundle(bytes.NewReader(bundle), DefaultVerifyOptions)
	if verr, ok := err.(*VError); !ok || verr.Reason != "file_missing" {
		t.Fatalf("want file_missing, got %v", err)
	}
}

func TestVerify_FileExtra_Stopless(t *testing.T) {
	bundle, _ := ForgeBundleEx(nil, "", "review/z_extra.txt", nil, nil)
	_, err := VerifyBundle(bytes.NewReader(bundle), DefaultVerifyOptions)
	if verr, ok := err.(*VError); !ok || verr.Reason != "file_extra" {
		t.Fatalf("want file_extra, got %v", err)
	}
}

func TestVerify_ContractEvidenceMismatch_Stopless(t *testing.T) {
	bundle, _ := ForgeBundleEx(nil, "review/evidence/prverify/ev", "", nil, func(c *Contract) {
		c.Evidence.Required = true
		c.Evidence.Present = true
	})
	// evidence directory is missing -> rep.EvidencePresent = false
	_, err := VerifyBundle(bytes.NewReader(bundle), DefaultVerifyOptions)
	if verr, ok := err.(*VError); !ok || verr.Reason != "contract_evidence_mismatch" {
		t.Fatalf("want contract_evidence_mismatch, got %v", err)
	}
}

func TestVerify_SHA256SUMSListsSeal_IsRejected(t *testing.T) {
	bundle, _ := ForgeBundleEx(nil, "", "", func(sums []byte) []byte {
		// append a bogus seal entry to sums
		return append(sums, []byte("cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe  review/meta/SHA256SUMS.sha256\n")...)
	}, nil)
	_, err := VerifyBundle(bytes.NewReader(bundle), DefaultVerifyOptions)
	if verr, ok := err.(*VError); !ok || verr.Reason != "manifest_invalid" {
		t.Fatalf("want manifest_invalid, got %v", err)
	}
}

func ForgeBundleEx(mutate func(*tar.Header), skipFile string, extraFile string, mutateSums func([]byte) []byte, mutateContract func(*Contract)) ([]byte, error) {
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
			Required:    true,
			Present:     true,
			BoundToHead: true,
			PathPrefix:  DirEvidence,
		},
		Tool: Tool{Name: "reviewbundle", Version: "0.0.0"},
	}
	if mutateContract != nil {
		mutateContract(&contract)
	}
	contractBytes, _ := json.Marshal(contract)

	// Files to add
	files := map[string][]byte{
		"review/INDEX.md":             []byte("# Index\n"),
		"review/meta/contract.json":   contractBytes,
		"review/patch/series.patch":   []byte("diff --git a/foo b/foo\n"),
		"review/evidence/prverify/ev": []byte("evidence containing head: cafebabe00112233445566778899aabbccddeeff\n"),
		"review/dummy.txt":            []byte("dummy\n"),
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
	if mutateSums != nil {
		sha256sumsBytes = mutateSums(sha256sumsBytes)
	}
	files["review/meta/SHA256SUMS"] = sha256sumsBytes
	filenames = append(filenames, "review/meta/SHA256SUMS")
	sort.Strings(filenames) // Re-sort to include SHA256SUMS

	// Calculate SHA256SUMS.sha256
	sumsHash := sha256.Sum256(sha256sumsBytes)
	sealBytes := []byte(fmt.Sprintf("%x  review/meta/SHA256SUMS\n", sumsHash))
	files["review/meta/SHA256SUMS.sha256"] = sealBytes
	filenames = append(filenames, "review/meta/SHA256SUMS.sha256")
	sort.Strings(filenames) // Re-sort to include Seal

	writeEntry := func(name string, content []byte) error {
		mode := os.FileMode(0644)
		if name == "review/evidence/prverify/ev" {
			mode = 0755
		}

		hdr := &tar.Header{
			Name:     name,
			Size:     int64(len(content)),
			Mode:     int64(mode),
			ModTime:  time.Unix(epoch, 0),
			Typeflag: tar.TypeReg,
			Uid:      0,
			Gid:      0,
			Uname:    "",
			Gname:    "",
			Format:   tar.FormatPAX,
		}
		if mutate != nil {
			mutate(hdr)
		}

		if err := tw.WriteHeader(hdr); err != nil {
			return err
		}
		if _, err := tw.Write(content); err != nil {
			return err
		}
		return nil
	}

	// Write to tar
	for _, name := range filenames {
		if name == skipFile {
			continue
		}
		if err := writeEntry(name, files[name]); err != nil {
			return nil, err
		}
	}
	if extraFile != "" {
		if err := writeEntry(extraFile, []byte("extra")); err != nil {
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

func ForgeBundle(mutate func(*tar.Header)) ([]byte, error) {
	return ForgeBundleEx(mutate, "", "", nil, nil)
}
