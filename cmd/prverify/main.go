package main

import (
	"bytes"
	"flag"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"sync"
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

	b.WriteString("## Notes / Evidence\n\n")

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
	allOK := true
	for _, s := range steps {
		status := "OK"
		if !s.ok {
			status = "FAIL"
			allOK = false
		}
		b.WriteString(fmt.Sprintf("- `%s` => %s (%s)\n", s.cmdLine, status, fmtDur(s.duration)))
	}

	b.WriteString("\n## Rollback\n\n")
	b.WriteString("Revert the merge/squash commit for this PR.\n")
	b.WriteString("```bash\n")
	b.WriteString("# 1コミットだけ戻す\n")
	b.WriteString("git revert <commit>\n\n")
	b.WriteString("# 範囲でまとめて戻す\n")
	b.WriteString("git revert <oldest_commit>^..<newest_commit>\n")
	b.WriteString("```\n")

	if allOK {
		b.WriteString("\n- Local run: PASS\n")
	} else {
		b.WriteString("\n- Local run: FAIL\n")
	}

	return b.String()
}

type driftError struct {
	category string
	reason   string
	fixCmd   string
	nextCmd  string
}

func (e *driftError) Error() string {
	return fmt.Sprintf("[%s Drift] %s", e.category, e.reason)
}

func (e *driftError) Print() {
	nextCmd := e.nextCmd
	if nextCmd == "" {
		nextCmd = "nix run .#prverify"
	}

	// Plain text output (no ANSI) - strictly for NO_COLOR or just cleaner logs
	if os.Getenv("NO_COLOR") != "" {
		fmt.Fprintf(os.Stderr, "Reason: %s\n", e.reason)
		fmt.Fprintf(os.Stderr, "Fix:    %s\n", e.fixCmd)
		fmt.Fprintf(os.Stderr, "Next:   %s\n", nextCmd)
		return
	}

	// ANSI output
	// Reason: <reason>
	// Fix:    <cmd>
	// Next:   <cmd>
	fmt.Fprintf(os.Stderr, "\x1b[1mReason:\x1b[0m %s\n", e.reason)
	fmt.Fprintf(os.Stderr, "\x1b[1mFix:\x1b[0m    \x1b[32m%s\x1b[0m\n", e.fixCmd)
	fmt.Fprintf(os.Stderr, "\x1b[1mNext:\x1b[0m   \x1b[34m%s\x1b[0m\n", nextCmd)
}

