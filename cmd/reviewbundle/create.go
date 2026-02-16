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

func CreateBundleUI(mode, outDir string, stdout, stderr io.Writer) error {
	epoch, err := ComputeEpochSec()
	if err != nil {
		return err
	}

	headSHA, err := getGitHeadSHA()
	if err != nil {
		return err
	}

	fmt.Fprintf(stdout, "Creating bundle (mode=%s, epoch=%d, head=%s)\n", mode, epoch, headSHA[:12])

	// C4: Pre-checks
	isDirty, err := isGitDirty()
	if err != nil {
		return err
	}

	if mode == ModeStrict {
		if isDirty {
			return NewVError(E_CONTRACT, "git", "repository is dirty (prohibited in strict mode)")
		}
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
		BaseRef:         "main", // TODO: discover
		HeadSHA:         headSHA,
		WarningsCount:   0,
		Evidence: Evidence{
			Required:    mode == ModeStrict,
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
	path, err := CreateBundle(contract, outDir)
	if err != nil {
		return err
	}

	fmt.Fprintf(stdout, "Bundle created: %s\n", path)
	return nil
}

func CreateBundle(c *Contract, outDir string) (string, error) {
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
	patch, err := getGitPatch(c.BaseRef, c.HeadSHA)
	if err != nil {
		return "", err
	}
	files[PathSeriesPatch] = patch

	// Evidence (Phase 7.5/8/9)
	bound, evFiles, err := collectEvidence(c.HeadSHA)
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

	if err := os.Rename(tmpPath, outPath); err != nil {
		return "", WrapVError(E_PATH, outPath, err)
	}

	// 4. Self-Audit (C6)
	_, err = VerifyBundlePath(outPath)
	if err != nil {
		// If verification fails, we keep the broken bundle for inspection but return error
		return outPath, fmt.Errorf("self-audit failed for %s: %w", outPath, err)
	}

	return outPath, nil
}

func getGitPatch(base, head string) ([]byte, error) {
	out, err := exec.Command("git", "format-patch", "--stdout", base+".."+head).Output()
	if err != nil {
		return nil, WrapVError(E_CONTRACT, "git format-patch", err)
	}
	return out, nil
}

func ComputeEpochSec() (int64, error) {
	if s := os.Getenv("SOURCE_DATE_EPOCH"); s != "" {
		sec, err := strconv.ParseInt(s, 10, 64)
		if err == nil {
			return sec, nil
		}
	}

	// Fallback to git show -s --format=%ct HEAD
	out, err := exec.Command("git", "show", "-s", "--format=%ct", "HEAD").Output()
	if err != nil {
		return 0, WrapVError(E_CONTRACT, "git", err)
	}
	sec, err := strconv.ParseInt(strings.TrimSpace(string(out)), 10, 64)
	if err != nil {
		return 0, WrapVError(E_CONTRACT, "git parse", err)
	}
	return sec, nil
}

func getGitHeadSHA() (string, error) {
	out, err := exec.Command("git", "rev-parse", "HEAD").Output()
	if err != nil {
		return "", WrapVError(E_CONTRACT, "git rev-parse", err)
	}
	return strings.TrimSpace(string(out)), nil
}

func collectEvidence(headSHA string) (bool, map[string][]byte, error) {
	evDir := "docs/evidence/prverify"
	entries, err := os.ReadDir(evDir)
	if err != nil {
		if os.IsNotExist(err) {
			return false, nil, nil
		}
		return false, nil, WrapVError(E_PATH, evDir, err)
	}

	files := make(map[string][]byte)
	bound := false

	for _, e := range entries {
		if e.IsDir() || !strings.HasPrefix(e.Name(), "prverify_") || !strings.HasSuffix(e.Name(), ".md") {
			continue
		}
		path := filepath.Join(evDir, e.Name())
		content, err := os.ReadFile(path)
		if err != nil {
			return false, nil, WrapVError(E_PATH, path, err)
		}

		// Bundle path: review/evidence/prverify/prverify_...
		bundlePath := filepath.Join(DirEvidence, "prverify", e.Name())
		files[bundlePath] = content

		// Phase 7.5: 4MB limit check for binding
		if len(content) > 4*1024*1024 {
			continue // Too big to trust for binding
		}
		if strings.Contains(string(content), headSHA) {
			bound = true
		}
	}

	return bound, files, nil
}

func isGitDirty() (bool, error) {
	out, err := exec.Command("git", "status", "--porcelain").Output()
	if err != nil {
		return false, WrapVError(E_CONTRACT, "git status", err)
	}
	return len(strings.TrimSpace(string(out))) > 0, nil
}
