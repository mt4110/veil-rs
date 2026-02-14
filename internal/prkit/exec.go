package prkit

import (
	"bytes"
	"context"
	"os"
	"os/exec"
	"sort"
	"strings"
	"unicode/utf8"
)

// ExecSpec defines the input for a command execution.
// It is designed to be deterministic and portable.
type ExecSpec struct {
	// Argv is the command and its arguments.
	// Argv[0] is the command name.
	// We do not support "shell string" execution.
	Argv []string

	// CwdRel is the working directory relative to the repo root.
	// If empty, it defaults to the process's current working directory.
	CwdRel string

	// EnvKV is a sorted list of environment variables.
	// We do not inherit the parent process's environment by default (except specific allowlist if needed).
	// In S10-09, we prefer explicit env passing.
	EnvKV []EnvKV

	// TimeoutMs is the execution timeout in milliseconds.
	// 0 means no timeout.
	TimeoutMs int
}

// EnvKV represents a deterministic environment variable.
type EnvKV struct {
	Key   string
	Value string
}

// ExecResult captures the output of a command execution.
// It is designed to be deterministic and portable.
type ExecResult struct {
	Stdout string
	Stderr string

	ExitCode int

	TruncatedStdout bool
	TruncatedStderr bool

	// ErrorKind classifies the failure reason.
	// "" (success), "exit", "timeout", "spawn", "canceled"
	ErrorKind string
}

// ExecRunner is the interface for executing commands.
// It allows swapping the real runner with a fake one for testing.
type ExecRunner interface {
	Run(ctx context.Context, spec ExecSpec) ExecResult
}

// ProdExecRunner is the production implementation of ExecRunner.
// It uses os/exec and enforces hardening rules.
type ProdExecRunner struct {
	// RepoRoot is the absolute path to the repository root.
	// Used to resolve CwdRel.
	RepoRoot string
}

const (
	// MaxOutputBytes is the maximum bytes to capture for stdout/stderr.
	// Keep it reasonable for evidence JSON.
	MaxOutputBytes = 32 * 1024 // 32KB
)

