package main

import (
	"archive/tar"
	"compress/gzip"
	"crypto/sha256"
	"fmt"
	"io"
	"os"
	"path"
	"strings"
	"time"
)

type Contract struct {
	ContractVersion string   `json:"contract_version"`
	Mode            string   `json:"mode"`
	Repo            string   `json:"repo"`
	EpochSec        int64    `json:"epoch_sec"`
	BaseRef         string   `json:"base_ref"`
	HeadSHA         string   `json:"head_sha"`
	WarningsCount   int      `json:"warnings_count"`
	Evidence        Evidence `json:"evidence"`
	Tool            Tool     `json:"tool"`
}

type Evidence struct {
	Required    bool   `json:"required"`
	Present     bool   `json:"present"`
	BoundToHead bool   `json:"bound_to_head"`
	PathPrefix  string `json:"path_prefix"`
}

type Tool struct {
	Name    string `json:"name"`
	Version string `json:"version"`
	Build   string `json:"build,omitempty"`
}

type VerifyOptions struct {
	BudgetBytes  int64
	BudgetFiles  int
	EvidenceScan bool
}

// DefaultVerifyOptions provides standard limits: 100MB, 10,000 files, and active evidence scanning.
var DefaultVerifyOptions = VerifyOptions{
	BudgetBytes:  100 * 1024 * 1024,
	BudgetFiles:  10000,
	EvidenceScan: true,
}

type VerifyReport struct {
	Contract *Contract
	Opts     *VerifyOptions

	// budget tracking
	UsedBytes int64
	UsedFiles int

	// computed
	ComputedSHA256 map[string][32]byte
	TruncatedFiles map[string]bool // new: track files > 4MB
	ActualFiles    map[string]struct{}

	// extracted raw files (needed for seal/checks)
	SHA256SUMS     []byte
	SHA256SUMSSeal []byte
	WarningsTxt    []byte
	EvidenceFiles  [][]byte

	// required layout presence
	HasIndex          bool
	HasContractJSON   bool
	HasSHA256SUMS     bool
	HasSHA256SUMSSeal bool
	HasSeriesPatch    bool

	// evidence scan result
	EvidencePresent     bool
	EvidenceBoundToHead bool

	// captured gzip header (for post-check)
	GzipModTime time.Time
	GzipName    string
	GzipComment string
	GzipExtra   []byte
	GzipOS      byte
}

func VerifyBundlePath(path string, opts VerifyOptions) (*VerifyReport, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, WrapVError(E_PATH, path, err)
	}
	defer f.Close()
	return VerifyBundle(f, opts)
}

func VerifyBundle(r io.Reader, opts VerifyOptions) (*VerifyReport, error) {
	rep, err := verifyReportFromStream(r, opts)
	if err != nil {
		return nil, err
	}
	if err := verifyPostConditions(rep); err != nil {
		return nil, err
	}
	return rep, nil
}

func verifyReportFromStream(r io.Reader, opts VerifyOptions) (*VerifyReport, error) {
	rep := &VerifyReport{
		Opts:           &opts,
		ComputedSHA256: make(map[string][32]byte),
		TruncatedFiles: make(map[string]bool),
		ActualFiles:    make(map[string]struct{}),
	}

	gz, err := gzip.NewReader(r)
	if err != nil {
		return nil, WrapVError(E_GZIP, "stream", err)
	}
	defer gz.Close()

	// Capture gzip header
	rep.GzipModTime = gz.Header.ModTime
	rep.GzipName = gz.Header.Name
	rep.GzipComment = gz.Header.Comment
	rep.GzipExtra = gz.Header.Extra
	rep.GzipOS = gz.Header.OS

	tr := tar.NewReader(gz)
	var prevNameCanon string
	var seenFirst bool
	var seenSec int64

	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, WrapVError(E_GZIP, "tar stream corrupted", err)
		}

		name := hdr.Name

		if rep.Opts.BudgetFiles > 0 {
			rep.UsedFiles++
			if rep.UsedFiles > rep.Opts.BudgetFiles {
				return nil, NewVError(E_BUDGET, name, "file count exceeds budget").WithReason("budget_exceeded")
			}
		}

		if err := validateTarOrder(name, prevNameCanon, seenFirst); err != nil {
			return nil, err
		}
		prevNameCanon = name
		seenFirst = true

		if err := validateTarPath(name, hdr.Typeflag); err != nil {
			return nil, err
		}
		if err := validateTarType(name, hdr.Typeflag); err != nil {
			return nil, err
		}
		if err := validateTarIdentity(name, hdr); err != nil {
			return nil, err
		}
		if err := validateTarTime(name, hdr, &seenSec); err != nil {
			return nil, err
		}
		if err := validateTarPAX(name, hdr); err != nil {
			return nil, err
		}

		if hdr.Typeflag != tar.TypeDir {
			rep.ActualFiles[name] = struct{}{}
		}

		updateLayoutPresence(name, hdr, rep)

		if err := processEntryContent(tr, hdr, rep); err != nil {
			return nil, err
		}
	}

	return rep, nil
}

