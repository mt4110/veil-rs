package main

import (
	"errors"
	"fmt"
	"io/fs"
	"strings"
	"time"

	"veil-rs/internal/registry"
)

// validateRegistryFile serves as the entry point for the registry check in validateDrift
func validateRegistryFile(repoFS fs.FS, utcToday time.Time) error {
	path := "ops/exceptions.toml"

	// 1. Check existence
	_, err := fs.Stat(repoFS, path)
	if err != nil {
		if errors.Is(err, fs.ErrNotExist) {
			return &driftError{
				category: "Registry",
				reason:   "ops/exceptions.toml is missing",
				fixCmd:   "mkdir -p ops && touch ops/exceptions.toml",
				nextCmd:  "nix run .#veil -- exceptions list",
			}
		}
		return &driftError{
			category: "Registry",
			reason:   fmt.Sprintf("failed to stat %s: %v", path, err),
			fixCmd:   "Ensure registry file is accessible.",
			nextCmd:  "nix run .#veil -- exceptions list",
		}
	}

	// 2. Load & Validate using shared logic
	loader := &registry.Loader{FS: repoFS}
	reg, err := loader.Load(utcToday)
	if err != nil {
		return &driftError{
			category: "Registry",
			reason:   fmt.Sprintf("Failed to load registry: %v", err),
			fixCmd:   fmt.Sprintf("Fix syntax errors in %s", path),
			nextCmd:  "nix run .#veil -- exceptions list",
		}
	}

	// 3. Validate
	if errs := registry.Validate(reg, utcToday); len(errs) > 0 {
		count := len(errs)
		maxShow := 10
		var sb strings.Builder
		sb.WriteString(fmt.Sprintf("Registry validation failed (%d errors): (utc_today=%s)", count, utcToday.Format("2006-01-02")))

		for i, e := range errs {
			if i == 0 {
				sb.WriteString("\n")
			}
			if i >= maxShow {
				sb.WriteString(fmt.Sprintf("- ... and %d more", count-maxShow))
				break
			}
			sb.WriteString(fmt.Sprintf("- %s\n", e.Error()))
		}
		
		isExpiryError := false
		for _, e := range errs {
			if strings.Contains(e.Error(), "expired") || strings.Contains(e.Error(), "status=expired") {
				isExpiryError = true
				break
			}
		}
		
		fixCmd := fmt.Sprintf("Correct the invalid entries in %s", path)
		nextCmd := "nix run .#veil -- exceptions list"
		
		if isExpiryError {
			fixCmd = "Renew expiry or remove exception (see runbook: docs/runbook/exception-registry.md)"
			nextCmd = "nix run .#veil -- exceptions list --status expired"
		}

		return &driftError{
			category: "Registry",
			reason:   sb.String(), 
			fixCmd:   fixCmd,
			nextCmd:  nextCmd,
		}
	}

	return nil
}
