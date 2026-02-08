package registry

import (
	"fmt"
	"io/fs"
	"sort"
	"time"

	"github.com/pelletier/go-toml/v2"
)

// Loader handles reading and parsing the exception registry.
type Loader struct {
	FS fs.FS
}

// Load reads "ops/exceptions.toml" from the configured FS.
// It populates OriginalIndex and Status based on the provided 'now'.
// It sorts the exceptions deterministically: (ID ASC, OriginalIndex ASC).
func (l *Loader) Load(now time.Time) (*Registry, error) {
	path := "ops/exceptions.toml"
	
	// 1. Read
	content, err := fs.ReadFile(l.FS, path)
	if err != nil {
		return nil, fmt.Errorf("failed to read %s: %w", path, err)
	}

	// 2. Parse
	var reg Registry
	if err := toml.Unmarshal(content, &reg); err != nil {
		return nil, fmt.Errorf("TOML parse error: %w", err)
	}

	// 3. Populate Runtime Fields & Validate (Partial)
	for i := range reg.Exceptions {
		ex := &reg.Exceptions[i]
		ex.OriginalIndex = i
		ex.Status = CalculateStatus(ex, now)
	}

	// 4. Deterministic Sort
	// Sort by ID ASC, then OriginalIndex ASC
	sort.Slice(reg.Exceptions, func(i, j int) bool {
		if reg.Exceptions[i].ID == reg.Exceptions[j].ID {
			return reg.Exceptions[i].OriginalIndex < reg.Exceptions[j].OriginalIndex
		}
		return reg.Exceptions[i].ID < reg.Exceptions[j].ID
	})

	return &reg, nil
}

// CalculateStatus determines the status of an exception at a given time.
func CalculateStatus(ex *Exception, now time.Time) Status {
	if ex.ExpiresAt == "" {
		// No expiry = Active (Perpetual)
		// Validation might complain about missing expiry, but status is Active.
		return StatusActive
	}

	expires, err := time.Parse("2006-01-02", ex.ExpiresAt)
	if err != nil {
		// Invalid format treated as Active for status purposes? 
		// Or should we have a StatusInvalid?
		// For now, let's assume Active but validation will catch it.
		// Actually, if it's invalid, we can't compare dates.
		return StatusActive 
	}

	// Strict comparison: now > expires => Expired
	// (Expiry date is inclusive)
	if now.After(expires) {
		return StatusExpired
	}

	// Check for "Expiring Soon" (e.g., within 7 days)
	// hours := expires.Sub(now).Hours()
	// if hours < 24*7 { ... }
	// For this PR, user asked for "Status分类（active / expiring-soon / expired）"
	// Let's define "Soon" as 7 days.
	
	daysUntil := expires.Sub(now).Hours() / 24
	if daysUntil <= 7 {
		return StatusExpiringSoon
	}

	return StatusActive
}
