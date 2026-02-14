package prkit

import "context"

// FakeExecRunner is a fake implementation of ExecRunner for testing.
type FakeExecRunner struct {
	// Handler is a function that determines the result for a given spec.
	Handler func(spec ExecSpec) ExecResult
}

func (f *FakeExecRunner) Run(ctx context.Context, spec ExecSpec) ExecResult {
	if f.Handler != nil {
		return f.Handler(spec)
	}
	return ExecResult{
		ExitCode:  -1,
		ErrorKind: "spawn",
		Stderr:    "fake runner: no handler defined",
	}
}