func main() {
	smokeOnly := flag.Bool("smoke-only", false, "run only the P0 CLI smoke suite (trycmd)")
	wantedPR := flag.Int("wanted-pr", 0, "prefer docs/pr/PR-<N>-*.md when selecting SOT (0 = auto)")
	mode := flag.String("mode", "full", "run mode (full, local-fast)")
	parallel := flag.Int("parallel", 1, "number of parallel jobs (default 1, max 2)")
	cacheDir := flag.String("cache-dir", ".local/cache", "cache directory")
	flag.Parse()

	if *parallel > 2 {
		fmt.Fprintf(os.Stderr, "ERROR: parallel_too_high capped_to=2\n")
		*parallel = 2
	}

	tsUTC := time.Now().UTC().Format("20060102T150405Z")
	obsDir := fmt.Sprintf(".local/obs/s12-05-5_prverify_%s", tsUTC)
	os.MkdirAll(obsDir, 0755)

	fmt.Printf("OK: obs_dir=%s\n", obsDir)
	fmt.Printf("OK: mode=%s parallel=%d\n", *mode, *parallel)

	root := bestEffortRepoRoot()

	rustcV := runCapture(root, "rustc", "-V")
	cargoV := runCapture(root, "cargo", "-V")
	gitSHA := runCapture(root, "git", "rev-parse", "HEAD")
	gitDirty := runCapture(root, "git", "status", "--porcelain")

	// 2. resolve_base
	baseSHA := ""
	if *mode == "local-fast" {
		baseSHA = runCapture(root, "git", "merge-base", "HEAD", "origin/main")
		if baseSHA == "" {
			baseSHA = runCapture(root, "git", "merge-base", "HEAD", "main")
		}
		if baseSHA == "" {
			fmt.Fprintf(os.Stderr, "ERROR: base_sha=missing fallback=full\n")
			*mode = "full"
		} else {
			fmt.Printf("OK: base_sha=%s\n", baseSHA)
		}
	}

	// 3. classify_changes
	touchGo, touchRust, touchOther := false, false, false
	skipHeavy := false
	if *mode == "local-fast" && baseSHA != "" {
		out, err := exec.Command("git", "diff", "--name-only", baseSHA+"..HEAD").Output()
		if err != nil {
			fmt.Fprintf(os.Stderr, "ERROR: diff_failed fallback=full\n")
			*mode = "full"
		} else {
			files := strings.Split(strings.TrimSpace(string(out)), "\n")
			if string(out) == "" {
				files = []string{}
			}
			touchDocs := false
			for _, f := range files {
				if f == "" {
					continue
				}
				if strings.HasSuffix(f, ".go") || f == "go.mod" || f == "go.sum" {
					touchGo = true
				} else if strings.HasSuffix(f, ".rs") || f == "Cargo.toml" || f == "Cargo.lock" || strings.HasPrefix(f, "rust-toolchain") || strings.HasPrefix(f, ".cargo/") {
					touchRust = true
				} else if strings.HasPrefix(f, "docs/") || strings.HasSuffix(f, ".md") {
					touchDocs = true
				} else if strings.HasSuffix(f, ".nix") || f == "flake.lock" {
					touchOther = true
				} else {
					touchOther = true
				}
			}

			fmt.Printf("OK: touch_go=%v touch_rust=%v touch_docs=%v touch_other=%v\n", touchGo, touchRust, touchDocs, touchOther)

			if touchOther {
				fmt.Fprintf(os.Stderr, "ERROR: unknown_changes fallback=full\n")
				*mode = "full"
			} else if !touchGo && !touchRust && touchDocs {
				skipHeavy = true
			}
		}
	}

	// 4. caching checks
	if *mode == "local-fast" || *mode == "full" {
		absCache, _ := filepath.Abs(*cacheDir)
		if absCache == "" {
			absCache = *cacheDir
		}
		os.MkdirAll(absCache, 0755)

		goCacheDir := filepath.Join(absCache, "go-build")
		cargoCacheDir := filepath.Join(absCache, "cargo-target")

		os.MkdirAll(goCacheDir, 0755)
		os.MkdirAll(cargoCacheDir, 0755)

		os.Setenv("GOCACHE", goCacheDir)
		os.Setenv("CARGO_TARGET_DIR", cargoCacheDir)
		fmt.Printf("OK: cache_go=%s cache_rust=%s\n", goCacheDir, cargoCacheDir)
	}

	var mu sync.Mutex
	var steps []stepResult
	var hasError bool

	addStep := func(s stepResult) {
		mu.Lock()
		defer mu.Unlock()
		steps = append(steps, s)
	}

	reportFail := func(s stepResult) {
		mu.Lock()
		defer mu.Unlock()
		hasError = true
		steps = append(steps, s)
	}

	runRustChecks := func() {
		if *mode == "local-fast" && !touchRust && !skipHeavy {
			fmt.Println("SKIP: rust checks (reason: touch_rust=false)")
			addStep(stepResult{cmdLine: "cargo checks (skip)", ok: true, duration: 0})
			return
		}
		if *mode == "local-fast" && skipHeavy {
			fmt.Println("SKIP: rust checks (reason: docs_only)")
			addStep(stepResult{cmdLine: "cargo checks (skip)", ok: true, duration: 0})
			return
		}

		// 1) P0 smoke suite
		cmdLine := "cargo test -p veil-cli --test cli_tests"
		fmt.Printf("==> %s\n", cmdLine)
		dur, err := runStreaming(root, "cargo", "test", "-p", "veil-cli", "--test", "cli_tests")
		res := stepResult{cmdLine: cmdLine, ok: err == nil, duration: dur}
		if err != nil {
			fmt.Fprintln(os.Stderr, "ERROR: smoke suite failed:", err)
			reportFail(res)
			return
		}
		addStep(res)

		// 2) Workspace tests
		if !*smokeOnly {
			cmdLine = "cargo test --workspace"
			fmt.Printf("==> %s\n", cmdLine)
			dur, err = runStreaming(root, "cargo", "test", "--workspace")
			res = stepResult{cmdLine: cmdLine, ok: err == nil, duration: dur}
			if err != nil {
				fmt.Fprintln(os.Stderr, "ERROR: workspace tests failed:", err)
				reportFail(res)
				return
			}
			addStep(res)
		}
	}

	runGoChecks := func() {
		if *mode == "local-fast" && !touchGo && !skipHeavy {
			fmt.Println("SKIP: go checks (reason: touch_go=false)")
			addStep(stepResult{cmdLine: "go checks (skip)", ok: true, duration: 0})
			return
		}
		if *mode == "local-fast" && skipHeavy {
			fmt.Println("SKIP: go checks (reason: docs_only)")
			addStep(stepResult{cmdLine: "go checks (skip)", ok: true, duration: 0})
			return
		}

		// 2.5) prkit tests
		cmdLine := "go test -count=1 ./cmd/prkit"
		fmt.Printf("==> %s\n", cmdLine)
		dur, err := runStreaming(root, "go", "test", "-count=1", "./cmd/prkit")
		res := stepResult{cmdLine: cmdLine, ok: err == nil, duration: dur}
		if err != nil {
			fmt.Fprintln(os.Stderr, "ERROR: prkit tests failed:", err)
			reportFail(res)
			return
		}
		addStep(res)

		// 3) Dependency Guard
		fmt.Println("==> Dependency Guard")
		start := time.Now()
		cmd := exec.Command("go", "run", "./cmd/dep-trace")
		cmd.Dir = root
		var buf bytes.Buffer
		cmd.Stdout = &buf
		cmd.Stderr = &buf
		err = cmd.Run()
		dur = time.Since(start)
		output := strings.TrimSpace(buf.String())
		res = stepResult{cmdLine: "dep-guard", ok: err == nil, duration: dur}
		if err != nil {
			fmt.Printf("FAIL: Dependency Guard found issues (or failed to run):\n\n%s\n\n", output)
			reportFail(res)
			return
		}
		addStep(res)
	}

	fastChecks := func() {
		// 4) Drift Check
		fmt.Println("==> Drift Check")
		start := time.Now()
		repoFS := os.DirFS(root)
		now := time.Now().UTC()
		utcToday := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, time.UTC)
		err := validateDrift(repoFS, *wantedPR, utcToday)
		dur := time.Since(start)
		res := stepResult{cmdLine: "drift-check", ok: err == nil, duration: dur}
		if err != nil {
			fmt.Println()
			if de, ok := err.(*driftError); ok {
				de.Print()
			} else {
				fmt.Fprintln(os.Stderr, "ERROR: drift check failed:", err)
			}
			fmt.Println()
			reportFail(res)
		} else {
			addStep(res)
		}

		// 5) Status Enforcement
		fmt.Println("==> Status Enforcement")
		start = time.Now()
		err = validateStatusEnforcement(root)
		dur = time.Since(start)
		res = stepResult{cmdLine: "status-enforcement", ok: err == nil, duration: dur}
		if err != nil {
			fmt.Println()
			if de, ok := err.(*driftError); ok {
				de.Print()
			} else {
				fmt.Fprintln(os.Stderr, "ERROR: status enforcement failed:", err)
			}
			fmt.Println()
			reportFail(res)
		} else {
			addStep(res)
		}
	}

	if *parallel == 2 {
		var wg sync.WaitGroup
		wg.Add(2)
		go func() {
			defer wg.Done()
			runRustChecks()
		}()
		go func() {
			defer wg.Done()
			runGoChecks()
		}()
		wg.Wait()
		fastChecks()
	} else {
		runRustChecks()
		if !hasError {
			runGoChecks()
		}
		if !hasError {
			fastChecks()
		}
	}

	obsFile := filepath.Join(obsDir, "step_durations.txt")
	var fObs *os.File
	if f, err := os.OpenFile(obsFile, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644); err == nil {
		fObs = f
	}

	for _, s := range steps {
		if fObs != nil {
			fmt.Fprintf(fObs, "OK: step=%q duration=%s\n", s.cmdLine, fmtDur(s.duration))
		}
	}
	if fObs != nil {
		fObs.Close()
	}
	fmt.Printf("OK: phase=end stop=0\n")

	fmt.Println()
	fmt.Print(renderMarkdown(rustcV, cargoV, gitSHA, gitDirty, steps))

	if hasError {
		os.Exit(1)
	}
}

