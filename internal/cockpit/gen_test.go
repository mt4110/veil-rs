package cockpit_test

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"veil-rs/internal/cockpit"
)

func TestGenerateDraftsIntegration(t *testing.T) {
	// 1. Setup minimal repo
	tmpDir := setupGitRepo(t)
	defer os.RemoveAll(tmpDir)

	// 2. Setup templates
	setupTemplates(t, tmpDir)

	// 3. Make some changes
	setupChanges(t, tmpDir)

	// 4. Run GenerateDrafts
	version := "v1.2.3"
	outDir, err := cockpit.GenerateDrafts(version, "HEAD~1")
	if err != nil {
		t.Fatalf("GenerateDrafts failed: %v", err)
	}

	// 5. Verify
	verifyDraftsOutput(t, tmpDir, outDir, version)
}

func setupGitRepo(t *testing.T) string {
	tmpDir, err := os.MkdirTemp("", "cockpit-gen-test-*")
	if err != nil {
		t.Fatal(err)
	}

	git := func(args ...string) {
		cmd := exec.Command("git", args...)
		cmd.Dir = tmpDir
		cmd.Env = append(os.Environ(),
			"GIT_AUTHOR_NAME=Test", "GIT_AUTHOR_EMAIL=test@example.com",
			"GIT_COMMITTER_NAME=Test", "GIT_COMMITTER_EMAIL=test@example.com",
		)
		if err := cmd.Run(); err != nil {
			t.Fatalf("git %v failed: %v", args, err)
		}
	}

	git("init", "-b", "main")
	git("config", "user.name", "Test")
	git("config", "user.email", "test@example.com")

	// Switch CWD to temp repo (GenerateDrafts relies on rev-parse --show-toplevel)
	// Note: changing CWD globally in tests is risky if parallel, but this test is sequential logic
	// Ideally GenerateDrafts should accept a repo path, but assuming current behavior:
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	return tmpDir
}

func setupTemplates(t *testing.T, tmpDir string) {
	docsDir := filepath.Join(tmpDir, "docs", "ai")
	if err := os.MkdirAll(docsDir, 0755); err != nil {
		t.Fatal(err)
	}

	templates := map[string]string{
		"PUBLISH_TEMPLATE.md":      "Title: Release vX.Y.Z\n",
		"RELEASE_BODY_TEMPLATE.md": "Body for vX.Y.Z...",
		"X_TEMPLATE.md":            "Tweet vX.Y.Z!",
	}
	for name, content := range templates {
		if err := os.WriteFile(filepath.Join(docsDir, name), []byte(content), 0644); err != nil {
			t.Fatal(err)
		}
	}

	gitCommit(t, tmpDir, "Add templates")
}

func setupChanges(t *testing.T, tmpDir string) {
	// Add a change for ai-pack to pick up
	if err := os.WriteFile(filepath.Join(tmpDir, "foo.txt"), []byte("bar"), 0644); err != nil {
		t.Fatal(err)
	}
	gitCommit(t, tmpDir, "Change")
}

func gitCommit(t *testing.T, dir, msg string) {
	git := func(args ...string) {
		cmd := exec.Command("git", args...)
		cmd.Dir = dir
		cmd.Env = append(os.Environ(),
			"GIT_AUTHOR_NAME=Test", "GIT_AUTHOR_EMAIL=test@example.com",
			"GIT_COMMITTER_NAME=Test", "GIT_COMMITTER_EMAIL=test@example.com",
		)
		if err := cmd.Run(); err != nil {
			t.Fatalf("git %v failed: %v", args, err)
		}
	}
	git("add", ".")
	git("commit", "-m", msg)
}

func verifyDraftsOutput(t *testing.T, tmpDir, outDir, version string) {
	expectedDir := filepath.Join(tmpDir, "dist", "publish", version)

	// Normalize paths for macOS /private/var symlink issues
	if realOut, err := filepath.EvalSymlinks(outDir); err == nil {
		outDir = realOut
	}
	if realExpected, err := filepath.EvalSymlinks(expectedDir); err == nil {
		expectedDir = realExpected
	}

	if outDir != expectedDir {
		t.Errorf("expected outDir %q, got %q", expectedDir, outDir)
	}

	// Check files exist
	files := []string{
		"PUBLISH_v1.2.3.md",
		"RELEASE_BODY_v1.2.3.md",
		"X_v1.2.3.md",
		"AI_PACK_v1.2.3.txt",
	}
	for _, f := range files {
		path := filepath.Join(outDir, f)
		if _, err := os.Stat(path); os.IsNotExist(err) {
			t.Errorf("expected file %q created, but missing", f)
		} else {
			verifyFileContent(t, path, f, version)
		}
	}
}

func verifyFileContent(t *testing.T, path, filename, version string) {
	if strings.HasSuffix(filename, ".md") {
		b, _ := os.ReadFile(path)
		content := string(b)
		if !strings.Contains(content, version) {
			t.Errorf("file %q missing version %q: %q", filename, version, content)
		}
		if strings.Contains(content, "vX.Y.Z") {
			t.Errorf("file %q still has placeholder vX.Y.Z: %q", filename, content)
		}
	}
}