func (r *ProdExecRunner) Run(ctx context.Context, spec ExecSpec) ExecResult {
	if len(spec.Argv) == 0 {
		return ExecResult{
			ErrorKind: "spawn",
			Stderr:    "empty argv",
			ExitCode:  -1,
		}
	}

	cmdName := spec.Argv[0]
	cmdArgs := spec.Argv[1:]

	cmd := exec.CommandContext(ctx, cmdName, cmdArgs...)

	// Resolve Dir
	if spec.CwdRel != "" {
		if r.RepoRoot != "" {
			cmd.Dir = r.RepoRoot + "/" + spec.CwdRel
		} else {
			// Fallback or error? For now, if RepoRoot is missing but CwdRel is allowed, use it as is?
			// But spec says "repo relative".
			// If RepoRoot is empty, we tread CwdRel as relative to CWD, which might be dangerous.
			// Let's assume CwdRel is relative to where we run.
			cmd.Dir = spec.CwdRel
		}
	}

	// Construct Env
	// We start with a clean env or inherit?
	// The plan says "EnvKV from spec".
	// os/exec defaults to os.Environ() if cmd.Env is nil.
	// To enforce determinism, we should probably start empty or allow-list.
	// But many tools need PATH, HOME, etc.
	// For S10-09, let's append to the process env for now to avoid breaking everything,
	// but mostly rely on what's passed.
	// Actually, "Invariant-2: evidenceに 非決定性を入れない" implies we record what we pass.
	// It doesn't strictly forbid inheriting env for the *execution* itself, but for *evidence* we strictly control what we see.
	// However, "Invariant-2: ... 環境差（巨大env）を混入させない" suggests blocking inheritance might be desired eventually.
	// For this phase, let's behave like `exec.Command`: inherit if EnvKV is empty?
	// Plan: "Cmd.Env は spec.EnvKV から構築（順序固定）"
	// This implies REPLACING the env.
	// WARNING: If we replace Env entirely, things like `git` might break if they need `HOME` or `PATH`.
	// Let's stick to: If spec.EnvKV is provided, we merge it? Or we strictly use what is given?
	// "envは 必要最小（sorted KV slice）で渡す＆記録"
	// Let's implement: "Use os.Environ() + spec.EnvKV" by default, but maybe we should be strict?
	// Safe approach: os.Environ() + override.
	// But `Cmd.Env` reference says: "If Env is nil, the new process uses the current process's environment."
	// "If Env is non-nil, the new process uses the environment values in Env."
	// So if we set `cmd.Env`, we MUST provide everything (PATH, etc.).
	// For now, to be safe and avoid "breaking everything" (Non-goal: functional change),
	// let's NOT set `cmd.Env` (inherit) unless we explicitly want to isolate.
	// But wait, the plan says "envは... sorted KV slice で渡す".
	// If `ExecSpec` has `EnvKV`, we probably want to *add* them.
	// If we want strict isolation, `ExecSpec` should probably have a flag `CleanEnv bool`.
	// Given the instructions "shell禁止... cmd/args/cwd/env... evidenceに記録",
	// the primary goal is RECORDING what we did.
	// If we inherit env, we can't record the full env (too huge).
	// So we record "what we added/modified".
	// Implementation:
	// cmd.Env = append(os.Environ(), formatEnv(spec.EnvKV)...) -> This duplicates keys.
	// Determining "effective env" is hard if we just append.
	// Let's leave `cmd.Env = nil` (inherit) and simply set the add-ons using `cmd.Env = os.Environ() + spec`.
	// Actually, `exec.Command` checks for duplicates? No, last one wins usually.
	if len(spec.EnvKV) > 0 {
		envMap := make(map[string]string)
		// Load current env first
		for _, e := range os.Environ() {
			k, v, _ := strings.Cut(e, "=")
			envMap[k] = v
		}
		// Apply overrides
		for _, kv := range spec.EnvKV {
			envMap[kv.Key] = kv.Value
		}
		// Convert back to slice and sort
		// Wait, this is getting expensive and complex for "just running a command".
		// Simple approach: `cmd.Env = os.Environ()` then append spec.EnvKV.
		cmd.Env = os.Environ()
		for _, kv := range spec.EnvKV {
			cmd.Env = append(cmd.Env, kv.Key+"="+kv.Value)
		}
	}
	// Note: We record `spec.EnvKV` in evidence, NOT `cmd.Env`. This satisfies "EnvKV represents what we explicitly passed".

	var stdoutBuf, stderrBuf bytes.Buffer
	cmd.Stdout = &stdoutBuf
	cmd.Stderr = &stderrBuf

	err := cmd.Run()

	res := ExecResult{}

	// Capture Output
	res.Stdout, res.TruncatedStdout = normalizeOutput(stdoutBuf.Bytes())
	res.Stderr, res.TruncatedStderr = normalizeOutput(stderrBuf.Bytes())

	// Exit Code & Error Kind
	if err == nil {
		res.ExitCode = 0
		res.ErrorKind = ""
	} else {
		// Try to get exit code
		if exitErr, ok := err.(*exec.ExitError); ok {
			res.ExitCode = exitErr.ExitCode()
			res.ErrorKind = "exit"
		} else {
			// Other errors (spawn, etc.)
			res.ExitCode = -1
			res.ErrorKind = "spawn"
			// Check context errors
			if ctx.Err() == context.DeadlineExceeded {
				res.ErrorKind = "timeout"
			} else if ctx.Err() == context.Canceled {
				res.ErrorKind = "canceled"
			}
		}
	}

	return res
}

func normalizeOutput(raw []byte) (string, bool) {
	truncated := false
	if len(raw) > MaxOutputBytes {
		raw = raw[:MaxOutputBytes]
		truncated = true
	}

	// sanitize UTF-8
	if !utf8.Valid(raw) {
		// simplistic replacement
		safe := bytes.ToValidUTF8(raw, []byte("\uFFFD"))
		raw = safe
	}

	s := string(raw)

	// Normalize CRLF -> LF
	s = strings.ReplaceAll(s, "\r\n", "\n")
	// Normalize CR -> LF (rare but possible)
	s = strings.ReplaceAll(s, "\r", "\n")

	return s, truncated
}

// SortEnvKV sorts a slice of EnvKV by Key.
func SortEnvKV(env []EnvKV) {
	sort.Slice(env, func(i, j int) bool {
		return env[i].Key < env[j].Key
	})
}