func validateDrift(repoFS fs.FS, wantedPR int, utcToday time.Time) error {
	if err := validateCI(repoFS); err != nil {
		return err
	}
	if err := validateDocs(repoFS); err != nil {
		return err
	}
	if err := validateRegistryFile(repoFS, utcToday); err != nil {
		return err
	}
	return validateSOT(repoFS, wantedPR)
}

func validateCI(repoFS fs.FS) error {
	ciPath := ".github/workflows/ci.yml"
	ciContent, err := fs.ReadFile(repoFS, ciPath)
	if err != nil {
		return &driftError{
			category: "CI",
			reason:   fmt.Sprintf("failed to read CI config: %v", err),
			fixCmd:   "Ensure .github/workflows/ci.yml exists and is readable.",
		}
	}
	ciStr := string(ciContent)

	checks := []struct {
		name   string
		check  func(string) bool
		reason string
		fix    string
	}{
		{
			"Install Script",
			func(s string) bool { return strings.Contains(s, "ops/ci/install_sqlx_cli.sh") },
			"CI must use ops/ci/install_sqlx_cli.sh",
			"Edit .github/workflows/ci.yml to use ops/ci/install_sqlx_cli.sh",
		},
		{
			"Log Generation",
			func(s string) bool {
				return strings.Contains(s, ".local/ci/sqlx_cli_install.log") && strings.Contains(s, ".local/ci/sqlx_prepare_check.txt")
			},
			"CI must generate specific log files (.local/ci/sqlx_cli_install.log, .local/ci/sqlx_prepare_check.txt)",
			"Update ci.yml to redirect output to these log files.",
		},
		{
			"Keep File",
			func(s string) bool { return strings.Contains(s, ": > .local/ci/.keep") },
			"CI must create .local/ci/.keep to preserve the directory",
			"mkdir -p .local/ci && : > .local/ci/.keep", // Just illustrating the command needed, though this check is about the CI wrapper
		},
		{
			"Artifact Upload",
			func(s string) bool {
				return strings.Contains(s, "actions/upload-artifact") && strings.Contains(s, "path:") && strings.Contains(s, ".local/ci/")
			},
			"CI must upload .local/ci/ as artifacts",
			"Add actions/upload-artifact step for .local/ci/ directory.",
		},
	}

	for _, c := range checks {
		if !c.check(ciStr) {
			return &driftError{
				category: "CI",
				reason:   c.reason,
				fixCmd:   c.fix,
			}
		}
	}

	return nil
}

