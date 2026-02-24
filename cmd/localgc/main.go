package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// localgc — .local directory GC tool (S12-06C)
//
// Modes:
//
//	--mode dry-run (default): list candidates, delete nothing
//	--mode plan:              list with size and reasons
//	--mode apply + --apply:   double-lock required to delete
//
// Invariants:
//   - Process always exits 0 (stopless)
//   - OK: phase=end stop=<0|1> always last line
//   - No os.Exit, no panic, no log.Fatal, no log.Panic

const (
	modeDefault = "dry-run"
	modePlan    = "plan"
	modeApply   = "apply"

	// Retention limits
	retainPrverify     = 5
	retainReviewBundle = 3
	retainObs          = 10
	retainEvidence     = 5
	retainHandoff      = 3
	retainLockBackup   = 5

	// Warn if any tracked dir exceeds this (MB)
	warnSizeMB = 500

	// Age threshold for misc scratch candidates (days)
	scratchAgeDays = 7
)

type entry struct {
	name  string
	path  string
	mtime time.Time
	size  int64
}

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

func run(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("localgc", flag.ContinueOnError)
	fs.SetOutput(stderr)
	modeFlag := fs.String("mode", modeDefault, "GC mode: dry-run|plan|apply")
	applyFlag := fs.Bool("apply", false, "Confirm apply (required together with --mode apply)")
	rootFlag := fs.String("root", ".", "Repo root to resolve .local from")

	if err := fs.Parse(args); err != nil {
		fmt.Fprintln(stdout, "ERROR: flag_parse detail="+err.Error())
		fmt.Fprintln(stdout, "OK: phase=end stop=1")
		return 0
	}

	mode := *modeFlag
	doApply := *applyFlag
	root := *rootFlag

	// Double-lock enforcement
	if doApply && mode != modeApply {
		fmt.Fprintf(stdout, "SKIP: --apply requires --mode apply (got --mode %s)\n", mode)
		fmt.Fprintln(stdout, "OK: phase=end stop=0")
		return 0
	}
	if mode == modeApply && !doApply {
		fmt.Fprintln(stdout, "SKIP: --mode apply requires --apply flag (double-lock)")
		fmt.Fprintln(stdout, "OK: phase=end stop=0")
		return 0
	}

	hasError := false
	localDir := filepath.Join(root, ".local")

	// Phase 0: fast inventory
	cats := inventoryLocal(localDir, stdout, &hasError)

	// Phase 1 (plan mode): show sizes
	if mode == modePlan {
		showPlan(cats, localDir, stdout, &hasError)
	}

	// Phase 2 (apply mode, double-lock confirmed): remove candidates
	if mode == modeApply && doApply {
		applyGC(cats, stdout, stderr, &hasError)
	}

	if mode == modeDefault {
		fmt.Fprintln(stdout, "INFO: dry-run complete (nothing deleted); use --mode plan for sizes, --mode apply --apply to delete")
	}

	stopVal := 0
	if hasError {
		stopVal = 1
	}
	fmt.Fprintf(stdout, "OK: phase=end stop=%d\n", stopVal)
	return 0
}

// category groups entries of one .local sub-category.
type category struct {
	label     string  // human label
	dir       string  // abs path
	entries   []entry // sorted newest-first
	keepCount int     // how many to keep
	gcable    []entry // entries beyond keepCount
}

