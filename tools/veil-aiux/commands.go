package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// ---- gen ----

type genOpts struct {
	version string
	clean   bool
	baseRef string
}

func cmdGen(args []string) int {
	fs := flag.NewFlagSet("gen", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)

	var o genOpts
	fs.StringVar(&o.version, "version", "", "target version (required), e.g. v0.14.0")
	fs.BoolVar(&o.clean, "clean", false, "remove dist/publish/<VER> before generation")
	fs.StringVar(&o.baseRef, "base-ref", "origin/main", "git base ref for diff stats (best-effort)")

	fs.Usage = func() {
		fmt.Fprintln(os.Stderr, "Usage: veil-aiux gen --version vX.Y.Z [--clean] [--base-ref origin/main]")
		fs.PrintDefaults()
	}

	if err := fs.Parse(args); err != nil {
		return die(E_USAGE, shortInvalidArgs, err.Error(), "Run: veil-aiux gen --help")
	}
	if extra := fs.Args(); len(extra) > 0 {
		return die(E_USAGE, shortUnexpectedArgs, fmt.Sprintf(fmtUnexpectedArgs, extra), fixRemoveExtraArgs)
	}
	if err := validateVersion(o.version); err != nil {
		return die(E_USAGE, shortInvalidVersion, err.Error(), msgExampleVersion)
	}
	if !haveGit() {
		return die(E_GEN, "git not found", "git is required to generate AI_PACK", "Run via Nix: nix run .#gen -- --version vX.Y.Z")
	}

	out := distOutDir(o.version)
	tmp := distTmpDir(o.version)

	// ensure dist/publish exists
	if err := ensureDir(filepath.Join("dist", "publish")); err != nil {
		return die(E_IO, "Failed to create dist/publish", err.Error(), "Check permissions or working directory")
	}

	// do not overwrite unless --clean is explicit
	if dirExists(out) && !o.clean {
		return die(E_USAGE, "Output already exists",
			fmt.Sprintf("%s already exists", out),
			"Re-run with --clean to overwrite (explicit) or remove the directory manually")
	}

	// clean (explicit only)
	if o.clean && dirExists(out) {
		if err := safeRemoveOutDir(out, o.version); err != nil {
			return die(E_USAGE, "Unsafe clean refused", err.Error(), "Use only the contract path dist/publish/<VER>")
		}
	}

	// remove tmp if left over
	_ = os.RemoveAll(tmp)
	if err := ensureDir(tmp); err != nil {
		return die(E_IO, "Failed to create tmp dir", err.Error(), "Check permissions")
	}

	// If anything fails, keep tmp for diagnosis (don’t auto-repair).
	// You can clean it manually.
	// generate artifacts into tmp
	if code := genIntoTmp(tmp, o.version, o.baseRef); code != E_OK {
		return code
	}

	// strict dist validation (exactly 4)
	if err := validateDistExactly4(tmp, o.version); err != nil {
		return die(E_CHECK, shortDistViolated, err.Error(), "Fix templates / generation to produce exactly the 4 contract artifacts")
	}

	// finalize: tmp -> out (atomic)
	if err := os.Rename(tmp, out); err != nil {
		return die(E_IO, "Finalize failed", err.Error(), "Ensure dist/publish is writable and output dir does not exist (or use --clean)")
	}

	// summary
	printGenSummary(o.version, o.baseRef, out)
	return E_OK
}