func validateDocs(repoFS fs.FS) error {
	docsFiles := []string{"docs/guardrails/sqlx.md", "docs/ci/prverify.md"}
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

	for _, dc := range docChecks {
		if !foundDocs[dc.name] {
			return &driftError{
				category: "Docs",
				reason:   fmt.Sprintf("Required term '%s' not found in %v", dc.term, docsFiles),
				fixCmd:   fmt.Sprintf("Edit one of %v to include '%s'", docsFiles, dc.term),
			}
		}
	}
	return nil
}

func validateSOT(repoFS fs.FS, wantedPR int) error {
	sotPath, err := findSOT(repoFS, wantedPR)
	if err != nil {
		if checkIgnore(repoFS, err) {
			fmt.Printf("WARN: [Ignored] %v\n", err)
			return nil
		}
		reason := err.Error()
		fix := "Check docs/pr/ directory and ensure exactly one SOT file (PR-XX-....md) exists."
		if strings.Contains(reason, "sot_ambiguous") {
			fix = "Remove duplicate SOT files for the same PR number in docs/pr/."
		} else if strings.Contains(reason, "sot_missing") {
			fix = "Create a SOT file in docs/pr/ (e.g., docs/pr/PR-38-epic-c.md)."
		}
		return &driftError{
			category: "SOT",
			reason:   reason,
			fixCmd:   fix,
		}
	}

	_, err = fs.ReadFile(repoFS, sotPath)
	if err != nil {
		if checkIgnore(repoFS, err) {
			return nil
		}
		return &driftError{
			category: "SOT",
			reason:   fmt.Sprintf("failed to read SOT %s: %v", sotPath, err),
			fixCmd:   fmt.Sprintf("chmod +r %s", sotPath),
		}
	}

	// Evidence check (SQLX check removed as per PR44 policy)
	return nil
}

