package main

import (
	"bufio"
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
)

type ChecksumLine struct {
	Path string
	Sum  [32]byte
}

func ParseSHA256SUMS(b []byte) ([]ChecksumLine, error) {
	var lines []ChecksumLine
	scanner := bufio.NewScanner(bytes.NewReader(b))
	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}
		// Format: "hash  path" (two spaces)
		parts := strings.SplitN(line, "  ", 2)
		if len(parts) != 2 {
			return nil, NewVError(E_SHA256, "SHA256SUMS", "invalid format")
		}
		sumHex := parts[0]
		path := parts[1]

		// 2.3 path 安全性
		if strings.HasPrefix(path, "/") || strings.HasPrefix(path, "\\") {
			return nil, NewVError(E_PATH, path, "absolute path forbidden in SHA256SUMS").WithReason("manifest_path_invalid")
		}
		if strings.Contains(path, "../") || strings.HasSuffix(path, "/..") || path == ".." {
			return nil, NewVError(E_PATH, path, "parent traversal forbidden in SHA256SUMS").WithReason("manifest_path_invalid")
		}
		if len(path) >= 2 && path[1] == ':' && ((path[0] >= 'A' && path[0] <= 'Z') || (path[0] >= 'a' && path[0] <= 'z')) {
			return nil, NewVError(E_PATH, path, "OS drive expression forbidden in SHA256SUMS").WithReason("manifest_path_invalid")
		}
		if path != "SHA256SUMS" && !strings.HasPrefix(path, "review/") {
			return nil, NewVError(E_PATH, path, "path must be within review/ directory").WithReason("manifest_path_invalid")
		}

		sum, err := ParseSHA256HexLine([]byte(sumHex))
		if err != nil {
			return nil, NewVError(E_SHA256, "SHA256SUMS", "invalid hex: "+sumHex)
		}
		lines = append(lines, ChecksumLine{Path: path, Sum: sum})
	}
	if err := scanner.Err(); err != nil {
		return nil, WrapVError(E_SHA256, "SHA256SUMS", err)
	}
	return lines, nil
}

func ParseSHA256HexLine(b []byte) ([32]byte, error) {
	var sum [32]byte
	if len(b) != 64 {
		return sum, fmt.Errorf("invalid length: %d", len(b))
	}
	_, err := hex.Decode(sum[:], b)
	return sum, err
}

func VerifySHA256SUMSSeal(sums []byte, seal []byte) error {
	// Seal is just the hex string of sha256(sums)
	// Expected seal content: "hexhash  review/meta/SHA256SUMS\n" ??
	// Or just the hash?
	// Contract says: "SHA256SUMS.sha256 contains the signed sha256 of SHA256SUMS"
	// S11-03 PLAN says: "SHA256SUMS.sha256 を生成（封印）"
	// Typically this is `sha256sum SHA256SUMS > SHA256SUMS.sha256`
	// So it matches standard format: "hash  filename\n"

	actualHash := sha256.Sum256(sums)

	// Parse the seal file
	lines, err := ParseSHA256SUMS(seal)
	if err != nil {
		return err
	}
	if len(lines) != 1 {
		return NewVError(E_SHA256, "SHA256SUMS.sha256", "must contain exactly one line")
	}

	// C1: Strict seal path validation
	if lines[0].Path != "review/meta/SHA256SUMS" && lines[0].Path != "SHA256SUMS" {
		return NewVError(E_SHA256, "SHA256SUMS.sha256", "unexpected filename in seal: "+lines[0].Path)
	}

	if lines[0].Sum != actualHash {
		return NewVError(E_SEAL, "SHA256SUMS.sha256", "seal mismatch (integrity violation)").WithReason("seal_broken")
	}

	return nil
}

func VerifyChecksumCompleteness(expected []ChecksumLine, computed map[string][32]byte) error {
	// 1. Check all expected are computed and match
	expectedMap := make(map[string]bool)

	for _, exp := range expected {
		expectedMap[exp.Path] = true
		comp, ok := computed[exp.Path]
		if !ok {
			return NewVError(E_MISSING, exp.Path, "missing in bundle but present in manifest").WithReason("missing_file")
		}
		if comp != exp.Sum {
			return NewVError(E_SHA256, exp.Path, fmt.Sprintf("checksum mismatch\nwant: %x\ngot:  %x", exp.Sum, comp)).WithReason("sha_mismatch")
		}
	}

	// 2. Check all computed checks are in expected (except self and seal and warnings?)
	// Contract: "SHA256SUMS MUST list all files except itself and *.sha256"
	// Actually "warnings.txt" IS included.
	for path := range computed {
		if path == "review/meta/SHA256SUMS" || path == "review/meta/SHA256SUMS.sha256" {
			continue
		}
		if !expectedMap[path] {
			return NewVError(E_EXTRA, path, "present in bundle but missing in manifest").WithReason("extra_file")
		}
	}
	return nil
}
