package registry

// Status represents the state of an exception entry relative to a reference time.
type Status string

const (
	StatusActive       Status = "active"
	StatusExpiringSoon Status = "expiring_soon" // e.g. within 7 days
	StatusExpired      Status = "expired"
)

// Exception represents a single entry in the registry.
// It mirrors the TOML structure but adds runtime status.
type Exception struct {
	ID            string   `toml:"id"`
	Rule          string   `toml:"rule"`
	Scope         string   `toml:"scope"`
	Reason        string   `toml:"reason"`
	Owner         string   `toml:"owner"`
	CreatedAt     string   `toml:"created_at"`
	Audit         []string `toml:"audit"`
	ExpiresAt     string   `toml:"expires_at,omitempty"`
	
	// Runtime fields (populated by Loader)
	OriginalIndex int      `toml:"-"`
	Status        Status   `toml:"-"`
}

// Registry represents the top-level structure of ops/exceptions.toml.
type Registry struct {
	Exceptions []Exception `toml:"exception"`
}
