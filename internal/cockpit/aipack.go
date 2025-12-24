package cockpit

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"time"
)

// GenerateAIPack generates the AI_PACK artifact.
// It returns the final path written to, whether a temp file was used, and any error.
func GenerateAIPack(baseRef, outPath string) (finalPath string, usedTemp bool, err error) {
	// 1. Resolve Root
	rootGit := GitX{} // Initial one just to find root
	rootStr, err := rootGit.Run("rev-parse", "--show-toplevel")
	if err != nil {
		return "", false, fmt.Errorf("failed to find repo root: %w", err)
	}
	repoRoot := strings.TrimSpace(rootStr)
	gx := GitX{Dir: repoRoot}

	// 2. Resolve Base Ref
	if baseRef == "" {
		baseRef = "origin/main"
	}
	// Fallback logic for baseRef
	if _, err := gx.Run("rev-parse", "--verify", baseRef); err != nil {
		fmt.Fprintf(os.Stderr, "Note: BASE_REF '%s' not found.\n", baseRef)
		if _, err := gx.Run("rev-parse", "--verify", "HEAD~1"); err == nil {
			fmt.Fprintln(os.Stderr, "Falling back to HEAD~1")
			baseRef = "HEAD~1"
		} else {
			fmt.Fprintln(os.Stderr, "Falling back to HEAD (empty diff expected)")
			baseRef = "HEAD"
		}
	}

	// 3. Resolve Output Path
	if outPath == "" {
		f, err := os.CreateTemp("", "ai_pack.*.txt")
		if err != nil {
			return "", false, fmt.Errorf("failed to create temp file: %w", err)
		}
		outPath = f.Name()
		f.Close()
		usedTemp = true
	}

	// Buffer for output content
	var buf bytes.Buffer
	write := func(format string, a ...any) {
		fmt.Fprintf(&buf, format+"\n", a...)
	}

	// === AI_PACK ===
	nowUTC := time.Now().UTC().Format("2006-01-02T15:04:05Z")
	headRef, _ := gx.Run("rev-parse", "--short", "HEAD")
	headRef = strings.TrimSpace(headRef)

	write("=== AI_PACK ===")
	write("generated_at_utc: %s", nowUTC)
	write("base_ref: %s", baseRef)
	write("head: %s", headRef)
	write("")

	// === STATUS ===
	write("=== STATUS ===")
	statusOut, _ := gx.Run("status", "-sb")
	if statusOut != "" {
		// git status -sb usually has a trailing newline from GitX.Run if we aren't careful,
		// but GitX.Run trims space? Let's check status.go.
		// Wait, GitX.Run usually trims output. git status -sb output multiple lines.
		// We want to preserve internal newlines but maybe trim trailing.
		write("%s", strings.TrimSpace(statusOut))
	}
	write("")

	// === SUMMARY ===
	write("=== SUMMARY ===")
	branch, _ := gx.Run("branch", "--show-current")
	write("branch: %s", strings.TrimSpace(branch))
	write("last_commits:")
	logs, _ := gx.Run("log", "--oneline", "-10")
	if logs != "" {
		write("%s", strings.TrimSpace(logs))
	}
	write("")

	// === CHANGED FILES (BASE..HEAD) ===
	write("=== CHANGED FILES (BASE..HEAD) ===")
	diffNameStatus, _ := gx.Run("diff", "--name-status", baseRef+"...HEAD")
	if diffNameStatus != "" {
		write("%s", strings.TrimSpace(diffNameStatus))
	}
	write("")

	// === DIFF (unified=6) ===
	write("=== DIFF (unified=6) ===")
	diffU6, _ := gx.Run("diff", "--unified=6", baseRef+"...HEAD")
	if diffU6 != "" {
		write("%s", strings.TrimSpace(diffU6))
	}
	write("")

	// === CONTEXT_MAP ===
	write("=== CONTEXT_MAP ===")
	write("# Purpose: give line-numbered context for the important files")
	write("# Format: FILE + nl excerpts (top + around changes)")
	write("")

	// Get changed files list
	changedFilesOut, _ := gx.Run("diff", "--name-only", baseRef+"...HEAD")
	changedFiles := strings.Split(strings.TrimSpace(changedFilesOut), "\n")

	// Filter and deduplicate relevant files
	relevantFiles := filterRelevantFiles(changedFiles)

	for _, relPath := range relevantFiles {
		absPath := filepath.Join(repoRoot, relPath)

		// Check if file exists and is not binary
		if !isFileAndText(absPath) {
			continue
		}

		write("---- FILE: %s ----", relPath)
		write("[head: top]")
		// Show top 60 lines
		topLines, err := readFileLines(absPath, 1, 60)
		if err == nil {
			write("%s", topLines)
		}
		write("")

		// Show lines around diff hunks
		// Extract approximate line numbers from diff: +<start>,<len>
		// git diff -U0 base...HEAD -- file
		hunkDiff, _ := gx.Run("diff", "-U0", baseRef+"...HEAD", "--", relPath)
		hunks := parseHunkStarts(hunkDiff)

		if len(hunks) > 0 {
			write("[around changes]")
			for _, start := range hunks {
				from := start - 20
				if from < 1 {
					from = 1
				}
				to := start + 60
				chunk, err := readFileLines(absPath, from, to)
				if err == nil {
					write("%s", chunk)
					write("")
				}
			}
		}
		write("")
	}
	write("=== END ===")

	// Write buffer to file
	if err := os.WriteFile(outPath, buf.Bytes(), 0644); err != nil {
		return "", usedTemp, fmt.Errorf("failed to write output file: %w", err)
	}

	return outPath, usedTemp, nil
}