// checkIgnore checks if the given error matches any rule in .driftignore.
func checkIgnore(repoFS fs.FS, targetErr error) bool {
	if targetErr == nil {
		return false
	}
	errMsg := targetErr.Error()
	content, err := fs.ReadFile(repoFS, ".driftignore")
	if err != nil {
		return false
	}

	today := time.Now().UTC().Format("20060102")
	for _, line := range strings.Split(string(content), "\n") {
		fullLine := strings.TrimSpace(line)
		if fullLine == "" || strings.HasPrefix(fullLine, "#") {
			continue
		}

		parts := strings.SplitN(fullLine, "#", 2)
		substring := strings.TrimSpace(parts[0])

		if substring == "" {
			fmt.Printf("WARN: [Invalid ignore] (empty substring matches everything) - line: '%s'\n", fullLine)
			continue
		}

		if !strings.Contains(errMsg, substring) {
			continue
		}

		if len(parts) > 1 {
			return parseException(substring, strings.TrimSpace(parts[1]), today)
		}
		// Metadata missing entirely
		fmt.Printf("WARN: [Invalid ignore] %s (missing metadata - expected '# reason | until_YYYYMMDD')\n", substring)
		return true
	}
	return false
}

func parseException(substring, meta, today string) bool {
	idx := strings.LastIndex(meta, "| until_")
	if idx == -1 {
		// Metadata present but no expiry suffix: treat as invalid ignore (but currently allowed with warning for transition).
		// Note context: "reason | until_YYYYMMDD"
		fmt.Printf("WARN: [Invalid ignore] %s (# %s) – missing expiry (expected '| until_YYYYMMDD')\n", substring, meta)
		return true // Still ignoring for now to avoid breaking legacy, but warning loudly.
	}

	expiry := strings.TrimSpace(meta[idx+len("| until_"):])
	if len(expiry) != 8 {
		fmt.Printf("WARN: [Invalid ignore] %s (# %s) – malformed expiry '%s' (expected YYYYMMDD)\n", substring, meta, expiry)
		return true
	}

	// Simple digit check
	for _, ch := range expiry {
		if ch < '0' || ch > '9' {
			fmt.Printf("WARN: [Invalid ignore] %s (# %s) – non-digit characters in expiry '%s'\n", substring, meta, expiry)
			return true
		}
	}

	if expiry < today {
		// Expired!
		if os.Getenv("NO_COLOR") != "" {
			fmt.Printf("WARN: [Expired] %s (expired on %s)\n", substring, expiry)
		} else {
			fmt.Printf("\x1b[1;33mWARN: [Expired] %s (expired on %s)\x1b[0m\n", substring, expiry)
		}
		// Return false so drift check FAILS.
		return false
	}

	// Valid and future
	return true
}

