package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// ---- Dist validation ----

func requiredArtifacts(ver string) []string {
	return []string{
		fmt.Sprintf("PUBLISH_%s.md", ver),
		fmt.Sprintf("RELEASE_BODY_%s.md", ver),
		fmt.Sprintf("X_%s.md", ver),
		fmt.Sprintf("AI_PACK_%s.txt", ver),
	}
}

func distOutDir(ver string) string {
	return filepath.Join("dist", "publish", ver)
}

func distTmpDir(ver string) string {
	// tmp is under dist/publish to keep rename atomic
	return filepath.Join("dist", "publish", fmt.Sprintf(".tmp-%s-%d", ver, os.Getpid()))
}

func validateDistExactly4(dir, ver string) error {
	ents, err := os.ReadDir(dir)
	if err != nil {
		return err
	}
	var names []string
	for _, e := range ents {
		if e.IsDir() {
			continue
		}
		names = append(names, e.Name())
	}
	sort.Strings(names)

	// leak guard: AI_PACK*.md is forbidden
	for _, n := range names {
		if strings.HasPrefix(n, "AI_PACK_") && strings.HasSuffix(n, ".md") {
			return fmt.Errorf("AI_PACK must be .txt (found forbidden file: %s)", filepath.Join(dir, n))
		}
	}

	req := requiredArtifacts(ver)
	sort.Strings(req)

	if len(names) != 4 {
		return fmt.Errorf("dist must contain exactly 4 artifacts (found %d): %v", len(names), names)
	}
	for i := range req {
		if names[i] != req[i] {
			return fmt.Errorf("dist artifacts mismatch: want %v, got %v", req, names)
		}
	}
	return nil
}

func safeRemoveOutDir(out string, ver string) error {
	// Must be exactly dist/publish/<VER>
	want := distOutDir(ver)
	// clean comparisons in OS-specific form
	out = filepath.Clean(out)
	want = filepath.Clean(want)
	if out != want {
		return fmt.Errorf("refusing to remove non-contract path: %s (want %s)", out, want)
	}
	if out == "." || out == "dist" || out == filepath.Join("dist", "publish") || out == string(filepath.Separator) {
		return fmt.Errorf("refusing to remove unsafe path: %s", out)
	}
	return os.RemoveAll(out)
}
