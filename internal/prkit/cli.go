package prkit

import (
	"io"
)

// Run is the canonical entry point for the prkit CLI logic.
// It handles initialization, flag parsing (to be moved in C4), and orchestration.
func Run(argv []string, stdout, stderr io.Writer, runner ExecRunner) int {
	// For now, this is just a placeholder signature to define the contract.
	// Implementation will be filled in during C2/C3/C4.
	return 0
}