func inventoryLocal(localDir string, stdout io.Writer, hasErr *bool) []category {
	cats := []category{
		{label: "prverify", dir: filepath.Join(localDir, "prverify"), keepCount: retainPrverify},
		{label: "review-bundles", dir: filepath.Join(localDir, "review-bundles"), keepCount: retainReviewBundle},
		{label: "obs", dir: filepath.Join(localDir, "obs"), keepCount: retainObs},
		{label: "evidence", dir: filepath.Join(localDir, "evidence"), keepCount: retainEvidence},
		{label: "handoff", dir: filepath.Join(localDir, "handoff"), keepCount: retainHandoff},
		{label: "lock_backup", dir: filepath.Join(localDir, "lock_backup"), keepCount: retainLockBackup},
	}

	for i := range cats {
		c := &cats[i]
		entries, err := readEntries(c.dir)
		if err != nil {
			// Missing dir is fine — just skip
			fmt.Fprintf(stdout, "OK: dir=%s count=0 newest=none\n", c.label)
			continue
		}
		// Sort newest first
		sort.Slice(entries, func(a, b int) bool {
			return entries[a].mtime.After(entries[b].mtime)
		})
		c.entries = entries

		newest := "none"
		if len(entries) > 0 {
			newest = entries[0].mtime.UTC().Format("20060102T150405Z")
		}
		fmt.Fprintf(stdout, "OK: dir=%s count=%d newest=%s\n", c.label, len(entries), newest)

		// Determine GC candidates
		if len(entries) > c.keepCount {
			c.gcable = entries[c.keepCount:]
		}
	}
	return cats
}

func readEntries(dir string) ([]entry, error) {
	des, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	var entries []entry
	for _, de := range des {
		info, err := de.Info()
		if err != nil {
			continue
		}
		entries = append(entries, entry{
			name:  de.Name(),
			path:  filepath.Join(dir, de.Name()),
			mtime: info.ModTime(),
			size:  info.Size(),
		})
	}
	return entries, nil
}

func showPlan(cats []category, localDir string, stdout io.Writer, hasErr *bool) {
	fmt.Fprintln(stdout, "## Plan")
	for _, c := range cats {
		if len(c.gcable) == 0 {
			continue
		}
		for _, e := range c.gcable {
			age := time.Since(e.mtime).Hours() / 24
			fmt.Fprintf(stdout, "CANDIDATE: dir=%s name=%s age=%.0fd size=%dB reason=exceeds_retain_%d\n",
				c.label, e.name, age, e.size, c.keepCount)
		}
	}

	// Misc scratch at .local/ root (loose .txt/.tsv/.stderr/.stdout older than scratchAgeDays)
	miscCandidates := scratchCandidates(localDir)
	for _, e := range miscCandidates {
		age := time.Since(e.mtime).Hours() / 24
		fmt.Fprintf(stdout, "CANDIDATE: dir=.local name=%s age=%.0fd size=%dB reason=scratch_older_%dd\n",
			e.name, age, e.size, scratchAgeDays)
	}
}

func scratchCandidates(localDir string) []entry {
	des, err := os.ReadDir(localDir)
	if err != nil {
		return nil
	}
	threshold := time.Now().Add(-time.Duration(scratchAgeDays) * 24 * time.Hour)
	var candidates []entry
	scratchExts := map[string]bool{".txt": true, ".tsv": true, ".stderr": true, ".stdout": true}
	for _, de := range des {
		if de.IsDir() {
			continue
		}
		ext := strings.ToLower(filepath.Ext(de.Name()))
		if !scratchExts[ext] {
			continue
		}
		info, err := de.Info()
		if err != nil {
			continue
		}
		if info.ModTime().Before(threshold) {
			candidates = append(candidates, entry{
				name:  de.Name(),
				path:  filepath.Join(localDir, de.Name()),
				mtime: info.ModTime(),
				size:  info.Size(),
			})
		}
	}
	return candidates
}

func applyGC(cats []category, stdout, stderr io.Writer, hasErr *bool) {
	fmt.Fprintln(stdout, "## Apply")
	deleted := 0
	for _, c := range cats {
		for _, e := range c.gcable {
			if err := os.RemoveAll(e.path); err != nil {
				fmt.Fprintf(stdout, "ERROR: remove_failed path=%s detail=%s\n", e.path, err.Error())
				fmt.Fprintln(stderr, err.Error())
				*hasErr = true
				continue
			}
			fmt.Fprintf(stdout, "OK: removed dir=%s name=%s\n", c.label, e.name)
			deleted++
		}
	}
	fmt.Fprintf(stdout, "OK: apply_complete deleted=%d\n", deleted)
}
