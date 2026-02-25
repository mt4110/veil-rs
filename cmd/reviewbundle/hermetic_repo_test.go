package main

import (
	"bytes"
	"fmt"
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

func TestCreate_StrictLocalEvidence(t *testing.T) {
	// 1. Setup Repo
	repoDir, _ := forgeHermeticRepo(t)

	// helper to run git inside repo
	gitEnv := []string{
		"PATH=" + os.Getenv("PATH"),
		"HOME=" + repoDir,
		"XDG_CONFIG_HOME=" + repoDir,
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

	execGit := func(args ...string) string {
		cmd := exec.Command("git", args...)
		cmd.Dir = repoDir
		cmd.Env = gitEnv
		out, err := cmd.CombinedOutput()
		if err != nil {
			t.Fatalf("git %v failed: %v\nOutput: %s", args, err, out)
		}
		return strings.TrimSpace(string(out))
	}

	ignorePath := filepath.Join(repoDir, ".gitignore")
	if err := os.WriteFile(ignorePath, []byte(".local/\n"), 0644); err != nil {
		t.Fatal(err)
	}
	execGit("add", ".gitignore")
	execGit("commit", "-m", "ignore .local")
	headSHA := execGit("rev-parse", "HEAD")

	// 3. Create .local/prverify with matching SHA
	localEvDir := filepath.Join(repoDir, ".local", "prverify")
	if err := os.MkdirAll(localEvDir, 0755); err != nil {
		t.Fatal(err)
	}

	// Create older non-matching evidence
	oldReport := filepath.Join(localEvDir, "prverify_20250101T000000Z_aaaaaaa.md")
	if err := os.WriteFile(oldReport, []byte("old report sans sha"), 0644); err != nil {
		t.Fatal(err)
	}

	// Create newer matching evidence
	newReport := filepath.Join(localEvDir, "prverify_20260218T120000Z_bbbbbbb.md")
	// Must contain full SHA
	reportContent := fmt.Sprintf("# Report\n\n- head_sha: %s\n", headSHA)
	if err := os.WriteFile(newReport, []byte(reportContent), 0644); err != nil {
		t.Fatal(err)
	}

	// 4. Create Bundle (Strict)
	c := &Contract{
		ContractVersion: "1.1",
		Mode:            "strict",
		Repo:            "veil-rs",
		EpochSec:        1700000000,
		BaseRef:         "main", // assume main exists
		HeadSHA:         headSHA,
		Evidence: Evidence{
			Required:   true,
			PathPrefix: "review/evidence/",
		},
		Tool: Tool{Name: "test", Version: "0.0.0"},
	}

	outDir := t.TempDir()
	bundlePath, err := CreateBundle(c, outDir, repoDir, "")
	if err != nil {
		t.Fatalf("CreateBundle failed: %v", err)
	}

	// 5. Verify Bundle Contains Evidence (via VerifyBundlePath)
	rep, err := VerifyBundlePath(bundlePath)
	if err != nil {
		t.Fatalf("VerifyBundlePath failed: %v", err)
	}
	if !rep.EvidenceBoundToHead {
		t.Error("EvidenceBoundToHead is false, expected true")
	}
	if !rep.EvidencePresent {
		t.Error("EvidencePresent is false, expected true")
	}
	// Check if correct file name is included?
	// VerifyReport doesn't expose file list easily in public struct except via ComputedSHA256 keys or EvidenceFiles contents.
	// We trust EvidenceBoundToHead=true implies it found the SHA.
}
