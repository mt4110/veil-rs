package main

import (
	"archive/tar"
	"compress/gzip"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"
)

func CreateBundleUI(mode, outDir, repoDir, heavy string, autocommit bool, message string, stdout, stderr io.Writer) error {
	// S12-03: Strict Ritual Capsule
	if mode == ModeStrict {
		return strictCreateCapsule(outDir, repoDir, heavy, autocommit, message, stdout, stderr)
	}

	epoch, err := ComputeEpochSec(repoDir)
	if err != nil {
		return err
	}

	headSHA, err := getGitHeadSHA(repoDir)
	if err != nil {
		return err
	}

	fmt.Fprintf(stdout, "Creating bundle (mode=%s, epoch=%d, head=%s)\n", mode, epoch, headSHA[:12])

	// C4: Pre-checks
	isDirty, err := isGitDirty(repoDir)
	if err != nil {
		return err
	}

	if mode == ModeStrict {
		// This block is now handled by strictCreateCapsule
		// if isDirty {
		// 	return NewVError(E_CONTRACT, "git", "repository is dirty (prohibited in strict mode)")
		// }
		// Evidence check will come in C5/C6
	} else if mode == ModeWIP {
		if isDirty {
			fmt.Fprintln(stderr, "WARN: repository is dirty")
		}
	}

	// C4: contract.json generation
	contract := &Contract{
		ContractVersion: "1.1",
		Mode:            mode,
		Repo:            "veil-rs", // Canonical
		EpochSec:        epoch,
		BaseRef:         "main", // Canonical entry
		HeadSHA:         headSHA,
		WarningsCount:   0,
		Evidence: Evidence{
			Required:    false,
			Present:     false,
			BoundToHead: false,
			PathPrefix:  DirEvidence,
		},
		Tool: Tool{
			Name:    "reviewbundle",
			Version: "1.0.0",
		},
	}

	if isDirty {
		contract.WarningsCount++
	}

	// C5/C6: Actual bundle generation
	path, err := CreateBundle(contract, outDir, repoDir)
	if err != nil {
		return err
	}

	fmt.Fprintf(stdout, "Bundle created: %s\n", path)
	return nil
}

