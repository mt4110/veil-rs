package main

import (
	"bytes"
	"fmt"
	"path/filepath"
	"strings"
)

// ---- Templates ----

type tplKind int

const (
	tplPublish tplKind = iota
	tplReleaseBody
	tplX
)

func templateCandidates(kind tplKind) []string {
	// Adjust here if your repo layout differs.
	switch kind {
	case tplPublish:
		return []string{
			filepath.Join("docs", "ai", "PUBLISH_TEMPLATE.md"),
		}
	case tplReleaseBody:
		return []string{
			filepath.Join("docs", "ai", "RELEASE_BODY_TEMPLATE.md"),
		}
	case tplX:
		return []string{
			filepath.Join("docs", "ai", "X_TEMPLATE.md"),
		}
	default:
		return nil
	}
}

func resolveTemplate(kind tplKind) (string, error) {
	paths := templateCandidates(kind)
	for _, p := range paths {
		if fileExists(p) {
			return p, nil
		}
	}
	return "", fmt.Errorf("template not found (searched: %s)", strings.Join(paths, ", "))
}

func applyTemplate(b []byte, ver string) []byte {
	s := string(b)
	// Conservative replacements (no magic)
	s = strings.ReplaceAll(s, "{{VERSION}}", ver)
	s = strings.ReplaceAll(s, "{{VER}}", ver)
	s = strings.ReplaceAll(s, "${VERSION}", ver)
	s = strings.ReplaceAll(s, "<VER>", ver)
	return []byte(s)
}

func checkMarkdownTemplate(path string, content []byte) error {
	if len(bytes.TrimSpace(content)) == 0 {
		return fmt.Errorf("template is empty: %s", path)
	}
	// H1 count (lines starting with "# ")
	lines := strings.Split(string(content), "\n")
	h1 := 0
	for _, ln := range lines {
		if strings.HasPrefix(ln, "# ") {
			h1++
		}
	}
	if h1 != 1 {
		return fmt.Errorf("template must contain exactly 1 H1 (# ): %s (found %d)", path, h1)
	}
	// code fence count must be even
	fences := strings.Count(string(content), "```")
	if fences%2 != 0 {
		return fmt.Errorf("code fence count must be even: %s (found %d)", path, fences)
	}
	return nil
}
