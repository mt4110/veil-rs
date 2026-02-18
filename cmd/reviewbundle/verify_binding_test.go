package main

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
	"testing"
	"time"
)

func TestVerify_EvidenceBinding(t *testing.T) {
	// Constants for testing
	const (
		mockSHA   = "1111111111111111111111111111111111111111" // 40 chars
		mockEpoch = 1600000000
	)

	tests := []struct {
		name      string
		mode      string
		evidence  map[string]string // filename -> content
		wantError string            // empty if success expected
	}{
		{
			name: "Strict_Pass",
			mode: "strict",
			evidence: map[string]string{
				"review/evidence/prverify.md": "Report\n- head_sha: " + mockSHA + "\n",
			},
			wantError: "",
		},
		{
			name:      "Strict_Fail_NoEvidence",
			mode:      "strict",
			evidence:  nil,
			wantError: "required but missing",
		},
		{
			name: "Strict_Fail_UnboundEvidence",
			mode: "strict",
			evidence: map[string]string{
				"review/evidence/prverify.md": "Report for some other SHA",
			},
			wantError: "no evidence file contains HEAD SHA",
		},
		{
			name:      "WIP_Pass_NoEvidence",
			mode:      "wip",
			evidence:  nil,
			wantError: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Prepare file contents
			files := make(map[string][]byte)

			// 1. review/INDEX.md
			files["review/INDEX.md"] = []byte("index\n")

			// 2. review/evidence/...
			for k, v := range tt.evidence {
				files[k] = []byte(v)
			}

			// 3. review/meta/contract.json
			contract := &Contract{
				ContractVersion: "1.1",
				Mode:            tt.mode,
				Repo:            "github.com/example/repo",
				EpochSec:        mockEpoch,
				HeadSHA:         mockSHA,
			}
			if tt.mode == "strict" {
				contract.Evidence.Required = true
			}
			contractBytes, _ := json.Marshal(contract)
			files["review/meta/contract.json"] = contractBytes

			// 4. review/patch/series.patch
			files["review/patch/series.patch"] = []byte("patch")

			// Calculate SHA256SUMS
			var sumLines []string
			var paths []string
			for k := range files {
				paths = append(paths, k)
			}
			sort.Strings(paths)

			for _, p := range paths {
				sum := sha256.Sum256(files[p])
				// Format: "hash  path"
				sumLines = append(sumLines, fmt.Sprintf("%x  %s", sum, p))
			}
			sha256SumsContent := []byte(strings.Join(sumLines, "\n") + "\n")

			// Calculate SHA256SUMS.seal
			shaSum := sha256.Sum256(sha256SumsContent)
			sealContent := []byte(fmt.Sprintf("%x  review/meta/SHA256SUMS\n", shaSum))

			// Output to tar
			var buf bytes.Buffer
			gw := gzip.NewWriter(&buf)
			gw.Header.ModTime = time.Unix(mockEpoch, 0)
			gw.Header.OS = 255
			tw := tar.NewWriter(gw)

			add := func(name string, content []byte) {
				addFile(t, tw, name, content)
			}

			// Add files in strict alphabetical order
			// 1. review/INDEX.md
			add("review/INDEX.md", files["review/INDEX.md"])

			// 2. review/evidence/...
			var evKeys []string
			for k := range tt.evidence {
				evKeys = append(evKeys, k)
			}
			sort.Strings(evKeys)
			for _, k := range evKeys {
				add(k, files[k])
			}

			// 3. review/meta/SHA256SUMS
			add("review/meta/SHA256SUMS", sha256SumsContent)

			// 4. review/meta/SHA256SUMS.sha256
			add("review/meta/SHA256SUMS.sha256", sealContent)

			// 5. review/meta/contract.json
			add("review/meta/contract.json", files["review/meta/contract.json"])

			// 6. review/patch/series.patch
			add("review/patch/series.patch", files["review/patch/series.patch"])

			tw.Close()
			gw.Close()

			// Verify
			_, err := VerifyBundle(&buf)
			if tt.wantError == "" {
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
			} else {
				if err == nil {
					t.Fatalf("expected error containing %q, got nil", tt.wantError)
				}
				if !strings.Contains(err.Error(), tt.wantError) {
					t.Errorf("expected error containing %q, got %v", tt.wantError, err)
				}
			}
		})
	}
}

func addFile(t *testing.T, tw *tar.Writer, name string, body []byte) {
	h := &tar.Header{
		Name:     name,
		Mode:     0644,
		Size:     int64(len(body)),
		ModTime:  time.Unix(1600000000, 0),
		Typeflag: tar.TypeReg,
	}
	if err := tw.WriteHeader(h); err != nil {
		t.Fatal(err)
	}
	if _, err := tw.Write(body); err != nil {
		t.Fatal(err)
	}
}
