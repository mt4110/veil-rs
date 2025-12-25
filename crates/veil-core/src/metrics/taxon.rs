use std::fmt;

/// Represents a taxonomic classification for an event.
/// Format: `domain.key=value` (ASCII).
/// Example: `http.status=429`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Taxon(String);

impl Taxon {
    /// Constructs a new `Taxon` from the given components.
    ///
    /// This function does **not** perform format validation.
    /// Callers (e.g., parsing logic) are responsible for ensuring the result matches the schema pattern:
    /// `^[a-zA-Z0-9_]+\.[a-zA-Z0-9_]+=[a-zA-Z0-9_\-\.]+$`.
    pub fn new(domain: &str, key: &str, value: &str) -> Self {
        // Simple formatting
        // "ASCII, domain.key=value"
        Taxon(format!("{}.{}={}", domain, key, value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Taxon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Predefined taxons helper (v1 examples)
pub mod v1 {
    use super::Taxon;

    // HTTP
    pub fn http_status(code: u16) -> Taxon {
        Taxon::new("http", "status", &code.to_string())
    }

    // Net
    pub fn net_kind(kind: &str) -> Taxon {
        Taxon::new("net", "kind", kind)
    }

    // Cache
    pub fn cache_kind(kind: &str) -> Taxon {
        Taxon::new("cache", "kind", kind)
    }

    // Lock
    pub fn lock_kind(kind: &str) -> Taxon {
        Taxon::new("lock", "kind", kind)
    }

    // OSV
    pub fn osv_stage(stage: &str) -> Taxon {
        Taxon::new("osv", "stage", stage)
    }
}
