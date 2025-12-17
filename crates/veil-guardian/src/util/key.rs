// use blake3::Hash;

/// Normalizes a key for use as a filename.
///
/// Rules:
/// 1. Allowed characters: `[A-Za-z0-9._-]`. All others are replaced with `_`.
/// 2. If the key contains unsafe characters (i.e. was modified by rule 1), a hash of the *original* key is appended
///    to prevent collisions (e.g. `foo:bar` vs `foo_bar`).
/// 3. If the normalized string exceeds `MAX_LEN` (128), it is truncated to `TRUNC_LEN` (64) and the hash is appended.
///
/// The hash format is `{prefix}-{hash_hex}`.
/// Hash used is BLAKE3 (first 16 hex chars).
pub fn normalize_key(key: &str) -> String {
    const MAX_LEN: usize = 128;
    const TRUNC_LEN: usize = 64;

    let mut safe_str = String::with_capacity(key.len());
    let mut modified = false;

    for c in key.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
            safe_str.push(c);
        } else {
            safe_str.push('_');
            modified = true;
        }
    }

    // append hash if modified to avoid collision (injectivity)
    // or if length exceeds limit
    if modified || safe_str.len() > MAX_LEN {
        // Hash original key
        let hash = blake3::hash(key.as_bytes());
        let hash_hex = hash.to_hex(); // 64 chars
        let short_hash = &hash_hex[..16]; // 16 chars enough for collision resistance in this context

        let prefix_len = if safe_str.len() > TRUNC_LEN {
            TRUNC_LEN
        } else {
            safe_str.len()
        };

        let prefix = &safe_str[..prefix_len];
        format!("{}-{}", prefix, short_hash)
    } else {
        safe_str
    }
}
