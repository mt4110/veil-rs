package main

import (
	"bytes"
	"flag"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"strings"
	"time"
)

type stepResult struct {
	cmdLine  string
	ok       bool
	duration time.Duration
}

func runStreaming(dir string, name string, args ...string) (time.Duration, error) {
	start := time.Now()
	cmd := exec.Command(name, args...)
	cmd.Dir = dir
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	err := cmd.Run()
	return time.Since(start), err
}

func runCapture(dir string, name string, args ...string) string {
	cmd := exec.Command(name, args...)
	cmd.Dir = dir
	var buf bytes.Buffer
	cmd.Stdout = &buf
	cmd.Stderr = &buf
	if err := cmd.Run(); err != nil {
		return ""
	}
	return strings.TrimSpace(buf.String())
}

func bestEffortRepoRoot() string {
	root := runCapture("", "git", "rev-parse", "--show-toplevel")
	if root != "" {
		return root
	}
	wd, err := os.Getwd()
	if err != nil {
		return "."
	}
	return wd
}

func fmtDur(d time.Duration) string {
	if d >= time.Second {
		return fmt.Sprintf("%.2fs", d.Seconds())
	}
	return fmt.Sprintf("%dms", d.Milliseconds())
}

func renderMarkdown(rustcV, cargoV, gitSHA, gitDirty string, steps []stepResult) string {
	var b strings.Builder

	b.WriteString("-=======\n")
	b.WriteString("Notes / Evidence\n\n")

	b.WriteString("Local env:\n")
	if rustcV != "" {
		b.WriteString(fmt.Sprintf("- rustc: %s\n", rustcV))
	}
	if cargoV != "" {
		b.WriteString(fmt.Sprintf("- cargo: %s\n", cargoV))
	}
	if gitSHA != "" {
		if gitDirty == "" {
			b.WriteString(fmt.Sprintf("- git: %s (clean)\n", gitSHA))
		} else {
			b.WriteString(fmt.Sprintf("- git: %s (dirty)\n", gitSHA))
		}
	}

	b.WriteString("\nTests:\n")
	for _, s := range steps {
		status := "OK"
		if !s.ok {
			status = "FAIL"
		}
		b.WriteString(fmt.Sprintf("- `%s` => %s (%s)\n", s.cmdLine, status, fmtDur(s.duration)))
	}

	b.WriteString("\nRollback\n\n")
	b.WriteString("Revert the merge/squash commit for this PR.\n")
	b.WriteString("- Squash merge: `git revert <commit_sha>`\n")
	b.WriteString("- Merge commit: `git revert -m 1 <merge_commit_sha>`\n")
	b.WriteString("-=======\n")

	return b.String()
}

func main() {
	smokeOnly := flag.Bool("smoke-only", false, "run only the P0 CLI smoke suite (trycmd)")
	flag.Parse()

	root := bestEffortRepoRoot()

	rustcV := runCapture(root, "rustc", "-V")
	cargoV := runCapture(root, "cargo", "-V")
	gitSHA := runCapture(root, "git", "rev-parse", "--short=12", "HEAD")
	gitDirty := runCapture(root, "git", "status", "--porcelain")

	steps := []stepResult{}

	// 1) P0 smoke suite
	{
		cmdLine := "cargo test -p veil-cli --test cli_tests"
		fmt.Printf("==> %s\n", cmdLine)
		dur, err := runStreaming(root, "cargo", "test", "-p", "veil-cli", "--test", "cli_tests")
		steps = append(steps, stepResult{cmdLine: cmdLine, ok: err == nil, duration: dur})
		if err != nil {
			fmt.Fprintln(os.Stderr, "ERROR: smoke suite failed:", err)
			fmt.Println()
			fmt.Print(renderMarkdown(rustcV, cargoV, gitSHA, gitDirty, steps))
			os.Exit(1)
		}
	}

	// 2) Workspace tests (optional)
	if !*smokeOnly {
		cmdLine := "cargo test --workspace"
		fmt.Printf("==> %s\n", cmdLine)
		dur, err := runStreaming(root, "cargo", "test", "--workspace")
		steps = append(steps, stepResult{cmdLine: cmdLine, ok: err == nil, duration: dur})
		if err != nil {
			fmt.Fprintln(os.Stderr, "ERROR: workspace tests failed:", err)
			fmt.Println()
			fmt.Print(renderMarkdown(rustcV, cargoV, gitSHA, gitDirty, steps))
			os.Exit(1)
		}
	}

	// 3) Drift Check (Consistency between CI, Docs, and SOT)
	{
		fmt.Println("==> Drift Check")
		start := time.Now()
		// Use os.DirFS for real execution
		repoFS := os.DirFS(root)
		err := validateDrift(repoFS)
		dur := time.Since(start)
		steps = append(steps, stepResult{cmdLine: "drift-check", ok: err == nil, duration: dur})
		if err != nil {
			fmt.Fprintln(os.Stderr, "ERROR: drift check failed:", err)
			fmt.Println()
			fmt.Print(renderMarkdown(rustcV, cargoV, gitSHA, gitDirty, steps))
			os.Exit(1)
		}
	}

	fmt.Println()
	fmt.Print(renderMarkdown(rustcV, cargoV, gitSHA, gitDirty, steps))
}