// strictCreateCapsule implements the S12-03 strict ritual.
func strictCreateCapsule(outDir, repoDir, heavy string, autocommit bool, message string, stdout, stderr io.Writer) error {
	// A: Preflight & Dirty Check
	headSHA, err := getGitHeadSHA(repoDir)
	if err != nil {
		return err
	}
	fmt.Fprintf(stdout, "OK: HEAD_START=%s\n", headSHA)

	isDirty, err := isGitDirty(repoDir)
	if err != nil {
		return err
	}

	if isDirty {
		fmt.Fprintln(stdout, "INFO: repo dirty")
		if !autocommit {
			fmt.Fprintln(stderr, "ERROR: repo dirty; commit first OR pass --autocommit --message")
			fmt.Fprintln(stdout, "SKIP: strict create (dirty)")
			return nil // Intentionally nil return to avoid exit code 1, but stop process
		}
		if message == "" {
			fmt.Fprintln(stderr, "ERROR: --autocommit requires --message")
			fmt.Fprintln(stdout, "SKIP: strict create (missing message)")
			return nil
		}
		// Commit
		// Check for unstaged?
		// "unstaged changes exist" -> ERROR
		// "staged changes missing" -> ERROR
		// We need to check porcelain status in detail.
		// For now, let's implement the basic git commit wrapper.
		// NOTE: Protocol says: "unstaged changes exist -> ERROR" (don't auto add)
		// "staged missing -> ERROR" (don't empty commit)
		// Let's rely on git commit failing if nothing staged, but we should check unstaged.
		hasUnstaged, err := hasUnstagedChanges(repoDir)
		if err != nil {
			return err
		}
		if hasUnstaged {
			fmt.Fprintln(stderr, "ERROR: unstaged changes exist; stage explicitly")
			fmt.Fprintln(stdout, "SKIP: strict create (unstaged)")
			return nil
		}

		// Try commit
		if err := gitCommit(repoDir, message); err != nil {
			// This likely means nothing to commit
			fmt.Fprintf(stderr, "ERROR: commit failed (nothing staged?): %v\n", err)
			fmt.Fprintln(stdout, "SKIP: strict create (commit failed)")
			return nil
		}
		// Update HEAD
		headSHA, err = getGitHeadSHA(repoDir)
		if err != nil {
			return err
		}
		fmt.Fprintf(stdout, "OK: committed; HEAD_NOW=%s\n", headSHA)
	} else {
		fmt.Fprintln(stdout, "OK: repo clean")
	}

	// B: Evidence Resolver
	evidenceDirs := []string{".local/prverify", "docs/evidence/prverify"}
	if repoDir != "" {
		for i, d := range evidenceDirs {
			evidenceDirs[i] = filepath.Join(repoDir, d)
		}
	}

	resolveEvidence := func() (string, error) {
		for _, dir := range evidenceDirs {
			entries, err := os.ReadDir(dir)
			if err != nil {
				continue
			}
			// Sort newest first
			sort.Slice(entries, func(i, j int) bool {
				return entries[i].Name() > entries[j].Name()
			})
			for _, e := range entries {
				if strings.HasPrefix(e.Name(), "prverify_") && strings.HasSuffix(e.Name(), ".md") {
					path := filepath.Join(dir, e.Name())
					content, err := os.ReadFile(path)
					if err != nil {
						continue
					}
					if strings.Contains(string(content), headSHA) {
						return path, nil
					}
				}
			}
		}
		return "", nil
	}

	reportPath, _ := resolveEvidence()
	if reportPath != "" {
		fmt.Fprintf(stdout, "OK: evidence_report=%s\n", reportPath)
	} else {
		fmt.Fprintln(stdout, "INFO: evidence for HEAD not found")
		if heavy == "never" {
			fmt.Fprintln(stderr, "ERROR: missing prverify for HEAD; heavy=never")
			fmt.Fprintln(stdout, "SKIP: strict create (no evidence)")
			return nil
		}

		// Heavy step
		fmt.Fprintln(stdout, "INFO: run prverify (heavy)")
		if err := runPrverify(repoDir); err != nil {
			fmt.Fprintf(stderr, "ERROR: prverify failed: %v\n", err)
			fmt.Fprintln(stdout, "SKIP: strict create (prverify failed)")
			return nil
		}

		// Check head drift
		newHead, err := getGitHeadSHA(repoDir)
		if err != nil {
			return err
		}
		if newHead != headSHA {
			fmt.Fprintln(stderr, "ERROR: HEAD changed during prverify; rerun ritual")
			fmt.Fprintln(stdout, "SKIP: strict create (HEAD drift)")
			return nil
		}

		// Re-resolve
		reportPath, _ = resolveEvidence()
		if reportPath == "" {
			fmt.Fprintln(stderr, "ERROR: prverify completed but report still missing HEAD")
			fmt.Fprintln(stdout, "SKIP: strict create (no head evidence)")
			return nil
		}
		fmt.Fprintf(stdout, "OK: evidence_report=%s\n", reportPath)
	}

	// C: Delegate to CreateBundle
	// We construct contract manually here because CreateBundle logic expects to FIND evidence itself?
	// Actually `collectEvidence` in CreateBundle searches again.
	// But `strictCreateCapsule` ensures it EXISTS.
	// We should just call `CreateBundle` with ModeStrict, and it will call `collectEvidence`.
	// Since we verified evidence exists and matches HEAD, `collectEvidence` should find it.
	// HOWEVER, `collectEvidence` might find *other* evidence if we are not careful about `repoDir`.
	// But `collectEvidence` searches the same dirs.
	// So we can just proceed.

	epoch, err := ComputeEpochSec(repoDir)
	if err != nil {
		return err
	}

	contract := &Contract{
		ContractVersion: "1.1",
		Mode:            ModeStrict,
		Repo:            "veil-rs",
		EpochSec:        epoch,
		BaseRef:         "main",
		HeadSHA:         headSHA,
		// Evidence fields will be filled by CreateBundle -> collectEvidence
		Evidence: Evidence{
			Required:   true,
			Present:    false,
			PathPrefix: DirEvidence,
		},
		Tool: Tool{
			Name:    "reviewbundle",
			Version: "1.0.0",
		},
	}

	path, err := CreateBundle(contract, outDir, repoDir)
	if err != nil {
		return err
	}
	fmt.Fprintf(stdout, "Bundle created: %s\n", path)
	fmt.Fprintln(stdout, "OK: strict bundle created")

	return nil
}

