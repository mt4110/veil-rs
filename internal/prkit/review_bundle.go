package prkit

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
)

func generateReviewBundle() (string, error) {
	// 1. Find the script
	scriptPath, err := findReviewBundleScript()
	if err != nil {
		return "", err
	}

	// 2. Run the script
	// MODE=wip bash <script>
	// Run from repo root to ensure consistent behavior
	// We use ExecRunner which handles CwdRel relative to RepoRoot.
	// We set CwdRel="." to mean "RepoRoot".

	spec := ExecSpec{
		Argv:   []string{"bash", scriptPath},
		CwdRel: ".",
		EnvKV:  []EnvKV{{Key: "MODE", Value: "wip"}},
	}
	res := Runner.Run(context.Background(), spec)

	// output is stdout + stderr?
	// The original code used CombinedOutput.
	// ExecResult separates them.
	// We want to parse stdout for "OK: <path>".
	// But duplicate stderr for debugging?
	// For now, let's look at stdout.
	output := res.Stdout
	if res.Stderr != "" {
		output += "\n-- stderr --\n" + res.Stderr
	}

	if res.ExitCode != 0 {
		return "", fmt.Errorf("review bundle script failed: %s\nOutput:\n%s", res.ErrorKind, output)
	}

	// 3. Parse output
	// Expecting *last line* to be "OK: <path>"
	lines := strings.Split(strings.TrimSpace(output), "\n")
	if len(lines) == 0 {
		return "", fmt.Errorf("empty output from review bundle script")
	}

	lastLine := strings.TrimSpace(lines[len(lines)-1])
	const okPrefix = "OK: "
	if !strings.HasPrefix(lastLine, okPrefix) {
		return "", fmt.Errorf("could not find '%s<path>' in last line of review bundle output:\n%s", okPrefix, output)
	}

	bundlePath := strings.TrimPrefix(lastLine, okPrefix)
	if bundlePath == "" {
		return "", fmt.Errorf("empty bundle path in review bundle output:\n%s", output)
	}

	// Adjust bundlePath to be absolute if it's relative
	// If it's relative, it's relative to CWD? Or RepoRoot?
	// The script is run from RepoRoot (CwdRel=".").
	// So output is likely relative to RepoRoot.
	// We should resolve it against RepoRoot.
	// But we removed `findRepoRoot` call here.
	// Use Runner.RepoRoot? But `Runner` is global interface.
	// We can call FindRepoRoot again or rely on main.go having initialized it?
	// Since we are inside `prkit`, we can assume `FindRepoRoot` works.
	repoRoot, err := FindRepoRoot()
	if err != nil {
		return "", fmt.Errorf("failed to resolve repo root for bundle path: %w", err)
	}
	if !filepath.IsAbs(bundlePath) {
		bundlePath = filepath.Join(repoRoot, bundlePath)
	}

	// 4. Validate existence
	if _, err := os.Stat(bundlePath); err != nil {
		return "", fmt.Errorf("bundle path reported by script does not exist: %s: %w", bundlePath, err)
	}

	// 5. Compute SHA256
	sha, err := computeSHA256(bundlePath)
	if err != nil {
		return "", fmt.Errorf("failed to compute sha256 of bundle %s: %w", bundlePath, err)
	}

	// 6. Return formatted string
	// Format: "review_bundle:<filename>:<sha256>"
	filename := filepath.Base(bundlePath)
	return fmt.Sprintf("review_bundle:%s:%s", filename, sha), nil
}

func findReviewBundleScript() (string, error) {
	candidates := []string{
		"ops/ci/review_bundle.sh",
		"ops/review_bundle.sh",
	}

	startDir, err := os.Getwd()
	if err != nil {
		return "", fmt.Errorf("failed to get current working directory: %w", err)
	}

	dir := startDir
	for {
		for _, c := range candidates {
			fullPath := filepath.Join(dir, c)
			if _, err := os.Stat(fullPath); err == nil {
				return fullPath, nil
			}
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			break
		}
		dir = parent
	}

	return "", fmt.Errorf("review_bundle script not found in %v (starting from %s)", candidates, startDir)
}

func computeSHA256(path string) (string, error) {
	f, err := os.Open(path)
	if err != nil {
		return "", err
	}
	defer f.Close()

	h := sha256.New()
	if _, err := io.Copy(h, f); err != nil {
		return "", err
	}

	return hex.EncodeToString(h.Sum(nil)), nil
}
