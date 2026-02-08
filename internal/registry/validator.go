package registry

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

// Validate checks the schema and constraints of the registry.
// It returns a list of errors to ensure deterministic reporting.
func Validate(reg *Registry, utcToday time.Time) []error {
	var errs []error
	seenIDs := make(map[string]bool)

	// Sort exceptions by ID for deterministic validation order
	// Fallback to OriginalIndex used to preserve logical order of file for missing/duplicate IDs
	sort.Slice(reg.Exceptions, func(i, j int) bool {
		if reg.Exceptions[i].ID == reg.Exceptions[j].ID {
			return reg.Exceptions[i].OriginalIndex < reg.Exceptions[j].OriginalIndex
		}
		return reg.Exceptions[i].ID < reg.Exceptions[j].ID
	})

	for _, ex := range reg.Exceptions {
		// Stable key for error reporting
		key := ex.ID
		if key == "" {
			key = fmt.Sprintf("idx %d", ex.OriginalIndex)
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
			_, err := time.Parse("2006-01-02", ex.ExpiresAt)
			if err != nil {
				errs = append(errs, fmt.Errorf("%s: invalid expires_at '%s' (must be YYYY-MM-DD)", key, ex.ExpiresAt))
			} else {
				// Status Check (instead of raw date comparison)
				// We assume Status is populated by Loader.
				// If not populated (e.g. manual struct creation), CalculateStatus must be called.
				// Loader populates it.
				
				// Expiry Rule:
				// If missing expires_at -> Allowed (StatusActive acts as perpetual)
				// If present checks status.
				
				// Using CalculateStatus ensures consistency with "Status" type
				status := CalculateStatus(&ex, utcToday) 
				if status == StatusExpired {
					// UX Requirement: "id=<ID> expires=<DATE> now=<DATE> status=expired"
					// But Validate returns "error". We need to format the error string to be useful for consumers?
					// Or consumers format it?
					// Validate returns standard errors.
					// For "Expiry Enforcement Failure", we might want a specific error type or structured error?
					// But Validate returns []error.
					// Let's stick to a descriptive error string for now, and prverify can format the Drift Error.
					// Wait, prverify takes these errors and puts them in "Reason".
					// The "Reason" needs to be 1-scroll.
					// If we have 100 expired items, we list them.
					// Maybe each error line uses the format?
					
					// "EX-01: expired (expires=2026-01-01, now=2026-02-08, status=expired)"
					errs = append(errs, fmt.Errorf("%s: expired (expires=%s, now=%s, status=%s)", key, ex.ExpiresAt, utcToday.Format("2006-01-02"), status))
				}
			}
		}
	}

	return errs
}