func validateTarOrder(name, prev string, seenFirst bool) error {
	if seenFirst && name < prev {
		return NewVError(E_ORDER, name, fmt.Sprintf("is not sorted (prev: %s)", prev))
	}
	return nil
}

func validateTarPath(name string, typeFlag byte) error {
	if strings.HasPrefix(name, "/") {
		return NewVError(E_PATH, name, "absolute path forbidden")
	}
	if strings.Contains(name, "\x00") {
		return NewVError(E_PATH, name, "contains NUL char")
	}
	if strings.Contains(name, "\\") {
		return NewVError(E_PATH, name, "contains backslash")
	}
	clean := path.Clean(name)
	if clean == ".." || strings.HasPrefix(clean, "../") {
		return NewVError(E_PATH, name, "parent traversal prohibited")
	}

	// Normalize check
	normalized := clean
	if typeFlag == tar.TypeDir && !strings.HasSuffix(normalized, "/") {
		if name != clean && name != clean+"/" {
			return NewVError(E_PATH, name, "path not normalized")
		}
	} else {
		if name != clean {
			return NewVError(E_PATH, name, "path not normalized")
		}
	}
	return nil
}

func validateTarType(name string, flag byte) error {
	switch flag {
	case tar.TypeDir, tar.TypeReg, tar.TypeSymlink:
		return nil
	default:
		return NewVError(E_TYPE, name, fmt.Sprintf("forbidden type flag: %c", flag))
	}
}

func validateTarIdentity(name string, hdr *tar.Header) error {
	if hdr.Uid != 0 || hdr.Gid != 0 {
		return NewVError(E_IDENTITY, name, fmt.Sprintf("non-zero uid/gid: %d/%d", hdr.Uid, hdr.Gid))
	}
	if hdr.Uname != "" || hdr.Gname != "" {
		return NewVError(E_IDENTITY, name, fmt.Sprintf("non-empty uname/gname: %q/%q", hdr.Uname, hdr.Gname))
	}

	// Phase 7.6: Mode Normalization
	mode := hdr.Mode & 0777
	switch hdr.Typeflag {
	case tar.TypeDir:
		if mode != 0755 {
			return NewVError(E_IDENTITY, name, fmt.Sprintf("dir mode must be 0755 (got %o)", mode))
		}
	case tar.TypeReg:
		// Check for executable bit in git's bitmask sense
		// If executable (0755) or regular (0644)
		if mode != 0644 && mode != 0755 {
			return NewVError(E_IDENTITY, name, fmt.Sprintf("regular file mode must be 0644 or 0755 (got %o)", mode))
		}
	case tar.TypeSymlink:
		// Symlink mode is NOT validated (Phase 7.6)
	}

	return nil
}

func validateTarTime(name string, hdr *tar.Header, seenSec *int64) error {
	ts := hdr.ModTime
	if ts.Nanosecond() != 0 {
		return NewVError(E_TIME, name, "non-zero nanoseconds forbidden")
	}
	if *seenSec == 0 {
		*seenSec = ts.Unix()
	} else {
		if ts.Unix() != *seenSec {
			return NewVError(E_TIME, name, fmt.Sprintf("mtime mismatch (expected %d, got %d)", *seenSec, ts.Unix()))
		}
	}
	return nil
}

