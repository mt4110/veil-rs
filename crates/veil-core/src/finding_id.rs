use std::fmt;
use std::path::Path;

use blake3::Hasher;
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};

use std::str::FromStr;

/// A deterministic identifier for a Finding.
///
/// FindingId is generated from a set of stable properties:
/// - Rule ID
/// - File path (relative)
/// - Span (start/end lines/columns)
/// - Normalized capture hash
///
/// It explicitly excludes the raw secret content to ensure safety.
/// The string representation is "fx_" + base32_nopad(blake3_hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FindingId([u8; 32]);

impl FromStr for FindingId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("fx_") {
            return Err("invalid finding id prefix (must start with fx_)".to_string());
        }
        
        // We don't necessarily need to decode back to bytes for equality checks if we only store the hash.
        // But for full roundtrip correctness:
        let base32_part = &s[3..].to_uppercase(); // data-encoding expects uppercase for BASE32
        let bytes = BASE32_NOPAD
            .decode(base32_part.as_bytes())
            .map_err(|e| format!("invalid base32 encoding: {}", e))?;
            
        let array: [u8; 32] = bytes.try_into().map_err(|_| "invalid finding id length (must be 32 bytes)".to_string())?;
        
        Ok(Self(array))
    }
}


impl FindingId {
    /// Create a new deterministic FindingId.
    pub fn new(rule_id: &str, path: &Path, span: &SpanData, capture: &str) -> Self {
        let mut hasher = Hasher::new();
        
        // 1. Rule ID
        hasher.update(rule_id.as_bytes());
        hasher.update(b"\0");

        // 2. Path (normalized to string)
        // We assume the caller provides a relative path or canonical path as appropriate for the context.
        // For FindingId stability, it should be relative to repo root.
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update(b"\0");

        // 3. Span (u64 le bytes)
        hasher.update(&span.start_line.to_le_bytes());
        hasher.update(&span.start_col.to_le_bytes());
        hasher.update(&span.end_line.to_le_bytes());
        hasher.update(&span.end_col.to_le_bytes());

        // 4. Capture (normalized and hashed)
        // We trim whitespace to avoid noise from surrounding context if any.
        let capture_hash = blake3::hash(capture.trim().as_bytes());
        hasher.update(capture_hash.as_bytes());

        Self(*hasher.finalize().as_bytes())
    }
}

impl fmt::Display for FindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fx_{}", BASE32_NOPAD.encode(&self.0).to_lowercase())
    }
}

impl Serialize for FindingId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for FindingId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if !s.starts_with("fx_") {
            return Err(serde::de::Error::custom("invalid finding id prefix"));
        }
        
        // We don't necessarily need to decode back to bytes for equality checks if we only store the hash.
        // But for full roundtrip correctness:
        let base32_part = &s[3..].to_uppercase(); // data-encoding expects uppercase for BASE32
        let bytes = BASE32_NOPAD
            .decode(base32_part.as_bytes())
            .map_err(serde::de::Error::custom)?;
            
        let array: [u8; 32] = bytes.try_into().map_err(|_| serde::de::Error::custom("invalid finding id length"))?;
        
        Ok(Self(array))
    }
}

/// Helper struct for Span data needed for ID generation.
/// This avoids coupling to a specific Span type in the rest of the codebase for now,
/// or can be replaced by the actual Span type if it's available and simple.
pub struct SpanData {
    pub start_line: u64,
    pub start_col: u64,
    pub end_line: u64,
    pub end_col: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_finding_id_stability() {
        let rule = "rule-1";
        let path = PathBuf::from("src/main.rs");
        let span = SpanData {
            start_line: 10,
            start_col: 5,
            end_line: 10,
            end_col: 20,
        };
        let capture = "  secret_123  ";

        let id1 = FindingId::new(rule, &path, &span, capture);
        let id2 = FindingId::new(rule, &path, &span, capture);
        
        assert_eq!(id1, id2);
        assert_eq!(id1.to_string(), id2.to_string());
        
        // Ensure format
        let s = id1.to_string();
        assert!(s.starts_with("fx_"));
        // 32 bytes * 8 bits / 5 bits per char = 51.2 -> 52 chars.
        assert_eq!(s.len(), 3 + 52); 
    }

    #[test]
    fn test_finding_id_sensitivity() {
        let base_span = SpanData { start_line: 1, start_col: 1, end_line: 1, end_col: 1 };
        let id1 = FindingId::new("rule1", Path::new("a"), &base_span, "secret");
        
        // Different rule
        let id2 = FindingId::new("rule2", Path::new("a"), &base_span, "secret");
        assert_ne!(id1, id2);

        // Different path
        let id3 = FindingId::new("rule1", Path::new("b"), &base_span, "secret");
        assert_ne!(id1, id3);

        // Different span
        let span2 = SpanData { start_line: 2, ..base_span };
        let id4 = FindingId::new("rule1", Path::new("a"), &span2, "secret"); // struct update syntax not avail for local struct definition in test without implementation, just construct new
        assert_ne!(id1, id4);

        // Different capture (whitespace trimmed match)
        let id5 = FindingId::new("rule1", Path::new("a"), &base_span, "secret ");
        // "secret" and "secret " should match because of trim()
        assert_eq!(id1, id5);

        let id6 = FindingId::new("rule1", Path::new("a"), &base_span, "secret2");
        assert_ne!(id1, id6);
    }

    #[test]
    fn test_serde_roundtrip() {
        let id = FindingId::new("r", Path::new("p"), &SpanData{start_line:1,start_col:1,end_line:1,end_col:1}, "s");
        let json = serde_json::to_string(&id).unwrap();
        let decoded: FindingId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, decoded);
    }
}
