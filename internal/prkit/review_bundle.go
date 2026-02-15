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
	// 0. Ensure we know RepoRoot
	repoRoot, err := FindRepoRoot()
	if err != nil {
		return "", fmt.Errorf("failed to find repo root: %w", err)
	}

	// 1. Find the script (Repo-relative)
	scriptPath, err := findReviewBundleScript(repoRoot)
	if err != nil {
		return "", err
	}

	// 2. Run the script
	// MODE=wip bash <script>
	// Run from repo root.
	spec := ExecSpec{
		Argv: []string{"bash", scriptPath},
		Dir:  ".",
		Env:  []string{"MODE=wip"},
	}
	res := Runner.Run(context.Background(), spec)

	// 3. Check execution (Fail early if non-zero)
	if res.ExitCode != 0 {
		return "", fmt.Errorf("review bundle script failed: %s\nStdout:\n%s\nStderr:\n%s",
			res.ErrorKind, res.Stdout, res.Stderr)
	}

	// 4. Parse stdout (Ignore stderr for parsing)
	// Expecting *last line* to be "OK: <path>"
	// S10-09: "OK: 行は stdout を後ろから走査して最後の OK: を拾う"
	lines := strings.Split(strings.TrimSpace(res.Stdout), "\n")
	var bundlePath string
	const okPrefix = "OK: "

	for i := len(lines) - 1; i >= 0; i-- {
		line := strings.TrimSpace(lines[i])
		if strings.HasPrefix(line, okPrefix) {
			bundlePath = strings.TrimPrefix(line, okPrefix)
			break
		}
	}

	if bundlePath == "" {
		return "", fmt.Errorf("could not find '%s<path>' in output:\n%s", okPrefix, res.Stdout)
	}

	// 5. Normalization (Repo-relative contract)
	// If absolute, make it relative to repoRoot
	if filepath.IsAbs(bundlePath) {
		rel, err := filepath.Rel(repoRoot, bundlePath)
		if err == nil {
			bundlePath = rel
		}
		// If fails to relativize, checking existence will handle it?
		// Plan says: "bundlePath は repo相対で返す（絶対なら Rel(repoRoot, abs)）"
	}

	// Ensure it is not outside repo (just in case)
	cleanPath := filepath.Clean(bundlePath)
	if strings.HasPrefix(cleanPath, "..") {
		return "", fmt.Errorf("bundle path escapes repo root: %s", bundlePath)
	}
	bundlePath = cleanPath

	// 6. Validate existence
	fullPath := filepath.Join(repoRoot, bundlePath)
	if _, err := os.Stat(fullPath); err != nil {
		return "", fmt.Errorf("bundle path reported by script does not exist: %s (abs: %s): %w", bundlePath, fullPath, err)
	}

	// 7. Compute SHA256
	sha, err := computeSHA256(fullPath)
	if err != nil {
		return "", fmt.Errorf("failed to compute sha256 of bundle %s: %w", fullPath, err)
	}

	// 8. Return formatted string
	// Format: "review_bundle:<filename>:<sha256>"
	// Filename should be base name? The spec usually implies filename.
	filename := filepath.Base(bundlePath)
	return fmt.Sprintf("review_bundle:%s:%s", filename, sha), nil
}

func findReviewBundleScript(repoRoot string) (string, error) {
	candidates := []string{
		"ops/ci/review_bundle.sh",
		"ops/review_bundle.sh",
	}

	// Search relative to repoRoot
	for _, c := range candidates {
		fullPath := filepath.Join(repoRoot, c)
		if _, err := os.Stat(fullPath); err == nil {
			return c, nil
		}
	}

	return "", fmt.Errorf("review_bundle script not found in %v (checked in %s)", candidates, repoRoot)
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
