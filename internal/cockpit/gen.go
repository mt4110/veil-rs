package cockpit

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// GenerateDrafts generates publication drafts for the given version.
// version: e.g. "v0.14.0"
// baseRef: optional, defaults to origin/main (handled by GenerateAIPack)
// Returns the output directory path.
func GenerateDrafts(version, baseRef string) (string, error) {
	if version == "" {
		return "", fmt.Errorf("version is required (e.g. vX.Y.Z)")
	}

	// 1. Resolve Root
	gx := GitX{}
	rootStr, err := gx.Run("rev-parse", "--show-toplevel")
	if err != nil {
		return "", fmt.Errorf("failed to find repo root: %w", err)
	}
	repoRoot := strings.TrimSpace(rootStr)

	// 2. Prepare Output Directory
	outDir := filepath.Join(repoRoot, "dist", "publish", version)
	if err := os.MkdirAll(outDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create output directory: %w", err)
	}

	// 3. Process Templates
	templates := []string{
		"PUBLISH_TEMPLATE.md",
		"RELEASE_BODY_TEMPLATE.md",
		"X_TEMPLATE.md",
	}

	for _, tmpl := range templates {
		srcPath := filepath.Join(repoRoot, "docs", "ai", tmpl)
		// e.g. PUBLISH_v0.14.0.md
		baseName := strings.TrimSuffix(tmpl, "_TEMPLATE.md")
		dstName := fmt.Sprintf("%s_%s.md", baseName, version)
		dstPath := filepath.Join(outDir, dstName)

		if err := copyAndReplace(srcPath, dstPath, "vX.Y.Z", version); err != nil {
			return "", fmt.Errorf("failed to process template %s: %w", tmpl, err)
		}
	}

	// 4. Generate AI_PACK Artifact
	packPath := filepath.Join(outDir, fmt.Sprintf("AI_PACK_%s.txt", version))
	// GenerateAIPack handles default baseRef logic internally
	if _, _, err := GenerateAIPack(baseRef, packPath); err != nil {
		return "", fmt.Errorf("failed to generate AI_PACK: %w", err)
	}

	return outDir, nil
}

// copyAndReplace reads src, replaces all occurrences of old with new, and writes to dst.
func copyAndReplace(src, dst, old, replacement string) error {
	contentBytes, err := os.ReadFile(src)
	if err != nil {
		return err
	}
	content := string(contentBytes)
	replaced := strings.ReplaceAll(content, old, replacement)

	return os.WriteFile(dst, []byte(replaced), 0644)
}
