package prkit

import (
	"errors"
	"flag"
	"fmt"
	"io"
)

type cliConfig struct {
	dryRun       bool
	runMode      bool
	reviewBundle bool
	outPath      string
	format       string
	sotNew       bool
	epic         string
	slug         string
	release      string
	apply        bool
	help         bool
}

// Run is the canonical entry point for the prkit CLI logic.
func Run(argv []string, stdout, stderr io.Writer, runner ExecRunner) int {
	ResetTrace()

	// Initialize ExecRunner
	if runner != nil {
		Init("", runner)
	}

	// Try to find repo root
	if root, err := FindRepoRoot(); err == nil {
		Init(root, runner)
	}

	conf, fs, err := parseCLIConfig(argv)
	if err != nil {
		if errors.Is(err, flag.ErrHelp) || conf.help {
			printUsage(stderr, fs)
			return 0
		}
		printUsage(stderr, fs)
		return failCLI(err, stdout, stderr)
	}

	if conf.help {
		printUsage(stderr, fs)
		return 0
	}

	if err := validateCLIConfig(conf); err != nil {
		return failCLI(err, stdout, stderr)
	}

	return executeCLI(conf, stdout, stderr)
}

func parseCLIConfig(argv []string) (*cliConfig, *flag.FlagSet, error) {
	conf := &cliConfig{}
	fs := flag.NewFlagSet("prkit", flag.ContinueOnError)
	fs.SetOutput(io.Discard)

	fs.BoolVar(&conf.help, "help", false, "Show help")
	fs.BoolVar(&conf.help, "h", false, "Show help (shorthand)")
	fs.BoolVar(&conf.dryRun, "dry-run", false, "Enable dry-run mode (output only)")
	fs.BoolVar(&conf.runMode, "run", false, "Enable execution mode")
	fs.BoolVar(&conf.reviewBundle, "review-bundle", false, "Generate review bundle (run mode only)")
	fs.StringVar(&conf.outPath, "out", "", "Output path for evidence (run mode only)")
	fs.StringVar(&conf.format, "format", "portable-json", "Output format (default: portable-json)")
	fs.BoolVar(&conf.sotNew, "sot-new", false, "Generate SOT scaffolding")
	fs.StringVar(&conf.epic, "epic", "", "Epic name (for SOT)")
	fs.StringVar(&conf.slug, "slug", "", "PR slug (for SOT)")
	fs.StringVar(&conf.release, "release", "", "Release version (for SOT, optional/autodetect)")
	fs.BoolVar(&conf.apply, "apply", false, "Apply SOT scaffolding (write file)")

	if err := fs.Parse(argv); err != nil {
		return conf, fs, err
	}
	return conf, fs, nil
}

func validateCLIConfig(conf *cliConfig) error {
	if conf.sotNew {
		return validateSOTFlags(conf)
	}

	if err := validateModeExclusivity(conf); err != nil {
		return err
	}

	if err := validateFormatAndPaths(conf); err != nil {
		return err
	}

	return nil
}

func validateSOTFlags(conf *cliConfig) error {
	if conf.dryRun || conf.runMode || conf.reviewBundle || conf.outPath != "" {
		return errors.New("Error: --sot-new cannot be combined with --dry-run, --run, --review-bundle, or --out")
	}
	if conf.epic == "" || conf.slug == "" {
		return errors.New("Error: --sot-new requires --epic and --slug")
	}
	return nil
}

func validateModeExclusivity(conf *cliConfig) error {
	if conf.dryRun && conf.runMode {
		return errors.New("Error: cannot specify both --dry-run and --run")
	}
	if !conf.dryRun && !conf.runMode {
		return errors.New("Error: must specify either --dry-run, --run, or --sot-new")
	}
	return nil
}

func validateFormatAndPaths(conf *cliConfig) error {
	if conf.dryRun {
		if conf.reviewBundle {
			return errors.New("Error: --review-bundle is only supported in --run mode")
		}
	} else {
		if conf.outPath == "" {
			return errors.New("Error: --out is required in --run mode")
		}
	}

	if conf.format != "portable-json" {
		return fmt.Errorf("Error: unsupported format: %s", conf.format)
	}
	return nil
}

func executeCLI(conf *cliConfig, stdout, stderr io.Writer) int {
	if conf.sotNew {
		if err := ScaffoldSOT(stdout, conf.epic, conf.slug, conf.release, conf.apply); err != nil {
			return failCLI(fmt.Errorf("failed to scaffold SOT: %w", err), stdout, stderr)
		}
		return 0
	}

	var exitCode int
	var err error
	if conf.dryRun {
		exitCode, err = RunDryRun(stdout)
	} else {
		exitCode, err = RunExecuteMode(conf.outPath, conf.reviewBundle, stdout)
	}

	if err != nil {
		return failCLI(err, stdout, stderr)
	}
	return exitCode
}

func printUsage(stderr io.Writer, fs *flag.FlagSet) {
	fmt.Fprintf(stderr, "Usage of prkit:\n")
	fs.SetOutput(stderr)
	fs.PrintDefaults()
	fs.SetOutput(io.Discard)
}

func failCLI(err error, stdout, stderr io.Writer) int {
	if err != nil {
		fmt.Fprintln(stderr, err.Error())
	}
	if genErr := GenerateFailureEvidence(err, stdout); genErr != nil {
		fmt.Fprintf(stderr, "failed to generate failure evidence: %v\n", genErr)
	}
	return 2
}
