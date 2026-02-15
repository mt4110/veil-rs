package prkit

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"
	"unicode/utf8"
)

// Global state for compatibility and trace collection
var (
	Runner         ExecRunner
	ExecutionTrace []Command
	traceMu        sync.Mutex
)

func init() {
	// Ensure Runner is never nil by default
	Runner = &ProdExecRunner{}
}

// ResetTrace clears the execution trace.
func ResetTrace() {
	traceMu.Lock()
	defer traceMu.Unlock()
	ExecutionTrace = []Command{}
}

// Init initializes the prkit global state.
func Init(repoRoot string, r ExecRunner) {
	if r != nil {
		Runner = r
	} else {
		Runner = &ProdExecRunner{RepoRoot: repoRoot}
	}
}

// ExecRunner defines the interface for running external commands.
type ExecRunner interface {
	Run(ctx context.Context, spec ExecSpec) ExecResult
}

// ExecSpec defines the execution specification.
type ExecSpec struct {
	Name      string   // Command name (e.g., "git")
	Argv      []string // Arguments (renamed from Args to match usage)
	Dir       string   // Working directory
	Env       []string // Optional environment variables (KEY=VALUE)
	TimeoutMs int      // Optional timeout in milliseconds
}

// ExecResult captures the result of an execution.
type ExecResult struct {
	Stdout   string
	Stderr   string
	ExitCode int
	Error    error
	// Helpers for errors
	ErrorKind string
}

// ProdExecRunner is the production implementation of ExecRunner.
type ProdExecRunner struct {
	RepoRoot string
}

// Run executes the command using os/exec and records evidence.
func (r *ProdExecRunner) Run(ctx context.Context, spec ExecSpec) ExecResult {
	// 1. Resolve Name/Argv
	name, args, err := resolveArgv(spec)
	if err != nil {
		return ExecResult{
			ExitCode:  -1,
			ErrorKind: "InvalidSpec",
			Error:     err,
		}
	}

	// 2. Resolve Dir
	absDir, cleanRel, err := r.resolveDir(spec.Dir)
	if err != nil {
		return ExecResult{
			ExitCode:  -1,
			ErrorKind: "SecurityViolation",
			Error:     err,
		}
	}

	var cancel context.CancelFunc
	ctxToUse := ctx
	if spec.TimeoutMs > 0 {
		var timeoutCtx context.Context
		timeoutCtx, cancel = context.WithTimeout(ctx, time.Duration(spec.TimeoutMs)*time.Millisecond)
		ctxToUse = timeoutCtx
	}
	if cancel != nil {
		defer cancel()
	}

	cmd := exec.CommandContext(ctxToUse, name, args...)
	cmd.Dir = absDir

	// 3. Resolve Env
	finalEnv, envMode, envOverrides := resolveEnv(spec.Env)
	cmd.Env = finalEnv

	var stdoutBuf, stderrBuf bytes.Buffer
	cmd.Stdout = &stdoutBuf
	cmd.Stderr = &stderrBuf

	start := Now()
	eErr := cmd.Run()
	_ = Now().Sub(start)

	// Result Construction
	res := ExecResult{
		Stdout:   normalizeOutput(stdoutBuf.Bytes()),
		Stderr:   normalizeOutput(stderrBuf.Bytes()),
		ExitCode: 0,
		Error:    eErr,
	}

	if eErr != nil {
		if exitErr, ok := eErr.(*exec.ExitError); ok {
			res.ExitCode = exitErr.ExitCode()
			res.ErrorKind = "ExitError"
		} else {
			res.ExitCode = 1
			res.ErrorKind = fmt.Sprintf("ExecError: %T", eErr)
		}
	}

	// 4. Redaction & Evidence
	redactedRes, cmdRecord := r.prepareEvidence(res, name, args, cleanRel, envMode, finalEnv, envOverrides)

	traceMu.Lock()
	ExecutionTrace = append(ExecutionTrace, cmdRecord)
	traceMu.Unlock()

	return redactedRes
}

func resolveArgv(spec ExecSpec) (string, []string, error) {
	if len(spec.Argv) == 0 {
		return "", nil, fmt.Errorf("ExecSpec.Argv must not be empty (must contain executable name at [0])")
	}

	if spec.Name != "" {
		// Legacy/Compat mode: Name is executable, Argv is arguments
		return spec.Name, spec.Argv, nil
	}
	// Standard mode: Argv[0] is executable
	name := spec.Argv[0]
	var args []string
	if len(spec.Argv) > 1 {
		args = spec.Argv[1:]
	}
	return name, args, nil
}