func genIntoTmp(tmpDir, ver, baseRef string) int {
	// 1) templates
	pPath, err := resolveTemplate(tplPublish)
	if err != nil {
		return die(E_CHECK, shortTemplateMissing, err.Error(), "Create docs/ai/PUBLISH_TEMPLATE.md (or adjust templateCandidates)")
	}
	rPath, err := resolveTemplate(tplReleaseBody)
	if err != nil {
		return die(E_CHECK, shortTemplateMissing, err.Error(), "Create docs/ai/RELEASE_BODY_TEMPLATE.md (or adjust templateCandidates)")
	}
	xPath, err := resolveTemplate(tplX)
	if err != nil {
		return die(E_CHECK, shortTemplateMissing, err.Error(), "Create docs/ai/X_TEMPLATE.md (or adjust templateCandidates)")
	}

	pRaw, err := readFile(pPath)
	if err != nil {
		return die(E_IO, shortReadTemplateFailed, err.Error(), fixCheckFilePermissions)
	}
	rRaw, err := readFile(rPath)
	if err != nil {
		return die(E_IO, shortReadTemplateFailed, err.Error(), fixCheckFilePermissions)
	}
	xRaw, err := readFile(xPath)
	if err != nil {
		return die(E_IO, shortReadTemplateFailed, err.Error(), fixCheckFilePermissions)
	}

	// guardrails on templates
	if err := checkMarkdownTemplate(pPath, pRaw); err != nil {
		return die(E_CHECK, shortBadTemplate, err.Error(), fixBadTemplate)
	}
	if err := checkMarkdownTemplate(rPath, rRaw); err != nil {
		return die(E_CHECK, shortBadTemplate, err.Error(), fixBadTemplate)
	}
	if err := checkMarkdownTemplate(xPath, xRaw); err != nil {
		return die(E_CHECK, shortBadTemplate, err.Error(), fixBadTemplate)
	}

	// 2) apply replacements
	pOut := applyTemplate(pRaw, ver)
	rOut := applyTemplate(rRaw, ver)
	xOut := applyTemplate(xRaw, ver)

	// 3) write md outputs
	if err := writeFileAtomic(tmpDir, fmt.Sprintf("PUBLISH_%s.md", ver), pOut); err != nil {
		return die(E_IO, shortWriteFailed, err.Error(), fixCheckPermissions)
	}
	if err := writeFileAtomic(tmpDir, fmt.Sprintf("RELEASE_BODY_%s.md", ver), rOut); err != nil {
		return die(E_IO, shortWriteFailed, err.Error(), fixCheckPermissions)
	}
	if err := writeFileAtomic(tmpDir, fmt.Sprintf("X_%s.md", ver), xOut); err != nil {
		return die(E_IO, shortWriteFailed, err.Error(), fixCheckPermissions)
	}

	// 4) AI_PACK (txt) - best effort for baseRef, but git must exist
	aiPack, code := buildAIPack(ver, baseRef)
	if code != E_OK {
		return code
	}
	if err := writeFileAtomic(tmpDir, fmt.Sprintf("AI_PACK_%s.txt", ver), aiPack); err != nil {
		return die(E_IO, shortWriteFailed, err.Error(), fixCheckPermissions)
	}

	return E_OK
}

func buildAIPack(ver, baseRef string) ([]byte, int) {
	var b strings.Builder
	b.WriteString("AI_PACK (local-only)\n")
	b.WriteString(fmt.Sprintf("Version: %s\n", ver))
	b.WriteString(fmt.Sprintf("BaseRef: %s\n", baseRef))
	b.WriteString(fmt.Sprintf("Head: %s\n", gitHeadSha()))
	b.WriteString("\n")

	// file list
	names, err := gitDiffNameStatus(baseRef)
	if err != nil {
		// base-ref missing is not fatal, but other git errors might be
		if !gitBaseExists(baseRef) {
			b.WriteString("ChangedFiles: n/a (base-ref not found)\n\n")
		} else {
			return nil, die(E_GEN, "git diff failed", err.Error(), "Ensure you are in a git repo and base-ref is valid (or fetch it)")
		}
	} else {
		b.WriteString("ChangedFiles (name-status):\n")
		for _, ln := range names {
			b.WriteString("  ")
			b.WriteString(ln)
			b.WriteString("\n")
		}
		b.WriteString("\n")
	}

	// stats
	ns := gitDiffNumStat(baseRef)
	if ns.nA {
		b.WriteString("DiffStats: n/a\n\n")
	} else {
		b.WriteString(fmt.Sprintf("DiffStats: files=%d, +%d, -%d\n\n", ns.n, ns.add, ns.del))
	}

	// unified diff (limited)
	diff, _, err := gitDiffUnifiedLimited(baseRef)
	if err != nil {
		// we already checked baseRef for name status, so maybe real error default fallback
		// try verify again just in case
		if !gitBaseExists(baseRef) {
			b.WriteString("Diff: n/a (base-ref not found)\n")
		} else {
			return nil, die(E_GEN, "git diff failed", err.Error(), "Ensure base-ref exists or fetch it")
		}
	} else {
		b.WriteString("Diff (unified):\n")
		b.Write(diff)
	}
	return []byte(b.String()), E_OK
}

func printGenSummary(ver, baseRef, out string) {
	infof("=== GENERATION SUMMARY ===")
	infof("Version:  %s", ver)
	infof("Base Ref: %s", baseRef)
	infof("Output:   %s", out)

	req := requiredArtifacts(ver)
	infof("[Artifacts]")
	for _, name := range req {
		p := filepath.Join(out, name)
		if !fileExists(p) {
			infof("FAIL  %s", name)
			continue
		}
		st, _ := os.Stat(p)
		infof("PASS  %-22s (%d B)", name, st.Size())
	}

	ns := gitDiffNumStat(baseRef)
	infof("[AI_Pack]")
	if ns.nA {
		infof("Lines:    n/a")
		infof("Context:  n/a")
	} else {
		infof("Lines:    +%d / -%d", ns.add, ns.del)
		infof("Context:  %d files changed", ns.n)
	}
	infof("[Action]")
	infof("> nix run .#status -- --version %s", ver)
}

// ---- check ----

type checkOpts struct {
	version string
}

