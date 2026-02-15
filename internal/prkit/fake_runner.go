package prkit

import (
	"context"
	"fmt"
)

// FakeExecRunner is a fake implementation of ExecRunner for testing.
type FakeExecRunner struct {
	// Handler is a function that determines the result for a given spec.
	Handler func(spec ExecSpec) ExecResult
}

func (f *FakeExecRunner) Run(ctx context.Context, spec ExecSpec) ExecResult {
	// Name/Argv resolution (Same as ProdExecRunner)
	var name string
	var args []string

	if len(spec.Argv) == 0 {
		return ExecResult{
			ExitCode:  -1,
			ErrorKind: "InvalidSpec",
			Error:     fmt.Errorf("ExecSpec.Argv must not be empty"),
		}
	}

	if spec.Name != "" {
		name = spec.Name
		args = spec.Argv
	} else {
		name = spec.Argv[0]
		if len(spec.Argv) > 1 {
			args = spec.Argv[1:]
		}
	}

	var res ExecResult
	if f.Handler != nil {
		res = f.Handler(spec)
	} else {
		res = ExecResult{
			ExitCode:  -1,
			ErrorKind: "spawn",
			Stderr:    "fake runner: no handler defined",
		}
	}

	// Record to global trace so that evidence collection works in tests
	cmdRecord := Command{
		Argv:      append([]string{name}, args...),
		CwdRel:    spec.Dir, // Fake runner just records what was asked
		EnvMode:   "fake",
		EnvKV:     spec.Env,
		EnvHash:   hashEnv(spec.Env),
		ExitCode:  res.ExitCode,
		Stdout:    res.Stdout,
		Stderr:    res.Stderr,
		ErrorKind: res.ErrorKind,
	}

	traceMu.Lock()
	ExecutionTrace = append(ExecutionTrace, cmdRecord)
	traceMu.Unlock()

	return res
}
