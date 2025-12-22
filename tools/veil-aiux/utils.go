package main

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

func infof(format string, a ...any) { fmt.Fprintf(os.Stdout, format+"\n", a...) }
func warnf(format string, a ...any) { fmt.Fprintf(os.Stderr, "WARN: "+format+"\n", a...) }

func die(code int, short string, reason string, fix string) int {
	if short == "" {
		short = "Failure"
	}
	fmt.Fprintf(os.Stderr, "ERROR[%d]: %s\n", code, short)
	if reason != "" {
		fmt.Fprintf(os.Stderr, "Reason: %s\n", reason)
	}
	if fix != "" {
		fmt.Fprintf(os.Stderr, "Fix: %s\n", fix)
	}
	return code
}

// ---- Validation / Guards ----

var verRe = regexp.MustCompile(`^v\d+\.\d+\.\d+$`)

func validateVersion(ver string) error {
	if ver == "" {
		return errors.New("--version is required")
	}
	if !verRe.MatchString(ver) {
		return fmt.Errorf("invalid version format: %q (expected vX.Y.Z)", ver)
	}
	if strings.Contains(ver, "/") || strings.Contains(ver, "\\") || strings.Contains(ver, "..") {
		return fmt.Errorf("invalid version contains forbidden path tokens: %q", ver)
	}
	return nil
}

func ensureDir(path string) error {
	return os.MkdirAll(path, 0o755)
}

func dirExists(path string) bool {
	st, err := os.Stat(path)
	return err == nil && st.IsDir()
}

func fileExists(path string) bool {
	st, err := os.Stat(path)
	return err == nil && !st.IsDir()
}

func readFile(path string) ([]byte, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return b, nil
}

// ---- Atomic Writes (inside tmp dir) ----

func writeFileAtomic(dir, name string, data []byte) error {
	if err := ensureDir(dir); err != nil {
		return err
	}
	tmp := filepath.Join(dir, "."+name+".tmp")
	final := filepath.Join(dir, name)
	if err := os.WriteFile(tmp, data, 0o644); err != nil {
		return err
	}
	return os.Rename(tmp, final)
}