func hasUnstagedChanges(repoDir string) (bool, error) {
	// git diff --name-only (if output not empty, unstaged changes)
	cmd := exec.Command("git", "diff", "--name-only")
	if repoDir != "" {
		cmd.Dir = repoDir
	}
	out, err := cmd.Output()
	if err != nil {
		return false, err
	}
	return len(strings.TrimSpace(string(out))) > 0, nil
}

func gitCommit(repoDir, message string) error {
	// If git identity is missing (common in CI runners), make this commit hermetic.
	nameCmd := exec.Command("git", "config", "--get", "user.name")
	nameCmd.Dir = repoDir
	nameOut, _ := nameCmd.Output()

	emailCmd := exec.Command("git", "config", "--get", "user.email")
	emailCmd.Dir = repoDir
	emailOut, _ := emailCmd.Output()

	name := strings.TrimSpace(string(nameOut))
	email := strings.TrimSpace(string(emailOut))

	args := []string{"commit", "-m", message}
	if name == "" || email == "" {
		args = []string{
			"-c", "user.name=veil-ci",
			"-c", "user.email=veil-ci@example.invalid",
			"-c", "commit.gpgSign=false",
			"commit", "-m", message,
		}
	}

	cmd := exec.Command("git", args...)
	cmd.Dir = repoDir
	out, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git commit failed: %w: %s", err, strings.TrimSpace(string(out)))
	}
	return nil
}

