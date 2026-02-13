package prkit

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"os/exec"
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
	cmd := exec.Command("bash", scriptPath)
	cmd.Env = append(os.Environ(), "MODE=wip")

	// We need to capture stdout to parse "OK: <path>"
	// We also want to capture stderr to debug or pass through?
	// The plan says "parse output to get bundle path".
	outputBytes, err := cmd.CombinedOutput()
	output := string(outputBytes)

	if err != nil {
		return "", fmt.Errorf("review bundle script failed: %w\nOutput:\n%s", err, output)
	}

	// 3. Parse output
	// Expecting last line to be "OK: <path>"
	lines := strings.Split(strings.TrimSpace(output), "\n")
	var bundlePath string
	for i := len(lines) - 1; i >= 0; i-- {
		line := strings.TrimSpace(lines[i])
		if strings.HasPrefix(line, "OK: ") {
			bundlePath = strings.TrimPrefix(line, "OK: ")
			break
		}
	}

	if bundlePath == "" {
		return "", fmt.Errorf("could not find 'OK: <path>' in review bundle output:\n%s", output)
	}

	// 4. Compute SHA256
	// The path from review_bundle.sh might be relative to repo root or absolute.
	// The script is run from CWD which is repo root.
	// Output is usually relative ".local/..." or absolute.
	sha, err := computeSHA256(bundlePath)
	if err != nil {
		return "", fmt.Errorf("failed to compute sha256 of bundle %s: %w", bundlePath, err)
	}

	// 5. Return formatted string
	// Format: "review_bundle:<filename>:<sha256>"
	filename := filepath.Base(bundlePath)
	return fmt.Sprintf("review_bundle:%s:%s", filename, sha), nil
}

func findReviewBundleScript() (string, error) {
	candidates := []string{
		"ops/ci/review_bundle.sh",
		"ops/review_bundle.sh",
	}

	for _, c := range candidates {
		if _, err := os.Stat(c); err == nil {
			return c, nil
		}
	}
	return "", fmt.Errorf("review_bundle script not found in %v", candidates)
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
