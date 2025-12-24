package cockpit_test

import (
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"testing"

	"veil-rs/internal/cockpit"
)

func TestGenerateAIPack_Golden(t *testing.T) {
	// 1. Setup minimal git repo in temp dir
	tmpDir, err := os.MkdirTemp("", "cockpit-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	git := func(args ...string) {
		cmd := exec.Command("git", args...)
		cmd.Dir = tmpDir
		cmd.Env = append(os.Environ(),
			"GIT_AUTHOR_NAME=Test User",
			"GIT_AUTHOR_EMAIL=test@example.com",
			"GIT_AUTHOR_DATE=2024-01-01T12:00:00Z",
			"GIT_COMMITTER_NAME=Test User",
			"GIT_COMMITTER_EMAIL=test@example.com",
			"GIT_COMMITTER_DATE=2024-01-01T12:00:00Z",
		)
		if out, err := cmd.CombinedOutput(); err != nil {
			t.Fatalf("git %v failed: %v\n%s", args, err, out)
		}
	}

	git("init", "-b", "main")
	git("config", "user.name", "Test User")
	git("config", "user.email", "test@example.com")

	// Initial commit
	os.WriteFile(filepath.Join(tmpDir, "README.md"), []byte("Hello World\n"), 0644)
	git("add", ".")
	git("commit", "-m", "Initial commit")

	// Change file
	os.WriteFile(filepath.Join(tmpDir, "README.md"), []byte("Hello World\nLine 2\nLine 3\n"), 0644)
	git("add", ".")
	git("commit", "-m", "Update README")

	// Create ignore dir for stability
	os.MkdirAll(filepath.Join(tmpDir, "dist"), 0755)
	os.WriteFile(filepath.Join(tmpDir, ".gitignore"), []byte("dist/\n"), 0644)
	git("add", ".gitignore")
	git("commit", "-m", "Add gitignore")

	// Set repo root for GitX (since GenerateAIPack relies on rev-parse --show-toplevel)
	// We need to change cwd to the temp repo for the test to work naturally,
	// OR we assume GenerateAIPack works if run from within the repo.
	// Since GenerateAIPack runs `rev-parse --show-toplevel`, it needs CWD to be inside.
	wd, _ := os.Getwd()
	defer os.Chdir(wd)
	if err := os.Chdir(tmpDir); err != nil {
		t.Fatal(err)
	}

	// 2. Run GenerateAIPack
	outPath := filepath.Join(tmpDir, "dist", "AI_PACK.txt")
	// Using HEAD~1 as base to show the "Update README" and "Add gitignore" changes
	finalPath, usedTemp, err := cockpit.GenerateAIPack("HEAD~2", outPath)
	if err != nil {
		t.Fatalf("GenerateAIPack failed: %v", err)
	}
	if usedTemp {
		t.Error("expected usedTemp=false when outPath provided")
	}
	if finalPath != outPath {
		t.Errorf("expected finalPath=%q, got %q", outPath, finalPath)
	}

	// 3. Normalize & Verify
	contentBytes, err := os.ReadFile(outPath)
	if err != nil {
		t.Fatal(err)
	}
	content := string(contentBytes)
	normalized := normalizeAIPack(content)

	// We expect specific content. Instead of a separate file, we can assert key parts or compare snapshot.
	// For simplicity, let's verify key structure and normalized content match regex.

	mustContain := []string{
		"=== AI_PACK ===",
		"base_ref: HEAD~2",
		"head: <HASH>",
		"=== STATUS ===",
		"=== SUMMARY ===",
		"=== CHANGED FILES (BASE..HEAD) ===",
		"M\tREADME.md",
		"=== CONTEXT_MAP ===",
		"---- FILE: .gitignore ----",
		"---- FILE: README.md ----",
		"[around changes]",
		"     1\tdist/",
		"     1\tHello World",
		"     2\tLine 2",
		"=== END ===",
	}

	for _, s := range mustContain {
		if !regexp.MustCompile(regexp.QuoteMeta(s)).MatchString(normalized) {
			t.Errorf("Missing expected content part: %q\n--- Content ---\n%s", s, normalized)
		}
	}
}

// normalizeAIPack replaces volatile fields with fixed placeholders
func normalizeAIPack(s string) string {
	// generated_at_utc: ... -> generated_at_utc: <TIMESTAMP>
	s = regexp.MustCompile(`generated_at_utc: .*`).ReplaceAllString(s, "generated_at_utc: <TIMESTAMP>")
	// head: ... -> head: <HASH>
	s = regexp.MustCompile(`head: [a-f0-9]+`).ReplaceAllString(s, "head: <HASH>")
	// commit hashes in log or status
	// (Simple approach: look for known patterns or just trust the structure check above)

	// Normalize branch name if needed (git init -b main handles it, but local default might vary if git version old)
	// We forced -b main, so branch should be main.

	return s
}