func runPrverify(repoDir string) error {
	// nix run .#prverify
	cmd := exec.Command("nix", "run", ".#prverify")
	if repoDir != "" {
		cmd.Dir = repoDir
	}
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func CreateBundle(c *Contract, outDir, repoDir string) (string, error) {
	if err := os.MkdirAll(outDir, 0755); err != nil {
		return "", WrapVError(E_PATH, outDir, err)
	}

	ts := time.Unix(c.EpochSec, 0).UTC().Format("20060102_150405")
	name := fmt.Sprintf("veil-rs_review_%s_%s_%s.tar.gz", c.Mode, ts, c.HeadSHA[:12])
	outPath := filepath.Join(outDir, name)

	tmpPath := outPath + ".tmp"
	f, err := os.Create(tmpPath)
	if err != nil {
		return "", WrapVError(E_PATH, tmpPath, err)
	}
	defer os.Remove(tmpPath)
	defer f.Close()

	gw := gzip.NewWriter(f)
	gw.Header.ModTime = time.Unix(c.EpochSec, 0)
	gw.Header.OS = 255
	defer gw.Close()

	tw := tar.NewWriter(gw)
	defer tw.Close()

	// 1. Gather files
	files := make(map[string][]byte)

	// INDEX.md
	files[PathIndex] = []byte(fmt.Sprintf("# Review Bundle\n\nMode: %s\nEpoch: %d\nHead: %s\n", c.Mode, c.EpochSec, c.HeadSHA))

	// patch/series.patch
	patch, err := getGitPatch(c.BaseRef, c.HeadSHA, repoDir)
	if err != nil {
		return "", err
	}
	files[PathSeriesPatch] = patch

	// Evidence (Phase 7.5/8/9)
	bound, evFiles, err := collectEvidence(c.HeadSHA, repoDir)
	if err != nil {
		return "", err
	}
	if len(evFiles) > 0 {
		c.Evidence.Present = true
		c.Evidence.BoundToHead = bound
		for name, content := range evFiles {
			files[name] = content
		}
	}

	// meta/contract.json (update with evidence findings)
	cj, _ := json.MarshalIndent(c, "", "  ")
	files[PathContract] = cj

	// C4: Contract: if warnings_count > 0, warnings.txt MUST exist.
	if c.WarningsCount > 0 {
		files[PathWarnings] = []byte(fmt.Sprintf("warnings_count=%d\n", c.WarningsCount))
	}

	// 2. Generate SHA256SUMS (C6)
	var manifestKeys []string
	for k := range files {
		manifestKeys = append(manifestKeys, k)
	}
	sort.Strings(manifestKeys)

	var sumsBuilder strings.Builder
	for _, k := range manifestKeys {
		h := sha256.Sum256(files[k])
		fmt.Fprintf(&sumsBuilder, "%x  %s\n", h, k)
	}
	sums := []byte(sumsBuilder.String())
	files[PathSHA256SUMS] = sums

	// seal
	seal := sha256.Sum256(sums)
	files[PathSHA256SUMSSeal] = []byte(fmt.Sprintf("%x  %s\n", seal, PathSHA256SUMS))

	// 3. Sort and write (C5)
	var keys []string
	for k := range files {
		keys = append(keys, k)
	}
	sort.Strings(keys)

	for _, k := range keys {
		content := files[k]
		hdr := &tar.Header{
			Name:     k,
			Size:     int64(len(content)),
			Mode:     0644,
			ModTime:  time.Unix(c.EpochSec, 0),
			Typeflag: tar.TypeReg,
			Uid:      0,
			Gid:      0,
			Uname:    "",
			Gname:    "",
			Format:   tar.FormatPAX,
			// Phase 4.2/7.3: Explicitly zero extra times to avoid PAX mtime/atime/ctime
			AccessTime: time.Time{},
			ChangeTime: time.Time{},
			PAXRecords: nil,
		}
		if err := tw.WriteHeader(hdr); err != nil {
			return "", WrapVError(E_GZIP, k, err)
		}
		if _, err := tw.Write(content); err != nil {
			return "", WrapVError(E_GZIP, k, err)
		}
	}

	tw.Close()
	gw.Close()
	f.Close()

	// 4. Self-Audit (C6) - Verify atomic temp file
	_, err = VerifyBundlePath(tmpPath)
	if err != nil {
		// Verification failed on tmp file.
		return tmpPath, fmt.Errorf("self-audit failed for %s: %w", tmpPath, err)
	}

	if err := os.Rename(tmpPath, outPath); err != nil {
		return "", WrapVError(E_PATH, outPath, err)
	}

	return outPath, nil
}

func getGitPatch(base, head, repoDir string) ([]byte, error) {
	cmd := exec.Command("git", "format-patch", "--stdout", base+".."+head)
	if repoDir != "" {
		cmd.Dir = repoDir
	}
	out, err := cmd.Output()
	if err != nil {
		return nil, WrapVError(E_CONTRACT, "git format-patch", err)
	}
	return out, nil
}

func ComputeEpochSec(repoDir string) (int64, error) {
	if s := os.Getenv("SOURCE_DATE_EPOCH"); s != "" {
		sec, err := strconv.ParseInt(s, 10, 64)
		if err == nil {
			return sec, nil
		}
	}

	// Fallback to git show -s --format=%ct HEAD
	cmd := exec.Command("git", "show", "-s", "--format=%ct", "HEAD")
	if repoDir != "" {
		cmd.Dir = repoDir
	}
	out, err := cmd.Output()
	if err != nil {
		return 0, WrapVError(E_CONTRACT, "git", err)
	}
	sec, err := strconv.ParseInt(strings.TrimSpace(string(out)), 10, 64)
	if err != nil {
		return 0, WrapVError(E_CONTRACT, "git parse", err)
	}
	return sec, nil
}

func getGitHeadSHA(repoDir string) (string, error) {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	if repoDir != "" {
		cmd.Dir = repoDir
	}
	out, err := cmd.Output()
	if err != nil {
		return "", WrapVError(E_CONTRACT, "git rev-parse", err)
	}
	return strings.TrimSpace(string(out)), nil
}

func collectEvidence(headSHA, repoDir string) (bool, map[string][]byte, error) {
	files := make(map[string][]byte)
	bound := false

	// Helper to process directory
	processDir := func(dir string, isLocal bool) error {
		entries, err := os.ReadDir(dir)
		if err != nil {
			if os.IsNotExist(err) {
				return nil
			}
			return WrapVError(E_PATH, dir, err)
		}

		// Sort entries for determinism (repo evidence) or priority (local evidence)
		// Local: newest first to find best specific match
		// Repo: alphabetical for consistency (though map iteration order is random, bundle creation sorts keys)
		if isLocal {
			sort.Slice(entries, func(i, j int) bool {
				// We want newest first, but os.ReadDir returns DirEntry without ModTime directly in struct (need Info)
				// Actually filenames have timestamp: prverify_YYYYMMDDTHHMMSSZ_...
				// So just sorting by name DESC works for timestamp
				return entries[i].Name() > entries[j].Name()
			})
		}

		for _, e := range entries {
			if e.IsDir() || !strings.HasPrefix(e.Name(), "prverify_") || !strings.HasSuffix(e.Name(), ".md") {
				continue
			}

			// For local evidence, we only pick ONE: the newest one that binds to HEAD.
			// If we already bound, skip others?
			// The requirement: "local evidence... newest 1" (implied: that matches)
			// "local evidence: .local/prverify/prverify_*.md のうち HEAD SHA を含む最新1件"
			if isLocal && bound {
				continue
			}

			path := filepath.Join(dir, e.Name())
			content, err := os.ReadFile(path)
			if err != nil {
				return WrapVError(E_PATH, path, err)
			}

			// Phase 7.5: 4MB limit check for binding
			isTooBig := len(content) > 4*1024*1024
			containsHead := !isTooBig && strings.Contains(string(content), headSHA)

			if isLocal {
				// Local rule: only include if it binds to HEAD
				if !containsHead {
					continue
				}
				// Found newest binding local evidence!
				// Include it.
			}

			// Bundle path: review/evidence/prverify/prverify_...
			bundlePath := filepath.Join(DirEvidence, "prverify", e.Name())
			files[bundlePath] = content

			if containsHead {
				bound = true
			}
		}
		return nil
	}

	// 1. Repo evidence (docs/evidence/prverify)
	repoEvDir := "docs/evidence/prverify"
	if repoDir != "" {
		repoEvDir = filepath.Join(repoDir, repoEvDir)
	}
	if err := processDir(repoEvDir, false); err != nil {
		return false, nil, err
	}

	// 2. Local evidence (.local/prverify) - ONLY if not already bound?
	// The requirement implies finding a local one if needed.
	// Actually, strict mode MIGHT fail if repo evidence doesn't bind.
	// So we should always search local if strict? Or just always search?
	// The plan says: "strict create は evidence として次を扱う... 2) local evidence"
	// It doesn't strictly say "only if repo fails", but implies extending scope.
	// Safe to always search local for a HEAD match.
	localEvDir := ".local/prverify"
	if repoDir != "" {
		localEvDir = filepath.Join(repoDir, localEvDir)
	}
	if err := processDir(localEvDir, true); err != nil {
		return false, nil, err
	}

	return bound, files, nil
}

func isGitDirty(repoDir string) (bool, error) {
	cmd := exec.Command("git", "status", "--porcelain")
	if repoDir != "" {
		cmd.Dir = repoDir
	}
	out, err := cmd.Output()
	if err != nil {
		return false, WrapVError(E_CONTRACT, "git status", err)
	}
	return len(strings.TrimSpace(string(out))) > 0, nil
}
