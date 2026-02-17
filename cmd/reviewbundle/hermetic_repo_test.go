package main

import (
	"bytes"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// forgeHermeticRepo creates a temporary git repository with a predictable history.
// It returns the repository directory path and the base commit SHA.
// It ensures the environment is isolated from the host (no global config, fixed time/user).
func forgeHermeticRepo(t *testing.T) (string, string) {
	// 1. Create Git Repo Dir
	repoDir := t.TempDir()

	// 2. Setup Hermetic Environment wrapper
	runGit := func(args ...string) string {
		cmd := exec.Command("git", args...)
		cmd.Dir = repoDir

		// Unset host variables and enforce hermetic ones
		cmd.Env = []string{
			"PATH=" + os.Getenv("PATH"),  // Keep PATH to find git
			"HOME=" + repoDir,            // Fake HOME
			"XDG_CONFIG_HOME=" + repoDir, // Fake XDG
			"GIT_CONFIG_GLOBAL=/dev/null",
			"GIT_CONFIG_SYSTEM=/dev/null",
			"GIT_TERMINAL_PROMPT=0",
			"TZ=UTC",
			"LC_ALL=C",
			"GIT_AUTHOR_NAME=ci",
			"GIT_AUTHOR_EMAIL=ci@example.invalid",
			"GIT_AUTHOR_DATE=1700000000 +0000",
			"GIT_COMMITTER_NAME=ci",
			"GIT_COMMITTER_EMAIL=ci@example.invalid",
			"GIT_COMMITTER_DATE=1700000000 +0000",
		}

		out, err := cmd.CombinedOutput()
		if err != nil {
			t.Fatalf("git %v failed: %v\nOutput: %s", args, err, out)
		}
		return string(bytes.TrimSpace(out))
	}

	// 3. Init
	runGit("init")
	// Ensure main branch
	runGit("symbolic-ref", "HEAD", "refs/heads/main")

	// 4. Commit A (Base)
	if err := os.WriteFile(filepath.Join(repoDir, "file.txt"), []byte("base content\n"), 0644); err != nil {
		t.Fatal(err)
	}
	runGit("add", "file.txt")
	runGit("commit", "-m", "base commit")
	baseSHA := runGit("rev-parse", "HEAD")

	// 5. Commit B (Head)
	// We need 1 second gap for log ordering if we relied on time, but we use fixed time.
	// Actually, we reuse same time, but order is preserved by parent/child relationship.
	if err := os.WriteFile(filepath.Join(repoDir, "file.txt"), []byte("base content\nnew content\n"), 0644); err != nil {
		t.Fatal(err)
	}
	runGit("add", "file.txt")
	runGit("commit", "-m", "head commit")

	return repoDir, baseSHA
}

func TestForge_Smoke(t *testing.T) {
	repoDir, baseSHA := forgeHermeticRepo(t)
	if _, err := os.Stat(filepath.Join(repoDir, ".git")); err != nil {
		t.Error(".git missing")
	}
	if len(baseSHA) != 40 {
		t.Errorf("invalid baseSHA: %s", baseSHA)
	}

	// Check if we can run git log
	cmd := exec.Command("git", "log", "--oneline")
	cmd.Dir = repoDir
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(out), "head commit") {
		t.Error("head commit missing")
	}
}