func validateTarPAX(name string, hdr *tar.Header) error {
	if len(hdr.PAXRecords) > 0 {
		for k := range hdr.PAXRecords {
			// Phase 4.2: strict allowlist (path/linkpath only)
			if k == "path" || k == "linkpath" {
				continue
			}
			if k == "mtime" || k == "atime" || k == "ctime" {
				return NewVError(E_PAX, name, "forbidden PAX time key: "+k)
			}
			if strings.HasPrefix(k, "LIBARCHIVE.") || strings.HasPrefix(k, "SCHILY.xattr.") {
				return NewVError(E_XATTR, name, "xattr/provenance leak: "+k)
			}
			return NewVError(E_PAX, name, "forbidden PAX key (not in allowlist): "+k)
		}
	}
	if len(hdr.Xattrs) > 0 {
		return NewVError(E_XATTR, name, "xattr map present")
	}
	return nil
}

func updateLayoutPresence(name string, hdr *tar.Header, rep *VerifyReport) {
	switch {
	case name == PathIndex:
		rep.HasIndex = true
	case name == PathContract:
		rep.HasContractJSON = true
	case name == PathSHA256SUMS:
		rep.HasSHA256SUMS = true
	case name == PathSHA256SUMSSeal:
		rep.HasSHA256SUMSSeal = true
	case name == PathSeriesPatch:
		rep.HasSeriesPatch = true
	case strings.HasPrefix(name, DirEvidence):
		if hdr.Typeflag != tar.TypeDir {
			rep.EvidencePresent = true
		}
	}
}

func processEntryContent(tr *tar.Reader, hdr *tar.Header, rep *VerifyReport) error {
	if hdr.Typeflag == tar.TypeDir {
		return nil
	}

	name := hdr.Name
	var hash [32]byte

	if hdr.Typeflag == tar.TypeSymlink {
		if path.IsAbs(hdr.Linkname) || strings.Contains(hdr.Linkname, "..") {
			return NewVError(E_PATH, name, "unsafe symlink target: "+hdr.Linkname)
		}
		data := []byte("symlink\x00" + hdr.Linkname)
		hash = sha256.Sum256(data)
	} else {
		// Regular file
		isMeta := name == PathSHA256SUMS ||
			name == PathSHA256SUMSSeal ||
			name == PathContract ||
			name == PathWarnings ||
			strings.HasPrefix(name, DirEvidence)

		if isMeta {
			// Phase 7.5: 4MB limit for meta/evidence parsing
			lr := io.LimitReader(tr, 4*1024*1024)
			content, err := io.ReadAll(lr)
			if err != nil {
				return WrapVError(E_GZIP, name, err)
			}

			// Account for read bytes in budget (Codex P1 feedback)
			rep.UsedBytes += int64(len(content))
			if rep.Opts.BudgetBytes > 0 && rep.UsedBytes > rep.Opts.BudgetBytes {
				return NewVError(E_BUDGET, name, "byte count exceeds budget").WithReason("budget_exceeded")
			}

			// Check if we hit the limit
			if int64(len(content)) == 4*1024*1024 {
				// Peek one byte to see if there's more
				var b [1]byte
				n, _ := tr.Read(b[:])
				if n > 0 {
					rep.TruncatedFiles[name] = true

					// Codex P1 Fix: Count the remainder of the truncated file towards the budget
					remaining := hdr.Size - int64(len(content))
					if remaining > 0 {
						rep.UsedBytes += remaining
						if rep.Opts.BudgetBytes > 0 && rep.UsedBytes > rep.Opts.BudgetBytes {
							return NewVError(E_BUDGET, name, "byte count exceeds budget").WithReason("budget_exceeded")
						}
					}

					// Phase 7.7: Evidence scanning even for truncated files (Codex P2 feedback)
					if rep.Opts.EvidenceScan && strings.HasPrefix(name, DirEvidence) {
						if err := scanEvidenceContent(name, content); err != nil {
							return err
						}
					}

					return nil
				}
			}

			// Phase 7.7: Evidence scanning (normal files)
			if rep.Opts.EvidenceScan && strings.HasPrefix(name, DirEvidence) {
				if err := scanEvidenceContent(name, content); err != nil {
					return err
				}
			}

			hash = sha256.Sum256(content)

			if err := storeMetaContent(name, content, rep); err != nil {
				return err
			}
		} else {
			h := sha256.New()
			buf := make([]byte, 32*1024)
			for {
				n, readErr := tr.Read(buf)
				if n > 0 {
					rep.UsedBytes += int64(n)
					if rep.Opts.BudgetBytes > 0 && rep.UsedBytes > rep.Opts.BudgetBytes {
						return NewVError(E_BUDGET, name, "byte count exceeds budget").WithReason("budget_exceeded")
					}
					h.Write(buf[:n])
				}
				if readErr != nil {
					if readErr == io.EOF {
						break
					}
					return WrapVError(E_GZIP, name, readErr)
				}
			}
			copy(hash[:], h.Sum(nil))
		}
	}

	rep.ComputedSHA256[name] = hash
	return nil
}

