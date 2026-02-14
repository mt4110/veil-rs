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

// DefaultRunner is the default runner instance.
// It can be replaced by tests.
var Runner ExecRunner = &ProdExecRunner{}

// Init initializes the default runner with the repository root.
func Init(repoRoot string) {
	Runner = &ProdExecRunner{RepoRoot: repoRoot}
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

	r.resolveDir(cmd, spec)
	r.constructEnv(cmd, spec)

	var stdoutBuf, stderrBuf bytes.Buffer
	cmd.Stdout = &stdoutBuf
	cmd.Stderr = &stderrBuf

	err := cmd.Run()

	return r.constructResult(err, ctx, stdoutBuf.Bytes(), stderrBuf.Bytes())
}

func (r *ProdExecRunner) resolveDir(cmd *exec.Cmd, spec ExecSpec) {
	if spec.CwdRel != "" {
		if r.RepoRoot != "" {
			cmd.Dir = r.RepoRoot + "/" + spec.CwdRel
		} else {
			cmd.Dir = spec.CwdRel
		}
	}
}

func (r *ProdExecRunner) constructEnv(cmd *exec.Cmd, spec ExecSpec) {
	if len(spec.EnvKV) > 0 {
		cmd.Env = os.Environ()
		for _, kv := range spec.EnvKV {
			cmd.Env = append(cmd.Env, kv.Key+"="+kv.Value)
		}
	}
}

func (r *ProdExecRunner) constructResult(err error, ctx context.Context, stdout, stderr []byte) ExecResult {
	res := ExecResult{}

	// Capture Output
	res.Stdout, res.TruncatedStdout = normalizeOutput(stdout)
	res.Stderr, res.TruncatedStderr = normalizeOutput(stderr)

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
