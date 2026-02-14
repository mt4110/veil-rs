package prkit

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

func ScaffoldSOT(epic, slug, release string, apply bool) error {
	// 1. Validate inputs
	if epic == "" {
		return fmt.Errorf("--epic is required")
	}
	if slug == "" {
		return fmt.Errorf("--slug is required")
	}

	// 2. Detect release if missing
	if release == "" {
		detected, err := detectRelease()
		if err != nil {
			return fmt.Errorf("release autodetect failed (provide --release): %w", err)
		}
		release = detected
	}

	// 3. Determine SOT path
	// docs/pr/PR-TBD-<release>-epic-<epic>-<slug>.md
	filename := fmt.Sprintf("PR-TBD-%s-epic-%s-%s.md", release, epic, slug)
	repoRoot, err := FindRepoRoot()
	if err != nil {
		return err
	}
	path := filepath.Join(repoRoot, "docs", "pr", filename)

	// 4. Generate Content
	content := generateSOTContent(epic, slug, release)

	// 5. Output
	relPath, relErr := filepath.Rel(repoRoot, path)
	displayPath := path
	if relErr == nil {
		displayPath = relPath
	}

	if apply {
		if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
			return err
		}
		// Check if exists
		if _, err := os.Stat(path); err == nil {
			return fmt.Errorf("file already exists: %s", displayPath)
		}
		if err := os.WriteFile(path, []byte(content), 0644); err != nil {
			return err
		}
		fmt.Printf("Created SOT: %s\n", displayPath)
	} else {
		fmt.Printf("Preview SOT: %s\n", displayPath)
		fmt.Println("---------------------------------------------------")
		fmt.Println(content)
		fmt.Println("---------------------------------------------------")
		fmt.Println("Run with --apply to write file.")
	}

	return nil
}

func detectRelease() (string, error) {
	// git describe --tags --match "v*" --abbrev=0
	spec := ExecSpec{
		Argv: []string{"git", "describe", "--tags", "--match", "v*", "--abbrev=0"},
	}
	res := Runner.Run(context.Background(), spec)
	if res.ExitCode != 0 {
		return "", fmt.Errorf("git describe failed: %s", res.Stderr)
	}
	return strings.TrimSpace(res.Stdout), nil
}

func generateSOTContent(epic, slug, release string) string {
	ts := time.Now().UTC().Format("2006-01-02")
	user := os.Getenv("USER")
	if user == "" {
		user = "unknown"
	}
	return fmt.Sprintf(`# [PR-TBD] %s

## Meta
- Epic: %s
- Release: %s
- Date: %s
- Author: @%s
- Status: Draft

## Goal
TODO: Describe the goal of this PR.

## Plan
- [ ] TODO

## Verification
- [ ] TODO
`, slug, epic, release, ts, user)
}