func storeMetaContent(name string, content []byte, rep *VerifyReport) error {
	switch name {
	case PathSHA256SUMS:
		rep.SHA256SUMS = content
	case PathSHA256SUMSSeal:
		rep.SHA256SUMSSeal = content
	case PathContract:
		c, err := ParseContractJSON(content)
		if err != nil {
			return err
		}
		rep.Contract = c
	case PathWarnings:
		rep.WarningsTxt = content
	default:
		if strings.HasPrefix(name, DirEvidence) {
			rep.EvidenceFiles = append(rep.EvidenceFiles, content)
		}
	}
	return nil
}

func verifyPostConditions(rep *VerifyReport) error {
	if err := validateContractAndEpoch(rep); err != nil {
		return err
	}
	if err := validateGzipHeader(rep); err != nil {
		return err
	}
	if err := validateLayout(rep); err != nil {
		return err
	}
	if err := validateManifest(rep); err != nil {
		return err
	}
	if err := validateEvidenceBinding(rep); err != nil {
		return err
	}
	return nil
}

func validateContractAndEpoch(rep *VerifyReport) error {
	if !rep.HasContractJSON {
		return NewVError(E_LAYOUT, PathContract, "missing essential metadata")
	}
	if rep.Contract == nil {
		return NewVError(E_CONTRACT, PathContract, "failed to parse")
	}
	return ValidateContractV11(rep.Contract)
}

func validateGzipHeader(rep *VerifyReport) error {
	epoch := rep.Contract.EpochSec
	if rep.GzipModTime.Unix() != epoch {
		return NewVError(E_GZIP, "header", fmt.Sprintf("mtime mismatch (header: %d, contract: %d)", rep.GzipModTime.Unix(), epoch))
	}
	// Phase 7.2: strictly 255
	if rep.GzipOS != 255 {
		return NewVError(E_GZIP, "header", fmt.Sprintf("OS byte must be 255 (got %d)", rep.GzipOS))
	}
	if rep.GzipName != "" {
		return NewVError(E_GZIP, "header", "Name must be empty")
	}
	if rep.GzipComment != "" {
		return NewVError(E_GZIP, "header", "Comment must be empty")
	}
	if len(rep.GzipExtra) > 0 {
		return NewVError(E_GZIP, "header", "Extra data must be empty")
	}
	return nil
}

func validateLayout(rep *VerifyReport) error {
	if !rep.HasIndex {
		return NewVError(E_LAYOUT, PathIndex, "missing")
	}
	if !rep.HasSHA256SUMS {
		return NewVError(E_LAYOUT, PathSHA256SUMS, "missing")
	}
	if !rep.HasSHA256SUMSSeal {
		return NewVError(E_LAYOUT, PathSHA256SUMSSeal, "missing")
	}
	if !rep.HasSeriesPatch {
		return NewVError(E_LAYOUT, PathSeriesPatch, "missing")
	}
	if !rep.EvidencePresent && rep.Contract.Evidence.Required {
		return NewVError(E_EVIDENCE, DirEvidence, "required but missing").WithReason("contract_evidence_mismatch")
	}
	if rep.Contract.Mode == "strict" && !rep.Contract.Evidence.Required {
		return NewVError(E_CONTRACT, "contract.json", "strict mode requires evidence.required=true").WithReason("contract_invalid")
	}
	if rep.Contract.Evidence.Present != rep.EvidencePresent {
		return NewVError(E_CONTRACT, "contract.json", "evidence.present mismatch").WithReason("contract_evidence_mismatch")
	}
	// Validate path_prefix whenever it is set (not just when Required=true),
	// so bundles with evidence.present=true but required=false are also checked.
	if rep.Contract.Evidence.PathPrefix != "" {
		if !strings.HasPrefix(rep.Contract.Evidence.PathPrefix, "review/evidence/") || !strings.HasSuffix(rep.Contract.Evidence.PathPrefix, "/") {
			return NewVError(E_CONTRACT, "contract.json", "invalid evidence.path_prefix").WithReason("contract_invalid")
		}
	}
	return nil
}