func validateDrift(repoFS fs.FS) error {
	// A. CI Check (.github/workflows/ci.yml)
	// Note: in fs.FS paths are always slash-separated and relative to root (no leading period/slash)
	ciPath := ".github/workflows/ci.yml"
	ciContent, err := fs.ReadFile(repoFS, ciPath)
	if err != nil {
		return fmt.Errorf("failed to read CI config: %w", err)
	}
	ciStr := string(ciContent)

	checks := []struct {
		name  string
		check func(string) bool
		err   string
	}{
		{"Install Script", func(s string) bool { return strings.Contains(s, "ops/ci/install_sqlx_cli.sh") }, "CI must use ops/ci/install_sqlx_cli.sh"},
		{"Log Generation", func(s string) bool {
			return strings.Contains(s, ".local/ci/sqlx_cli_install.log") && strings.Contains(s, ".local/ci/sqlx_prepare_check.txt")
		}, "CI must generate specific log files"},
		{"Artifact Upload", func(s string) bool {
			return strings.Contains(s, "actions/upload-artifact") && strings.Contains(s, "path:") && strings.Contains(s, ".local/ci/")
		}, "CI must upload .local/ci/ as artifacts via upload-artifact action"},
		{"Keep File", func(s string) bool { return strings.Contains(s, ".local/ci/.keep") }, "CI must create .local/ci/.keep"},
	}

	for _, c := range checks {
		if !c.check(ciStr) {
			return fmt.Errorf("CI Drift: %s", c.err)
		}
	}

	// B. Docs Check (docs/guardrails/sqlx.md, docs/ci/prverify.md)
	docsFiles := []string{
		"docs/guardrails/sqlx.md",
		"docs/ci/prverify.md",
	}

	docChecks := []struct {
		name string
		term string
	}{
		{"SQLX_OFFLINE", "SQLX_OFFLINE"},
		{"Install Log", "sqlx_cli_install.log"},
		{"Shell Script Exception", "ops/ci/"},
	}

	foundDocs := make(map[string]bool)
	for _, f := range docsFiles {
		content, err := fs.ReadFile(repoFS, f)
		if err == nil {
			s := string(content)
			for _, dc := range docChecks {
				if strings.Contains(s, dc.term) {
					foundDocs[dc.name] = true
				}
			}
		}
	}

	// We just need these terms to appear *somewhere* in the doc set
	for _, dc := range docChecks {
		if !foundDocs[dc.name] {
			return fmt.Errorf("Docs Drift: Term '%s' not found in %v", dc.term, docsFiles)
		}
	}

	// C. SOT Check (docs/pr/PR-<number>-*.md)
	// Deterministic selection
	sotPath, err := findSOT(repoFS)
	if err != nil {
		return fmt.Errorf("SOT Drift: %w", err)
	}

	// Read and verify SOT content
	content, err := fs.ReadFile(repoFS, sotPath)
	if err != nil {
		return fmt.Errorf("SOT Drift: failed to read %s: %w", sotPath, err)
	}
	s := string(content)
	if !strings.Contains(s, "sqlx_cli_install.log") || !strings.Contains(s, "SQLX_OFFLINE") {
		return fmt.Errorf("SOT Drift: %s missing required evidence/policy keywords", sotPath)
	}

	return nil
}

// findSOT locates the Source of Truth file deterministically.
// Rules:
// 1. Must be in docs/pr/
// 2. Must match PR-\d+-*.md
// 3. Must be unique (ambiguity => error)
func findSOT(repoFS fs.FS) (string, error) {
	entries, err := fs.ReadDir(repoFS, "docs/pr")
	if err != nil {
		return "", fmt.Errorf("failed to read docs/pr: %w", err)
	}

	var candidates []string
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		name := e.Name()
		// Simple check: starts with PR-, contains digits, ends with .md
		// We avoid complex regex to keep it stdlib-lite if possible, but path.Match or logic is fine.
		// Strict format: PR-<digits>-<desc>.md
		if strings.HasPrefix(name, "PR-") && strings.HasSuffix(name, ".md") {
			// Check for digits after PR-
			rest := strings.TrimPrefix(name, "PR-")
			if len(rest) > 0 && rest[0] >= '0' && rest[0] <= '9' {
				candidates = append(candidates, "docs/pr/"+name)
			}
		}
	}

	if len(candidates) == 0 {
		return "", fmt.Errorf("sot_missing: no PR-XX-*.md files found in docs/pr/")
	}
	if len(candidates) > 1 {
		return "", fmt.Errorf("sot_ambiguous: multiple candidates found: %v", candidates)
	}

	return candidates[0], nil
}