// filterRelevantFiles applies the relevance logic:
// docs/ai/*, scripts/*, .github/workflows/*, Cargo.toml, Cargo.lock, README.md, crates/*
// OR distinct files (de-duped).
// The original shell script logic adds everything hitting the glob, then defaults to "everything else" if small & texty (but shell script effectively added everything that wasn't skipped by 'continue').
// In shell script:
// case "$f" in ... relevant+=("$f") ;; *) relevant+=("$f") ;; esac
// So actually it includes ALL changed files (that are not empty strings).
// The relevance check in shell script was effectively a no-op filter that included everything.
// We will replicate that behavior: include all changed files.
func filterRelevantFiles(files []string) []string {
	var unique []string
	seen := make(map[string]bool)
	for _, f := range files {
		f = strings.TrimSpace(f)
		if f == "" {
			continue
		}
		if seen[f] {
			continue
		}
		seen[f] = true
		unique = append(unique, f)
	}
	return unique
}

// isFileAndText checks if file exists, is regular, and is not binary (first 8KB check).
func isFileAndText(path string) bool {
	info, err := os.Stat(path)
	if err != nil || info.IsDir() {
		return false
	}

	f, err := os.Open(path)
	if err != nil {
		return false
	}
	defer f.Close()

	// Check first 8KB for NUL byte
	buf := make([]byte, 8192)
	n, err := f.Read(buf)
	if err != nil && err != io.EOF {
		return false
	}
	if n == 0 {
		return true // Empty file is text
	}
	if bytes.IndexByte(buf[:n], 0) != -1 {
		return false // Binary
	}
	return true
}

// readFileLines reads lines from start to end (inclusive, 1-based) and formats them like `nl -ba`.
func readFileLines(path string, start, end int) (string, error) {
	if start > end {
		return "", nil
	}
	f, err := os.Open(path)
	if err != nil {
		return "", err
	}
	defer f.Close()

	var out strings.Builder
	scanner := bufio.NewScanner(f)
	lineNum := 0
	for scanner.Scan() {
		lineNum++
		if lineNum > end {
			break
		}
		if lineNum >= start {
			// Format: "     1\tcontent" (6 width, tab)
			fmt.Fprintf(&out, "%6d\t%s\n", lineNum, scanner.Text())
		}
	}
	return strings.TrimSuffix(out.String(), "\n"), scanner.Err()
}

// parseHunkStarts extracts hunk start lines from unified diff.
// Regex: ^@@ .*[-+]\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@
// We want the start line of the "new" file (the one with +).
func parseHunkStarts(diffOut string) []int {
	// Limits to top 8 hunks as per shell script
	const maxHunks = 8
	var starts []int
	re := regexp.MustCompile(`^@@ .* \+(\d+)(?:,\d+)? @@`)

	scanner := bufio.NewScanner(strings.NewReader(diffOut))
	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "@@") {
			matches := re.FindStringSubmatch(line)
			if len(matches) >= 2 {
				if n, err := strconv.Atoi(matches[1]); err == nil {
					starts = append(starts, n)
					if len(starts) >= maxHunks {
						break
					}
				}
			}
		}
	}
	return starts
}