func (r *ProdExecRunner) resolveDir(specDir string) (string, string, error) {
	relDir := specDir
	if relDir == "" {
		relDir = "."
	}

	if filepath.IsAbs(relDir) {
		return "", "", fmt.Errorf("ExecSpec.Dir must be relative to repo root, got absolute path: %s", relDir)
	}

	cleanRel := filepath.Clean(relDir)
	if strings.HasPrefix(cleanRel, "..") {
		return "", "", fmt.Errorf("ExecSpec.Dir escapes repo root: %s", relDir)
	}

	if r.RepoRoot == "" {
		return "", "", fmt.Errorf("ExecRunner RepoRoot is not set; cannot resolve directory %q", relDir)
	}
	absDir := filepath.Join(r.RepoRoot, cleanRel)
	return absDir, cleanRel, nil
}

func resolveEnv(specEnv []string) ([]string, string, []string) {
	if len(specEnv) > 0 {
		// Strict mode? No, plan says "inherit+delta" always unless we want pure strict.
		// Wait, usage in tools.go or others might rely on strict implementation if previous implementation implied it?
		// Previous implementation:
		// if len(spec.Env) > 0 { env = spec.Env; envMode = "strict" } else { env = os.Environ(); envMode = "inherit" }
		//
		// New plan (Phase 4):
		// - RULE: ExecSpec.Env は “差分（override）” として解釈
		// - 実効 env = inherit_host_env + overrides
		//
		// So checking if previous impl used strict is important.
		// If I change to always inherit, I might break things expecting clean env.
		// But plan explicitly calls for "env契約の硬化（inherit+delta）".
		// I will implement "inherit+delta".

		currentEnv := os.Environ()
		envMap := make(map[string]string)
		for _, kv := range currentEnv {
			parts := strings.SplitN(kv, "=", 2)
			if len(parts) == 2 {
				envMap[parts[0]] = parts[1]
			}
		}

		for _, kv := range specEnv {
			parts := strings.SplitN(kv, "=", 2)
			if len(parts) == 2 {
				envMap[parts[0]] = parts[1]
			} else {
				// Warn on malformed env var (PR Review Feedback)
				fmt.Fprintf(os.Stderr, "Warning: Ignoring malformed environment variable: %q\n", kv)
			}
		}

		finalEnv := make([]string, 0, len(envMap))
		for k, v := range envMap {
			finalEnv = append(finalEnv, fmt.Sprintf("%s=%s", k, v))
		}
		sort.Strings(finalEnv)

		// Overrides for recording
		overrides := make([]string, len(specEnv))
		copy(overrides, specEnv)
		sort.Strings(overrides)

		return finalEnv, "inherit+delta", overrides
	}

	// No overrides -> pure inherit
	env := os.Environ()
	return env, "inherit", nil
}

func (r *ProdExecRunner) prepareEvidence(res ExecResult, name string, args []string, cleanRel string, envMode string, finalEnv []string, envOverrides []string) (ExecResult, Command) {
	// Redact stdout/stderr
	res.Stdout = r.redact(res.Stdout)
	res.Stderr = r.redact(res.Stderr)

	// Redact Argv
	fullArgv := append([]string{name}, args...)
	redactedArgv := make([]string, len(fullArgv))
	for i, a := range fullArgv {
		redactedArgv[i] = r.redact(a)
	}

	cmdRecord := Command{
		Argv:      redactedArgv,
		CwdRel:    cleanRel,
		EnvMode:   envMode,
		EnvHash:   hashEnv(finalEnv),
		EnvKV:     envOverrides,
		ExitCode:  res.ExitCode,
		Stdout:    res.Stdout,
		Stderr:    res.Stderr,
		ErrorKind: res.ErrorKind,
	}

	return res, cmdRecord
}

func (r *ProdExecRunner) redact(s string) string {
	if r.RepoRoot == "" {
		return s
	}
	return strings.ReplaceAll(s, r.RepoRoot, "<REPO_ROOT>")
}

func normalizeOutput(b []byte) string {
	if !utf8.Valid(b) {
		s := string(bytes.ToValidUTF8(b, []byte("?")))
		return strings.TrimSpace(s)
	}
	return strings.TrimSpace(string(b))
}

func hashEnv(env []string) string {
	// Create a copy to sort, to avoid mutating the original slice if it's used elsewhere
	sorted := make([]string, len(env))
	copy(sorted, env)
	sort.Strings(sorted)

	h := sha256.New()
	for _, e := range sorted {
		h.Write([]byte(e))
		h.Write([]byte("\n"))
	}
	return hex.EncodeToString(h.Sum(nil))
}