// findSOT locates the Source of Truth file deterministically.
// Rules:
// 1. Must match PR-\d+-*.md
// 2. If wantedPR > 0, filter only that PR number.
// 3. Otherwise, select the Highest PR Number available.
// 4. If multiple candidates exist for the selected PR number => Fail (Ambiguous).
func findSOT(repoFS fs.FS, wantedPR int) (string, error) {
	entries, err := fs.ReadDir(repoFS, "docs/pr")
	if err != nil {
		return "", fmt.Errorf("failed to read docs/pr: %w", err)
	}

	// Group path by PR number
	candidates := make(map[int][]string)
	maxPR := -1

	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		name := e.Name()
		// Parse PR number if name matches PR-\d+-*.md
		if !strings.HasPrefix(name, "PR-") || !strings.HasSuffix(name, ".md") {
			continue
		}

		// Extract digits between "PR-" and next "-"
		rest := strings.TrimPrefix(name, "PR-")
		idx := strings.Index(rest, "-")
		if idx <= 0 {
			continue
		}
		numStr := rest[:idx]

		// Verify digits (stdlib only)
		// Pure stdlib logic, no regex
		// manual simplified Atoi to avoid heavy imports if desired,
		// but standardstrconv is fine. We need to import strconv.
		// Since we didn't import strconv yet, let's use a simple loop or update imports.
		// Let's assume standard behavior and just verify digits.

		isDigits := true
		for _, r := range numStr {
			if r < '0' || r > '9' {
				isDigits = false
				break
			}
		}
		if !isDigits {
			continue
		}

		// Parse integer manually
		val := 0
		for _, r := range numStr {
			val = val*10 + int(r-'0')
		}

		if wantedPR > 0 && val != wantedPR {
			continue
		}

		candidates[val] = append(candidates[val], "docs/pr/"+name)
		if val > maxPR {
			maxPR = val
		}
	}

	if maxPR == -1 {
		if wantedPR > 0 {
			return "", fmt.Errorf("sot_missing: PR-%d not found", wantedPR)
		}
		return "", fmt.Errorf("sot_missing: no valid PR-XX-*.md files found in docs/pr/")
	}

	// Select the PR group
	selectedPR := maxPR
	if wantedPR > 0 {
		// If explicit wantedPR provided, we only populated that group
		// so maxPR implies wantedPR if found.
		selectedPR = wantedPR
	}

	files := candidates[selectedPR]
	if len(files) == 0 {
		// Should be covered by maxPR logic, but if wantedPR was set and not found:
		return "", fmt.Errorf("sot_missing: PR-%d not found", wantedPR)
	}
	if len(files) > 1 {
		return "", fmt.Errorf("sot_ambiguous: multiple files for PR-%d: %v", selectedPR, files)
	}

	return files[0], nil
}

func validateStatusEnforcement(root string) error {
	// 1. Identify Branch
	// Try CI env var first, then git
	branch := os.Getenv("GITHUB_HEAD_REF")
	if branch == "" {
		// Fallback to git
		cmd := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD")
		cmd.Dir = root
		out, err := cmd.Output()
		if err == nil {
			branch = strings.TrimSpace(string(out))
		}
	}

	// Normalize
	branch = strings.TrimSpace(branch)
	if branch == "" {
		// If we can't determine branch, we can't enforce S11 rule cleanly.
		// Fail open or closed?
		// Plan says: "if branchName does NOT match ^s11-: skip"
		// If unknown, we assume not S11 to avoid blocking legit workflows?
		// But "cannot resolve base ref" is an error.
		// Let's assume if we can't get branch, we skip (or print warning).
		return nil
	}

	if !strings.HasPrefix(branch, "s11-") {
		// Not an S11 branch
		return nil
	}

	// 2. Resolve Base Ref
	baseCandidates := []string{"origin/main", "main"}
	var baseRef string
	for _, ref := range baseCandidates {
		// git rev-parse --verify <ref>
		cmd := exec.Command("git", "rev-parse", "--verify", ref)
		cmd.Dir = root
		if err := cmd.Run(); err == nil {
			baseRef = ref
			break
		}
	}
	if baseRef == "" {
		return fmt.Errorf("cannot resolve base ref; expected origin/main or main")
	}

	// 3. Compute Diff
	// git diff --name-only --diff-filter=ACMRT <baseRef>...HEAD
	cmd := exec.Command("git", "diff", "--name-only", "--diff-filter=ACMRT", baseRef+"...HEAD")
	cmd.Dir = root
	var buf bytes.Buffer
	cmd.Stdout = &buf
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("cannot compute diff vs base ref %s: %v", baseRef, err)
	}

	output := strings.TrimSpace(buf.String())
	if output == "" {
		// No changes vs base
		return nil
	}

	files := strings.Split(output, "\n")
	// Deterministic sort
	sort.Strings(files)

	// 4. Check for STATUS.md
	found := false
	for _, f := range files {
		f = strings.TrimSpace(f)
		if f == "docs/ops/STATUS.md" {
			found = true
			break
		}
	}

	if !found {
		return &driftError{
			category: "S11 Discipline",
			reason:   "S11 requires STATUS.md update, but diff lacks docs/ops/STATUS.md",
			fixCmd:   "edit docs/ops/STATUS.md to reflect current phase progress",
			nextCmd:  "commit and re-run prverify",
		}
	}

	return nil
}
