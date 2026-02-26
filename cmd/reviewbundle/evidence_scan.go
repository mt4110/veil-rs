package main

import (
	"bytes"
	"strings"
)

var forbiddenAbsPrefixes = []string{
	"/Users/", "/home/", "/etc/", "/var/", "/private/", "/Volumes/", "/mnt/",
}

// scanEvidenceContent searches the byte content for forbidden patterns (stop=1).
func scanEvidenceContent(name string, content []byte) error {
	// Skip binary-looking files quickly (e.g. NUL byte present in first 512 bytes)
	checkLen := len(content)
	if checkLen > 512 {
		checkLen = 512
	}
	if bytes.IndexByte(content[:checkLen], 0) != -1 {
		return nil // skip binary
	}

	text := string(content)
	lowerText := strings.ToLower(text)

	// file scheme checks (case-insensitive), boundary-aware to reduce false positives
	for i := 0; i < len(text); i++ {
		// Require start-of-text or a boundary before a potential "file:" token
		if i > 0 && !isBoundary(text[i-1]) {
			continue
		}
		// Need at least "file:" starting at position i
		if i+5 > len(text) {
			break
		}
		lowerSlice := lowerText[i : i+5]
		if lowerSlice == "file:" {
			// Position just after "file:"
			j := i + 5
			if j >= len(text) {
				continue
			}
			// Require at least one slash or backslash to consider this a file URL/path
			if text[j] != '/' && text[j] != '\\' {
				continue
			}
			// Skip all consecutive slashes/backslashes (e.g., file:/, file://, file:///)
			k := j
			for k < len(text) && (text[k] == '/' || text[k] == '\\') {
				k++
			}
			// If there's nothing after the slashes, treat it as a forbidden file reference
			if k >= len(text) {
				return NewVError(E_EVIDENCE, name, "forbidden file scheme detected").WithReason("evidence_forbidden")
			}
			// If the next character is whitespace, this likely appears in prose
			// such as "the file:// protocol", so treat it as safe.
			if text[k] == ' ' || text[k] == '\t' || text[k] == '\n' || text[k] == '\r' {
				continue
			}
			// Otherwise, this looks like a real file URL/path and should be rejected.
			return NewVError(E_EVIDENCE, name, "forbidden file scheme detected").WithReason("evidence_forbidden")
		}
	}

	// absolute path heuristics and parent dir traversals
	// We want to detect expressions like `../` but NOT in `https://.../../`
	// And `/Users/` but at the beginning of line, or after space/quotes.

	// A simple approach is scanning line by line and word by word, or simply scanning indexes and checking preceding character.

	for i := 0; i < len(text); i++ {
		// Parent dir: ../
		if strings.HasPrefix(text[i:], "../") {
			// Ensure it's not part of a URL (very heuristic: check if preceded by something like `://`? or just check if `http` is nearby)
			// Actually safer: check prefix character: space, quote, beginning of line, or `/`
			if i == 0 || isBoundary(text[i-1]) {
				// but wait, bounded `../` could be safe? The PR spec says "基本スルーできる形で" (so maybe check if the line contains https? No, just preceded by boundary)
				// simplest boundary is space, quote, newline, or it's the start
				return NewVError(E_EVIDENCE, name, "forbidden parent traversal detected").WithReason("evidence_forbidden")
			}
		}

		// Windows drives (C:\, D:\, C:/)
		if i+2 < len(text) && ((text[i] >= 'A' && text[i] <= 'Z') || (text[i] >= 'a' && text[i] <= 'z')) {
			if text[i+1] == ':' && (text[i+2] == '\\' || text[i+2] == '/') {
				if i == 0 || isBoundary(text[i-1]) {
					return NewVError(E_EVIDENCE, name, "forbidden Windows drive path detected").WithReason("evidence_forbidden")
				}
			}
		}

		// Absolute path prefixes
		if text[i] == '/' {
			if i == 0 || isBoundary(text[i-1]) {
				for _, prefix := range forbiddenAbsPrefixes {
					if strings.HasPrefix(text[i:], prefix) {
						return NewVError(E_EVIDENCE, name, "forbidden absolute path detected: "+prefix).WithReason("evidence_forbidden")
					}
				}
			}
		}
	}

	return nil
}

func isBoundary(b byte) bool {
	return b == ' ' || b == '\t' || b == '\n' || b == '\r' || b == '"' || b == '\'' || b == '`'
}
