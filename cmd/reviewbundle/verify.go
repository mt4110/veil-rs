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

type VerifyReport struct {
	Contract *Contract

	// computed
	ComputedSHA256 map[string][32]byte

	// extracted raw files (needed for seal/checks)
	SHA256SUMS     []byte
	SHA256SUMSSeal []byte
	WarningsTxt    []byte
	EvidenceFiles  [][]byte // buffered evidence content for binding check

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
		clean := path.Clean(name)
		if clean == ".." || strings.HasPrefix(clean, "../") {
			return nil, NewVError(E_PATH, name, "parent traversal prohibited")
		}
		normalized := clean
		if hdr.Typeflag == tar.TypeDir && !strings.HasSuffix(normalized, "/") {
			// Allow trailing slash for Dir type, but strictly forbid . / .. / //
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

		// 6. PAX Header Check (C2: allowlist, forbid time/xattr)
		if len(hdr.PAXRecords) > 0 {
			for k := range hdr.PAXRecords {
				if k != "path" && k != "linkpath" {
					if k == "mtime" || k == "atime" || k == "ctime" {
						return nil, NewVError(E_PAX, name, "forbidden PAX time key: "+k)
					}
					if strings.HasPrefix(k, "LIBARCHIVE.") || strings.HasPrefix(k, "SCHILY.xattr.") {
						return nil, NewVError(E_XATTR, name, "xattr/provenance leak: "+k)
					}
					return nil, NewVError(E_PAX, name, "forbidden PAX key: "+k)
				}
			}
		}
		if len(hdr.Xattrs) > 0 {
			return nil, NewVError(E_XATTR, name, "xattr map present")
		}

		// 7. Track Required Layout Presence & Evidence
		switch {
		case name == "review/INDEX.md":
			rep.HasIndex = true
		case name == "review/meta/contract.json":
			rep.HasContractJSON = true
		case name == "review/meta/SHA256SUMS":
			rep.HasSHA256SUMS = true
		case name == "review/meta/SHA256SUMS.sha256":
			rep.HasSHA256SUMSSeal = true
		case name == "review/patch/series.patch":
			rep.HasSeriesPatch = true
		case strings.HasPrefix(name, "review/evidence/"):
			if hdr.Typeflag != tar.TypeDir {
				rep.EvidencePresent = true
			}
		}

		// 8. Compute SHA256 (C2)
		if hdr.Typeflag == tar.TypeDir {
			continue
		}

		var hash [32]byte
		var content []byte
		var readErr error

		if hdr.Typeflag == tar.TypeSymlink {
			data := []byte("symlink\x00" + hdr.Linkname)
			hash = sha256.Sum256(data)

			if path.IsAbs(hdr.Linkname) || strings.Contains(hdr.Linkname, "..") {
				return nil, NewVError(E_PATH, name, "unsafe symlink target: "+hdr.Linkname)
			}
		} else {
			isMeta := name == "review/meta/SHA256SUMS" ||
				name == "review/meta/SHA256SUMS.sha256" ||
				name == "review/meta/contract.json" ||
				name == "review/meta/warnings.txt" ||
				strings.HasPrefix(name, "review/evidence/")

			if isMeta {
				content, readErr = io.ReadAll(tr)
				if readErr != nil {
					return nil, WrapVError(E_GZIP, name, readErr)
				}
				hash = sha256.Sum256(content)

				switch name {
				case "review/meta/SHA256SUMS":
					rep.SHA256SUMS = content
				case "review/meta/SHA256SUMS.sha256":
					rep.SHA256SUMSSeal = content
				case "review/meta/contract.json":
					c, err := ParseContractJSON(content)
					if err != nil {
						return nil, err
					}
					rep.Contract = c
				case "review/meta/warnings.txt":
					rep.WarningsTxt = content
				default:
					if strings.HasPrefix(name, "review/evidence/") {
						rep.EvidenceFiles = append(rep.EvidenceFiles, content)
					}
				}
			} else {
				h := sha256.New()
				if _, err := io.Copy(h, tr); err != nil {
					return nil, WrapVError(E_GZIP, name, err)
				}
				copy(hash[:], h.Sum(nil))
			}
		}

		rep.ComputedSHA256[name] = hash
	}

	return rep, nil
}

func verifyPostConditions(rep *VerifyReport) error {
	// TODO: implement C3
	return nil
}
