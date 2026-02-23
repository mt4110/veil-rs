package prkit

import (
	"context"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
	"time"

	"veil-rs/internal/prkit/cirepro"
)

// runCIRepro handles "prkit ci-repro ..." subcommand.
// Always returns 0 (stopless). Truth lives in logs + summary.
func runCIRepro(argv []string, stdout, stderr io.Writer, runner ExecRunner) int {
	if len(argv) == 0 {
		fmt.Fprintln(stderr, "ERROR: ci-repro requires subcommand: run|step")
		fmt.Fprintln(stderr, "OK: usage: prkit ci-repro run [--out-dir DIR] [--run-id ID] [--with-strict]")
		fmt.Fprintln(stderr, "OK: usage: prkit ci-repro step <go-test|prverify|strict-create|strict-verify> [flags]")
		return 0
	}

	sub := argv[0]
	rest := argv[1:]

	deps := cirepro.Deps{
		NowUTC:          func() time.Time { return Now().UTC() },
		Getenv:          os.Getenv,
		MkdirAll:        os.MkdirAll,
		WriteFileAtomic: cirepro.WriteFileAtomic,
		ProbeGit: func() cirepro.GitInfo {
			gi := cirepro.GitInfo{TreeStatus: "UNKNOWN"}
			if root, err := FindRepoRoot(); err == nil {
				gi.RepoRoot = root
			}
			sha, err := getGitSHA()
			if err != nil {
				fmt.Fprintf(stderr, "OK: gitprobe helper getGitSHA failed: %v\n", err)
				return cirepro.DefaultGitProbe()
			}
			gi.SHA = sha
			if len(sha) >= 7 {
				gi.SHA7 = sha[:7]
			}
			cleanRes := checkGitCleanWorktree()
			if cleanRes.Status == "PASS" {
				gi.TreeStatus = "CLEAN"
			} else if cleanRes.Status == "ERROR" {
				fmt.Fprintf(stderr, "OK: gitprobe helper checkGitCleanWorktree failed: %s\n", cleanRes.Details)
				gi.TreeStatus = "UNKNOWN"
			} else {
				gi.TreeStatus = "DIRTY"
			}
			return gi
		},
		RunToLog: func(runArgv []string, logFile string) (int, error) {
			if len(runArgv) == 0 {
				return 1, fmt.Errorf("empty argv")
			}
			res := runner.Run(context.Background(), ExecSpec{Argv: runArgv})
			b := &strings.Builder{}
			b.WriteString("cmd: " + strings.Join(runArgv, " ") + "\n")
			if res.Stdout != "" {
				b.WriteString(res.Stdout + "\n")
			}
			if res.Stderr != "" {
				b.WriteString(res.Stderr + "\n")
			}
			if res.Error != nil || res.ExitCode != 0 {
				if res.ExitCode != 0 {
					fmt.Fprintf(b, "exit code: %d\n", res.ExitCode)
				}
				if res.Error != nil {
					fmt.Fprintf(b, "error: %v\n", res.Error)
				}
				if res.ErrorKind != "" {
					fmt.Fprintf(b, "error kind: %s\n", res.ErrorKind)
				}
			}
			_ = cirepro.WriteFileAtomic(logFile, []byte(b.String()), 0o644)
			return res.ExitCode, res.Error
		},
	}

	switch sub {
	case "run":
		cfg := parseCIReproFlags(rest, stderr)
		cfg.Command = append([]string{"ci-repro", "run"}, rest...)
		res := cirepro.Run(cfg, deps, stdout, stderr)
		printCIReproResult(res, stdout, stderr)
		return 0

	case "step":
		if len(rest) == 0 {
			fmt.Fprintln(stderr, "ERROR: ci-repro step requires step name")
			return 0
		}
		stepName := rest[0]
		flagsArgv := rest[1:]
		cfg := parseCIReproFlags(flagsArgv, stderr)
		cfg.Command = append([]string{"ci-repro", "step", stepName}, flagsArgv...)
		res := cirepro.RunStep(cfg, deps, stepName, stdout, stderr)
		printCIReproResult(res, stdout, stderr)
		return 0

	default:
		fmt.Fprintf(stderr, "ERROR: unknown ci-repro subcommand: %s\n", sub)
		return 0
	}
}

// parseCIReproFlags parses ci-repro flags with flag.ContinueOnError.
// NEVER calls os.Exit (contract).
func parseCIReproFlags(argv []string, stderr io.Writer) cirepro.Config {
	fs := flag.NewFlagSet("ci-repro", flag.ContinueOnError)
	fs.SetOutput(stderr)

	outDir := fs.String("out-dir", ".local/obs", "output directory")
	runID := fs.String("run-id", "", "run id (default: <UTC_TS>_<gitsha7>)")
	withStrict := fs.Bool("with-strict", false, "include strict-create/verify in run")

	if err := fs.Parse(argv); err != nil {
		fmt.Fprintf(stderr, "ERROR: flag parse failed: %v\n", err)
		// stopless: continue with defaults
	}

	return cirepro.Config{
		OutDir:     strings.TrimSpace(*outDir),
		RunID:      strings.TrimSpace(*runID),
		WithStrict: *withStrict,
	}
}

func printCIReproResult(res cirepro.Result, stdout, stderr io.Writer) {
	if res.Overall == "OK" {
		fmt.Fprintf(stdout, "OK: ci-repro overall=OK summary=%s\n", res.SummaryPath)
	} else {
		fmt.Fprintf(stderr, "ERROR: ci-repro overall=ERROR summary=%s\n", res.SummaryPath)
	}
	if res.StatusSnapshotPath != "" {
		fmt.Fprintf(stdout, "OK: status_snapshot=%s\n", res.StatusSnapshotPath)
	}
}