func validateManifest(rep *VerifyReport) error {
	checksums, err := ParseSHA256SUMS(rep.SHA256SUMS)
	if err != nil {
		return err
	}

	for _, line := range checksums {
		// SHA256SUMS must not list itself or its seal file.
		// Only the exact seal path is forbidden, not all *.sha256 files
		// (bundles may legitimately contain payload files with a .sha256 extension).
		if line.Path == PathSHA256SUMS || line.Path == PathSHA256SUMSSeal {
			return NewVError(E_SHA256, line.Path, "manifest must not list itself or its seal file").WithReason("manifest_invalid")
		}
	}

	// Filter out truncated files from expected checksums
	// because we skipped hashing them, so they verify as "missing" otherwise.
	var filteredChecksums []ChecksumLine
	for _, c := range checksums {
		if rep.TruncatedFiles[c.Path] {
			continue // Skip checksum validation for truncated files
		}
		filteredChecksums = append(filteredChecksums, c)
	}

	if err := VerifySHA256SUMSSeal(rep.SHA256SUMS, rep.SHA256SUMSSeal); err != nil {
		return err
	}

	allowSet := make(map[string]struct{})
	for _, c := range checksums {
		allowSet[c.Path] = struct{}{}
	}
	allowSet[PathSHA256SUMS] = struct{}{}
	allowSet[PathSHA256SUMSSeal] = struct{}{}

	for p := range rep.ActualFiles {
		if _, ok := allowSet[p]; !ok {
			return NewVError(E_EXTRA, p, "file not in manifest").WithReason("file_extra")
		}
	}
	for p := range allowSet {
		if _, ok := rep.ActualFiles[p]; !ok {
			return NewVError(E_MISSING, p, "file missing in bundle").WithReason("file_missing")
		}
	}

	return VerifyChecksumCompleteness(filteredChecksums, rep.ComputedSHA256)
}

func validateEvidenceBinding(rep *VerifyReport) error {
	mode := rep.Contract.Mode
	headSHA := rep.Contract.HeadSHA

	if mode == ModeStrict {
		if !rep.EvidencePresent {
			return NewVError(E_EVIDENCE, "strict_mode", "evidence files required in strict mode")
		}
		bound := false
		// Note: rep.EvidenceFiles does NOT contain truncated files content because processEntryContent returns early.
		for _, content := range rep.EvidenceFiles {
			if strings.Contains(string(content), headSHA) {
				bound = true
				break
			}
		}
		// Also strict mode requires all present evidence to be bindable?
		// Or just "some evidence binds"?
		// "bound=false" if >4MB.
		// If ANY evidence binds, we are good?
		// Task says: "strict: fail if evidence required but unbound"
		// This likely means if NO evidence binds.

		if !bound {
			// Check if we have truncated evidence files that might have bound if we read them?
			// But >4MB is explicitly "bound=false".
			// So if we only have >4MB evidence, bound is false.
			return NewVError(E_EVIDENCE, "binding", fmt.Sprintf("no evidence file contains HEAD SHA %s", headSHA))
		}
		rep.EvidenceBoundToHead = true
	} else if mode == ModeWIP {
		if rep.Contract.WarningsCount > 0 {
			if len(rep.WarningsTxt) == 0 {
				return NewVError(E_LAYOUT, PathWarnings, "missing (warnings_count > 0)")
			}
		}
	}
	return nil
}
