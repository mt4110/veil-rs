package main

import (
	"fmt"
	"os"
)

// ---- Main Dispatch ----

func main() { os.Exit(realMain(os.Args[1:])) }

func realMain(args []string) int {
	if len(args) == 0 {
		printRootUsage()
		return E_USAGE
	}
	cmd := args[0]
	subArgs := args[1:]

	switch cmd {
	case "gen":
		return cmdGen(subArgs)
	case "check":
		return cmdCheck(subArgs)
	case "status":
		return cmdStatus(subArgs)
	case "-h", "--help", "help":
		printRootUsage()
		return E_OK
	default:
		return die(E_USAGE, "Unknown command", fmt.Sprintf("unknown command: %q", cmd), "Use: veil-aiux help")
	}
}

func printRootUsage() {
	infof("veil-aiux — Cockpit (Go + Nix)")
	infof("")
	infof("Usage:")
	infof("  veil-aiux <command> [args]")
	infof("")
	infof("Commands:")
	infof("  gen     Generate dist artifacts (single entry)")
	infof("  check   Guardrails (generic or versioned)")
	infof("  status  Read-only dashboard for dist")
	infof("")
	infof("Examples:")
	infof("  veil-aiux gen --version v0.14.0 --clean --base-ref origin/main")
	infof("  veil-aiux check")
	infof("  veil-aiux check --version v0.14.0")
	infof("  veil-aiux status --version v0.14.0")
}
