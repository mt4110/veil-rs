package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestValidateStatusEnforcement(t *testing.T) {
	// Setup temp git repo
	tmpDir, err := os.MkdirTemp("", "prverify-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	runGit := func(args ...string) {
		cmd := exec.Command("git", args...)
		cmd.Dir = tmpDir
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			t.Fatalf("git %v failed: %v", args, err)
		}
	}

	runGit("init")
	runGit("config", "user.email", "test@example.com")
	runGit("config", "user.name", "Test User")
	runGit("commit", "--allow-empty", "-m", "initial commit")
	runGit("branch", "-m", "main")

	// Create a base ref (main)
	err = os.WriteFile(filepath.Join(tmpDir, "README.md"), []byte("hello"), 0644)
	if err != nil {
		t.Fatal(err)
	}
	runGit("add", "README.md")
	runGit("commit", "-m", "add readme")

	// Case 1: Non-S11 branch (should skip)
	// We need to manipulate env var or checkout
	// validateStatusEnforcement checks GITHUB_HEAD_REF first
	t.Setenv("GITHUB_HEAD_REF", "feature/foo")
	if err := validateStatusEnforcement(tmpDir); err != nil {
		t.Errorf("expected skip for non-S11 branch, got error: %v", err)
	}

	// For subsequent tests, switch to S11 branch
	t.Setenv("GITHUB_HEAD_REF", "s11-test")

	// Case 2: S11 branch, no changes vs base (should skip)
	runGit("checkout", "-b", "s11-test")
	if err := validateStatusEnforcement(tmpDir); err != nil {
		t.Errorf("expected skip for empty diff, got error: %v", err)
	}

	// Case 3: S11 branch, changes but missing STATUS.md (should fail)
	err = os.WriteFile(filepath.Join(tmpDir, "foo.txt"), []byte("bar"), 0644)
	if err != nil {
		t.Fatal(err)
	}
	runGit("add", "foo.txt")
	// Note: We don't commit yet? wait, diff is <baseRef>...HEAD
	// If uncommitted, diff base...HEAD might show changes if staged?
	// But usually diff base...HEAD compares commits.
	// If uncommitted changes exist, they are not in HEAD.
	// Wait, `git diff <base>...HEAD` shows difference between base and HEAD commit.
	// If I haven't committed "foo.txt", it won't show up in `git diff main...HEAD`.
	// So I must commit.
	runGit("commit", "-m", "add foo")

	err = validateStatusEnforcement(tmpDir)
	if err == nil {
		t.Error("expected error for missing STATUS.md, got nil")
	} else {
		// Check error message
		if err.Error() != "[S11 Discipline Drift] S11 requires STATUS.md update, but diff lacks docs/ops/STATUS.md" {
			t.Errorf("unexpected error message: %v", err)
		}
	}

	// Case 4: S11 branch, changes include STATUS.md (should pass)
	// Create docs/ops/STATUS.md
	opsDir := filepath.Join(tmpDir, "docs", "ops")
	if err := os.MkdirAll(opsDir, 0755); err != nil {
		t.Fatal(err)
	}
	err = os.WriteFile(filepath.Join(opsDir, "STATUS.md"), []byte("updated"), 0644)
	if err != nil {
		t.Fatal(err)
	}
	runGit("add", "docs/ops/STATUS.md")
	runGit("commit", "-m", "update status")

	if err := validateStatusEnforcement(tmpDir); err != nil {
		t.Errorf("expected pass when STATUS.md is updated, got error: %v", err)
	}
}