func cmdCheck(args []string) int {
	fs := flag.NewFlagSet("check", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)

	var o checkOpts
	fs.StringVar(&o.version, "version", "", "optional version for dist validation (vX.Y.Z)")

	fs.Usage = func() {
		fmt.Fprintln(os.Stderr, "Usage: veil-aiux check [--version vX.Y.Z]")
		fs.PrintDefaults()
	}

	if err := fs.Parse(args); err != nil {
		return die(E_USAGE, shortInvalidArgs, err.Error(), "Run: veil-aiux check --help")
	}
	if extra := fs.Args(); len(extra) > 0 {
		return die(E_USAGE, shortUnexpectedArgs, fmt.Sprintf(fmtUnexpectedArgs, extra), fixRemoveExtraArgs)
	}

	// generic template checks (same rules as gen)
	paths := []struct {
		kind tplKind
		fix  string
	}{
		{tplPublish, "Create docs/ai/PUBLISH_TEMPLATE.md (or adjust templateCandidates)"},
		{tplReleaseBody, "Create docs/ai/RELEASE_BODY_TEMPLATE.md (or adjust templateCandidates)"},
		{tplX, "Create docs/ai/X_TEMPLATE.md (or adjust templateCandidates)"},
	}
	for _, it := range paths {
		p, err := resolveTemplate(it.kind)
		if err != nil {
			return die(E_CHECK, shortTemplateMissing, err.Error(), it.fix)
		}
		raw, err := readFile(p)
		if err != nil {
			return die(E_IO, shortReadTemplateFailed, err.Error(), fixCheckFilePermissions)
		}
		if err := checkMarkdownTemplate(p, raw); err != nil {
			return die(E_CHECK, shortBadTemplate, err.Error(), fixBadTemplate)
		}
	}

	if o.version == "" {
		infof("CHECK: OK (generic)")
		return E_OK
	}
	if err := validateVersion(o.version); err != nil {
		return die(E_USAGE, shortInvalidVersion, err.Error(), msgExampleVersion)
	}

	out := distOutDir(o.version)
	if !dirExists(out) {
		return die(E_CHECK, "Dist missing", fmt.Sprintf("missing: %s", out), "Run: nix run .#gen -- --version vX.Y.Z --clean")
	}
	if err := validateDistExactly4(out, o.version); err != nil {
		return die(E_CHECK, shortDistViolated, err.Error(), "Fix dist to contain exactly the 4 contract artifacts (AI_PACK must be .txt)")
	}

	infof("CHECK: OK (versioned) ver=%s", o.version)
	return E_OK
}

// ---- status ----

type statusOpts struct {
	version string
	noColor bool
}

func cmdStatus(args []string) int {
	fs := flag.NewFlagSet("status", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)

	var o statusOpts
	fs.StringVar(&o.version, "version", "", "target version (required), e.g. v0.14.0")
	fs.BoolVar(&o.noColor, "no-color", false, "disable ANSI color output")

	fs.Usage = func() {
		fmt.Fprintln(os.Stderr, "Usage: veil-aiux status --version vX.Y.Z")
		fs.PrintDefaults()
	}

	if err := fs.Parse(args); err != nil {
		return die(E_USAGE, shortInvalidArgs, err.Error(), "Run: veil-aiux status --help")
	}
	if extra := fs.Args(); len(extra) > 0 {
		return die(E_USAGE, shortUnexpectedArgs, fmt.Sprintf(fmtUnexpectedArgs, extra), fixRemoveExtraArgs)
	}
	if err := validateVersion(o.version); err != nil {
		return die(E_USAGE, shortInvalidVersion, err.Error(), msgExampleVersion)
	}

	out := distOutDir(o.version)
	infof("Output: %s", out)
	infof("Policy: AI_PACK is local-only (.txt). CI artifacts must be md-only.")

	ok := verifyArtifactsExistence(o.version, out)
	if !checkLeakRisk(out) {
		ok = false
	}

	_ = o.noColor
	if !ok {
		return E_CHECK
	}
	return E_OK
}

func verifyArtifactsExistence(ver, out string) bool {
	req := requiredArtifacts(ver)
	ok := true
	for _, name := range req {
		p := filepath.Join(out, name)
		if !fileExists(p) {
			infof("FAIL  %s", name)
			ok = false
			continue
		}
		st, _ := os.Stat(p)
		infof("PASS  %-22s (%d B)", name, st.Size())
	}
	return ok
}

func checkLeakRisk(out string) bool {
	ents, err := os.ReadDir(out)
	if err != nil {
		return true // if read fails, we can't check, but main error might already be caught
	}
	ok := true
	for _, e := range ents {
		if e.IsDir() {
			continue
		}
		n := e.Name()
		if strings.HasPrefix(n, "AI_PACK_") && strings.HasSuffix(n, ".md") {
			warnf("LEAK RISK: forbidden file exists: %s", filepath.Join(out, n))
			ok = false
		}
	}
	return ok
}
