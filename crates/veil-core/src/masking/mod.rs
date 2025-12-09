use veil_config::MaskMode;

pub fn apply_masks(content: &str, ranges: Vec<std::ops::Range<usize>>, mode: MaskMode) -> String {
    if mode == MaskMode::Plain {
        return content.to_string();
    }

    if ranges.is_empty() {
        return content.to_string();
    }

    let mut sorted_ranges = ranges;
    sorted_ranges.sort_by_key(|r| r.start);

    let mut merged = Vec::new();
    let mut current_range = sorted_ranges[0].clone();

    for next in sorted_ranges.into_iter().skip(1) {
        if next.start < current_range.end {
            current_range.end = std::cmp::max(current_range.end, next.end);
        } else {
            merged.push(current_range);
            current_range = next;
        }
    }
    merged.push(current_range);

    let mut result = String::with_capacity(content.len());
    let mut last_pos = 0;

    for range in merged {
        if range.start > last_pos {
            result.push_str(&content[last_pos..range.start]);
        }

        let secret = &content[range.clone()];
        let replacement = match mode {
            MaskMode::Redact => "<REDACTED>".to_string(),
            MaskMode::Partial => {
                // TODO: partial masking currently assumes byte offsets which works for ASCII secrets.
                // For multibyte secrets, this might split characters.
                let char_count = secret.chars().count();
                if char_count <= 4 {
                    "****".to_string()
                } else {
                    let start: String = secret.chars().take(4).collect();
                    let end: String = secret.chars().skip(char_count.saturating_sub(4)).collect();
                    format!("{}...{}", start, end)
                }
            }
            MaskMode::Plain => secret.to_string(), // Should be unreachable with early check
        };

        result.push_str(&replacement);
        last_pos = range.end;
    }

    if last_pos < content.len() {
        result.push_str(&content[last_pos..]);
    }

    result
}

// Deprecated or removed mask_string?
// Scanner uses apply_masks now.

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;
    use veil_config::MaskMode;

    #[test]
    fn test_mask_ranges_simple() {
        let text = "Hello World";
        // Mask "World" (6..11)
        let ranges = vec![6..11; 1];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Redact),
            "Hello <REDACTED>"
        );
    }

    #[test]
    fn test_mask_partial() {
        let text = "AKIA1234567890ABCD";
        // Mask whole thing
        let ranges = vec![0..18];
        assert_eq!(apply_masks(text, ranges, MaskMode::Partial), "AKIA...ABCD");
    }

    #[test]
    fn test_mask_partial_short() {
        let text = "PWD=1234";
        // Mask "1234" (4..8)
        let ranges = vec![4..8];
        assert_eq!(apply_masks(text, ranges, MaskMode::Partial), "PWD=****");
    }

    #[test]
    fn test_mask_ranges_overlapping() {
        // "key=123 secret=456"
        // 012345678901234567
        // key=123: 4..7
        // key=12: 4..6
        // secret=456: 15..18
        let text = "key=123 secret=456";
        let ranges = vec![4..7, 4..6, 15..18];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Redact),
            "key=<REDACTED> secret=<REDACTED>"
        );
    }

    #[test]
    fn test_mask_ranges_nested() {
        let text = "abcdefg";
        // "abcde" (0..5), "bcd" (1..4)
        let ranges = vec![0..5, 1..4];
        assert_eq!(apply_masks(text, ranges, MaskMode::Redact), "<REDACTED>fg");
    }

    #[test]
    fn test_mask_ranges_adjacent() {
        let text = "abcdef";
        // "abc" (0..3), "def" (3..6) -> Should be merged or adjacent?
        // Logic: if next.start < current.end. 3 < 3 is False.
        // So they are separate. "abc" -> REDACTED, "def" -> REDACTED.
        // Result: <REDACTED><REDACTED>
        let ranges = vec![0..3, 3..6];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Redact),
            "<REDACTED><REDACTED>"
        );
    }
}
