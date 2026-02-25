package main

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCapsule_StrictRitual(t *testing.T) {
	// Helper to create hermetic repo
	repoDir, _ := forgeHermeticRepo(t)
	// Add .gitignore for .local
	if err := os.WriteFile(filepath.Join(repoDir, ".gitignore"), []byte(".local/\n"), 0644); err != nil {
		t.Fatalf("failed to write .gitignore: %v", err)
	}
	mustRunGit(t, repoDir, "add", ".gitignore")
	mustRunGit(t, repoDir, "commit", "-m", "ignore .local")
	headSHA := GetHeadSHA(t, repoDir)

	outDir := t.TempDir()

	// Helper to run CreateBundleUI and return stdout/stderr
	// Helper to run CreateBundleUI and return stdout, stderr, and any error
	runCapsule := func(heavy string, autocommit bool, msg string) (string, string, error) {
		var stdout, stderr bytes.Buffer
		err := CreateBundleUI("strict", outDir, repoDir, heavy, autocommit, msg, "", &stdout, &stderr)
		return stdout.String(), stderr.String(), err
	}

	t.Run("Clean_NoEvidence_HeavyNever_Skips", func(t *testing.T) {
		stdout, _, err := runCapsule("never", false, "")
		if err == nil || !strings.Contains(err.Error(), "evidence_required mode=strict") {
			t.Errorf("Expected evidence_required error, got: %v", err)
		}
		if !strings.Contains(stdout, "OK: HEAD_START=") {
			t.Errorf("Expected HEAD_START in stdout, got:\n%s", stdout)
		}
		if strings.Contains(stdout, "Bundle created:") {
			t.Error("Bundle should not be created")
		}
	})

	t.Run("Clean_EvidencePresent_Succeeds", func(t *testing.T) {
		// Create valid evidence
		evDir := filepath.Join(repoDir, ".local", "prverify")
		os.MkdirAll(evDir, 0755)
		evFile := filepath.Join(evDir, fmt.Sprintf("prverify_20240101_%s.md", headSHA))
		if err := os.WriteFile(evFile, []byte("Report with "+headSHA), 0644); err != nil {
			t.Fatalf("failed to write evidence file: %v", err)
		}

		stdout, _, err := runCapsule("never", false, "")
		if err != nil {
			t.Fatalf("Expected success, got error: %v", err)
		}
		if !strings.Contains(stdout, "OK: evidence_report=") {
			t.Errorf("Expected evidence found, got stdout:\n%s", stdout)
		}
		if !strings.Contains(stdout, "Bundle created:") {
			t.Errorf("Expected bundle created, got stdout:\n%s", stdout)
		}
	})

	t.Run("Dirty_NoAutocommit_Skips", func(t *testing.T) {
		// Make dirty
		if err := os.WriteFile(filepath.Join(repoDir, "dirty.txt"), []byte("dirty"), 0644); err != nil {
			t.Fatalf("failed to write dirty.txt: %v", err)
		}
		// Don't stage
		// Wait, dirty check uses `git status --porcelain`. Untracked files counts as dirty?
		// Usually yes.
		// `isGitDirty` helper uses `git status --porcelain`.
		// Untracked files show up as `??`.

		stdout, _, err := runCapsule("never", false, "")
		if err == nil || !strings.Contains(err.Error(), "repo dirty") {
			t.Errorf("Expected repo dirty error, got: %v", err)
		}
		if !strings.Contains(stdout, "INFO: repo dirty") {
			t.Errorf("Expected repo dirty info, got stdout:\n%s", stdout)
		}
		if !strings.Contains(stdout, "SKIP: strict create") {
			t.Errorf("Expected SKIP, got stdout:\n%s", stdout)
		}
	})

	t.Run("Dirty_Autocommit_Unstaged_Skips", func(t *testing.T) {
		// Still dirty from previous test (untracked file).
		// Try autocommit.
		_, _, _ = runCapsule("never", true, "wip")

		// `hasUnstagedChanges` uses `git diff --name-only`.
		// Untracked files are NOT shown in `git diff --name-only`.
		// But `isGitDirty` checks porcelain which includes `??`.
		// So `isDirty` is true.
		// `hasUnstaged` checks `git diff --name-only` (modified but not staged).
		// If only untracked, `hasUnstaged` is false.
		// Then it tries `git commit -m`.
		// `git commit` fails if nothing added.

		// Let's make it modified (not untracked) to trigger `hasUnstaged`.
		mustRunGit(t, repoDir, "add", "dirty.txt")
		mustRunGit(t, repoDir, "commit", "-m", "clean state")
		if err := os.WriteFile(filepath.Join(repoDir, "dirty.txt"), []byte("dirty modified"), 0644); err != nil {
			t.Fatalf("failed to update dirty.txt: %v", err)
		}

		stdout, _, err := runCapsule("never", true, "wip")
		if err == nil || !strings.Contains(err.Error(), "unstaged changes exist") {
			t.Errorf("Expected unstaged changes error, got: %v", err)
		}
		if !strings.Contains(stdout, "SKIP: strict create") {
			t.Errorf("Expected SKIP, got stdout:\n%s", stdout)
		}
	})

	t.Run("Dirty_Autocommit_Staged_Succeeds", func(t *testing.T) {
		// Stage the change
		mustRunGit(t, repoDir, "add", "dirty.txt")

		// Now it is staged. `isDirty` is true. `hasUnstaged` is false.
		// Commit should succeed.
		// But verify evidence will fail because new commit has new SHA, and we have no evidence for NEW sha.
		// So it should commit, print HEAD_NOW, then fail on missing evidence (since heavy=never).

		stdout, _, err := runCapsule("never", true, "auto commit msg")
		if err == nil || !strings.Contains(err.Error(), "evidence_required mode=strict") {
			t.Errorf("Expected evidence failure for NEW head, got: %v", err)
		}
		if !strings.Contains(stdout, "OK: committed; HEAD_NOW=") {
			t.Errorf("Expected commit success, got stdout:\n%s", stdout)
		}
	})
}
