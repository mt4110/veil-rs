package main

import (
	"archive/tar"
	"compress/gzip"
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

type VerifyReport struct {
	Contract *Contract

	// computed
	ComputedSHA256 map[string][32]byte

	// extracted raw files (needed for seal/checks)
	SHA256SUMS     []byte
	SHA256SUMSSeal []byte
	WarningsTxt    []byte

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

func VerifyBundlePath(path string) (*VerifyReport, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, WrapVError(E_PATH, path, err)
	}
	defer f.Close()
	return VerifyBundle(f)
}

func VerifyBundle(r io.Reader) (*VerifyReport, error) {
	rep, err := verifyReportFromStream(r)
	if err != nil {
		return nil, err
	}
	if err := verifyPostConditions(rep); err != nil {
		return nil, err
	}
	return rep, nil
}

func verifyReportFromStream(r io.Reader) (*VerifyReport, error) {
	rep := &VerifyReport{
		ComputedSHA256: make(map[string][32]byte),
	}

	gz, err := gzip.NewReader(r)
	if err != nil {
		return nil, WrapVError(E_GZIP, "stream", err)
	}
	defer gz.Close()

	// Capture gzip header for post-validation (against epoch)
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

		// 1. Ordering Check (Bytewise Lexicographic)
		// Directories in tar usually end with /, but sorting must be on full path
		// We use raw name for sorting check as per contract
		if seenFirst {
			if name < prevNameCanon {
				return nil, NewVError(E_ORDER, name, fmt.Sprintf("is not sorted (prev: %s)", prevNameCanon))
			}
		}
		prevNameCanon = name
		seenFirst = true

		// 2. Path Safety Check
		if strings.HasPrefix(name, "/") {
			return nil, NewVError(E_PATH, name, "absolute path forbidden")
		}
		if strings.Contains(name, "\x00") {
			return nil, NewVError(E_PATH, name, "contains NUL char")
		}
		if strings.Contains(name, "\\") {
			return nil, NewVError(E_PATH, name, "contains backslash")
		}
		// Clean check: path.Clean should match raw path (modulo trailing slash for dir)
		clean := path.Clean(name)
		if clean == ".." || strings.HasPrefix(clean, "../") {
			return nil, NewVError(E_PATH, name, "parent traversal prohibited")
		}
		// Normalize for comparison: remove trailing slash if present in name but not clean
		// (tar dirs SHOULD have trailing slash, but path.Clean removes it)
		// We check if Clean(name) is essentially the same structure.
		// Strict check: contract says "Clean(path) == path" (normalized)
		// If name has trailing slash, Clean removes it.
		// We allow trailing slash for Directory type only, but internal segments must be clean.
		normalized := clean
		if hdr.Typeflag == tar.TypeDir && !strings.HasSuffix(normalized, "/") {
			// append slash for comparison if original had it?
			// Actually, contract 3.1 says "Clean(path) == path".
			// Standard tar usage often has trailing slash for dirs.
			// Let's interpret "Clean(path) == path" as "no . or .. or //".
			// If we strict compare name to clean, we forbid trailing slash on dirs.
			// Let's relax slightly to allow trailing slash on Dir type, but strictly forbid . / .. / //
			if name != clean && name != clean+"/" {
				return nil, NewVError(E_PATH, name, "path not normalized")
			}
		} else {
			if name != clean {
				return nil, NewVError(E_PATH, name, "path not normalized")
			}
		}

		// 3. Type Allowlist
		switch hdr.Typeflag {
		case tar.TypeDir, tar.TypeReg, tar.TypeSymlink:
			// OK
		default:
			return nil, NewVError(E_TYPE, name, fmt.Sprintf("forbidden type flag: %c", hdr.Typeflag))
		}

		// 4. Identity Leak Check
		if hdr.Uid != 0 || hdr.Gid != 0 {
			return nil, NewVError(E_IDENTITY, name, fmt.Sprintf("non-zero uid/gid: %d/%d", hdr.Uid, hdr.Gid))
		}
		if hdr.Uname != "" || hdr.Gname != "" {
			return nil, NewVError(E_IDENTITY, name, fmt.Sprintf("non-empty uname/gname: %q/%q", hdr.Uname, hdr.Gname))
		}

		// 5. Time consistency (epoch check logic)
		// We don't know epoch yet (in C1), but we can enforce consistency:
		// All entries must have nanos=0 and same integer seconds.
		ts := hdr.ModTime
		if ts.Nanosecond() != 0 {
			return nil, NewVError(E_TIME, name, "non-zero nanoseconds forbidden")
		}
		if seenSec == 0 {
			seenSec = ts.Unix()
		} else {
			if ts.Unix() != seenSec {
				return nil, NewVError(E_TIME, name, fmt.Sprintf("mtime mismatch (expected %d, got %d)", seenSec, ts.Unix()))
			}
		}

		// C2/C3 logic placeholders (Stream scan continues)
	}

	return rep, nil
}

func verifyPostConditions(rep *VerifyReport) error {
	// TODO: implement C3
	return nil
}
