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

	if lines[0].Path != "review/meta/SHA256SUMS" {
		// Tolerable? Standard sha256sum output includes filename.
		// Contract says "review/meta/SHA256SUMS" is the name.
		// If generated from root, it might be that.
		// Let's enforce it matches.
		if !strings.HasSuffix(lines[0].Path, "SHA256SUMS") {
			return NewVError(E_SHA256, "SHA256SUMS.sha256", "unexpected filename in seal: "+lines[0].Path)
		}
	}

	if lines[0].Sum != actualHash {
		return NewVError(E_SHA256, "SHA256SUMS.sha256", "seal mismatch (integrity violation)")
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
			return NewVError(E_SHA256, exp.Path, "missing in bundle but present in manifest")
		}
		if comp != exp.Sum {
			return NewVError(E_SHA256, exp.Path, fmt.Sprintf("checksum mismatch\nwant: %x\ngot:  %x", exp.Sum, comp))
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
			return NewVError(E_SHA256, path, "present in bundle but missing in manifest")
		}
	}
	return nil
}
