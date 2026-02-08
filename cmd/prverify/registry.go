package main

import (
	"errors"
	"fmt"
	"io/fs"
	"sort"
	"strings"
	"time"

	"github.com/pelletier/go-toml/v2"
)

// Exception represents a single entry in the registry
type Exception struct {
	ID        string   `toml:"id"`
	Rule      string   `toml:"rule"`
	Scope     string   `toml:"scope"`
	Reason    string   `toml:"reason"`
	Owner     string   `toml:"owner"`
	CreatedAt string   `toml:"created_at"`
	Audit     []string `toml:"audit"`
	ExpiresAt string   `toml:"expires_at,omitempty"`
}

// Registry represents the top-level structure of ops/exceptions.toml
type Registry struct {
	Exceptions []Exception `toml:"exception"`
}

// validateRegistryFile serves as the entry point for the registry check in validateDrift
func validateRegistryFile(repoFS fs.FS) error {
	path := "ops/exceptions.toml"

	// 1. Check existence
	_, err := fs.Stat(repoFS, path)
	if err != nil {
		if errors.Is(err, fs.ErrNotExist) {
			return &driftError{
				category: "Registry",
				reason:   "ops/exceptions.toml is missing",
				fixCmd:   "touch ops/exceptions.toml",
			}
		}
		return &driftError{
			category: "Registry",
			reason:   fmt.Sprintf("failed to stat %s: %v", path, err),
			fixCmd:   "Ensure registry file is readable.",
		}
	}

	// 2. Read
	content, err := fs.ReadFile(repoFS, path)
	if err != nil {
		return &driftError{
			category: "Registry",
			reason:   fmt.Sprintf("failed to read %s: %v", path, err),
			fixCmd:   "Ensure registry file is readable.",
		}
	}

	// 3. Parse
	var reg Registry
	if err := toml.Unmarshal(content, &reg); err != nil {
		return &driftError{
			category: "Registry",
			reason:   fmt.Sprintf("TOML parse error: %v", err),
			fixCmd:   fmt.Sprintf("edit %s", path),
		}
	}

	// 4. Validate
	if errs := validateRegistry(&reg); len(errs) > 0 {
		count := len(errs)
		maxShow := 10
		var sb strings.Builder
		sb.WriteString(fmt.Sprintf("Registry validation failed (%d errors): ", count))

		for i, e := range errs {
			if i >= maxShow {
				sb.WriteString(fmt.Sprintf("... and %d more", count-maxShow))
				break
			}
			if i > 0 {
				sb.WriteString("; ")
			}
			sb.WriteString(e.Error())
		}

		return &driftError{
			category: "Registry",
			reason:   sb.String(), // Single-line formatted reason
			fixCmd:   fmt.Sprintf("Correct the invalid entries in %s", path),
		}
	}

	return nil
}

// validateRegistry checks the schema and constraints of the registry
// It returns a list of errors to ensure deterministic reporting of all issues
func validateRegistry(reg *Registry) []error {
	var errs []error
	seenIDs := make(map[string]bool)

	// Sort exceptions by ID for deterministic validation order
	sort.SliceStable(reg.Exceptions, func(i, j int) bool {
		return reg.Exceptions[i].ID < reg.Exceptions[j].ID
	})

	for i, ex := range reg.Exceptions {
		// Stable key for error reporting
		key := ex.ID
		if key == "" {
			key = fmt.Sprintf("idx %d", i)
		}

		// ID Uniqueness & Presence
		if ex.ID == "" {
			errs = append(errs, fmt.Errorf("%s: missing id", key))
		} else {
			if seenIDs[ex.ID] {
				errs = append(errs, fmt.Errorf("duplicate id: %s", ex.ID))
			}
			seenIDs[ex.ID] = true
		}

		// Mandatory fields
		if ex.Rule == "" {
			errs = append(errs, fmt.Errorf("%s: missing rule", key))
		}
		if ex.Scope == "" {
			errs = append(errs, fmt.Errorf("%s: missing scope", key))
		}
		if ex.Reason == "" {
			errs = append(errs, fmt.Errorf("%s: missing reason", key))
		}
		if ex.Owner == "" {
			errs = append(errs, fmt.Errorf("%s: missing owner", key))
		}
		if ex.CreatedAt == "" {
			errs = append(errs, fmt.Errorf("%s: missing created_at", key))
		}
		if len(ex.Audit) == 0 {
			errs = append(errs, fmt.Errorf("%s: missing audit trail", key))
		}

		// Scope Grammar
		if ex.Scope != "" {
			if !strings.HasPrefix(ex.Scope, "path:") && !strings.HasPrefix(ex.Scope, "fingerprint:") {
				errs = append(errs, fmt.Errorf("%s: invalid scope format '%s' (must start with path: or fingerprint:)", key, ex.Scope))
			}
		}

		// Date Formats
		if ex.CreatedAt != "" {
			if _, err := time.Parse("2006-01-02", ex.CreatedAt); err != nil {
				errs = append(errs, fmt.Errorf("%s: invalid created_at '%s' (must be YYYY-MM-DD)", key, ex.CreatedAt))
			}
		}
		if ex.ExpiresAt != "" {
			if _, err := time.Parse("2006-01-02", ex.ExpiresAt); err != nil {
				errs = append(errs, fmt.Errorf("%s: invalid expires_at '%s' (must be YYYY-MM-DD)", key, ex.ExpiresAt))
			}
		}
	}

	return errs
}
