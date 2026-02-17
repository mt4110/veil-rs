package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// InitRepo initializes a new git repository in the given directory.
// It sets up user.name and user.email to ensure commits work.
// It also sets `init.defaultBranch` to `main`.
func InitRepo(t *testing.T, dir string) {
	t.Helper()
	mustRunGit(t, dir, "init")
	mustRunGit(t, dir, "config", "user.name", "Test User")
	mustRunGit(t, dir, "config", "user.email", "test@example.com")
	mustRunGit(t, dir, "config", "init.defaultBranch", "main")
}

// CommitFile creates or updates a file and commits it.
func CommitFile(t *testing.T, dir, filename, content, msg string) {
	t.Helper()
	path := filepath.Join(dir, filename)
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		t.Fatalf("failed to create dir for %s: %v", filename, err)
	}
	if err := os.WriteFile(path, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write %s: %v", filename, err)
	}
	mustRunGit(t, dir, "add", filename)
	mustRunGit(t, dir, "commit", "-m", msg)
}

// MakeBranch creates a new branch from the current HEAD.
func MakeBranch(t *testing.T, dir, name string) {
	t.Helper()
	mustRunGit(t, dir, "branch", name)
}

// Tag creates a lightweight tag.
func Tag(t *testing.T, dir, name string) {
	t.Helper()
	mustRunGit(t, dir, "tag", name)
}

// GetHeadSHA returns the full SHA of the current HEAD.
func GetHeadSHA(t *testing.T, dir string) string {
	t.Helper()
	cmd := exec.Command("git", "rev-parse", "HEAD")
	cmd.Dir = dir
	out, err := cmd.Output()
	if err != nil {
		t.Fatalf("failed to get HEAD SHA: %v", err)
	}
	return strings.TrimSpace(string(out))
}

func mustRunGit(t *testing.T, dir string, args ...string) {
	t.Helper()
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("git %v failed: %v\nOutput: %s", args, err, out)
	}
}
