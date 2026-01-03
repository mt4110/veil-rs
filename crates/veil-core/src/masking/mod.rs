use veil_config::MaskMode;

pub const DEFAULT_PLACEHOLDER: &str = "<REDACTED>";

pub fn apply_masks(
    content: &str,
    ranges: Vec<std::ops::Range<usize>>,
    mode: MaskMode,
    placeholder: &str,
) -> String {
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
        if next.start <= current_range.end {
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
            MaskMode::Redact => placeholder.to_string(),
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
            MaskMode::Plain => secret.to_string(), // Unreachable due to early exit
        };

        result.push_str(&replacement);
        last_pos = range.end;
    }

    if last_pos < content.len() {
        result.push_str(&content[last_pos..]);
    }

    result
}

#[derive(Debug, Clone)]
pub struct MaskSpan {
    pub start: usize,
    pub end: usize,
    pub placeholder: String,
    pub priority: u32,
}

pub fn apply_masks_spans(content: &str, mut spans: Vec<MaskSpan>, mode: MaskMode) -> String {
    if mode == MaskMode::Plain {
        return content.to_string();
    }
    if spans.is_empty() {
        return content.to_string();
    }

    // 1. Sort by start position to identify connected components
    spans.sort_by_key(|s| s.start);

    // 2. Group into connected components
    let mut components: Vec<Vec<MaskSpan>> = Vec::new();
    if !spans.is_empty() {
        let mut current_component = vec![spans[0].clone()];
        let mut current_end = spans[0].end;

        for span in spans.into_iter().skip(1) {
            // Overlap or Abutment means connected
            if span.start <= current_end {
                current_end = std::cmp::max(current_end, span.end);
                current_component.push(span);
            } else {
                components.push(current_component);
                current_component = vec![span.clone()];
                current_end = span.end;
            }
        }
        components.push(current_component);
    }

    // 3. Process components
    let mut final_spans = Vec::new();
    for component in components {
        // Calculate union range
        let union_start = component.iter().map(|s| s.start).min().unwrap();
        let union_end = component.iter().map(|s| s.end).max().unwrap();
        let union_range = union_start..union_end;

        // Determine winner
        // Winner Criteria: 1. Priority DESC, 2. Length DESC, 3. Start ASC
        let winner = component
            .iter()
            .max_by(|a, b| {
                a.priority
                    .cmp(&b.priority)
                    .then_with(|| (a.end - a.start).cmp(&(b.end - b.start)))
                    // Start: Earlier is better (smaller start index wins).
                    // We use b.cmp(a) because max_by selects 'Greater'.
                    // If b.start > a.start, then 'a' starts earlier, so 'a' should win.
                    .then_with(|| b.start.cmp(&a.start))
            })
            .unwrap();

        final_spans.push((union_range, winner.placeholder.clone()));
    }

    // 4. Build Result
    let mut result = String::with_capacity(content.len());
    let mut last_pos = 0;

    for (range, ph) in final_spans {
        if range.start > last_pos {
            result.push_str(&content[last_pos..range.start]);
        }

        let secret_chunk = &content[range.clone()];
        let replacement = match mode {
            MaskMode::Redact => ph,
            MaskMode::Partial => {
                let char_count = secret_chunk.chars().count();
                if char_count <= 4 {
                    "****".to_string()
                } else {
                    let start: String = secret_chunk.chars().take(4).collect();
                    let end: String = secret_chunk
                        .chars()
                        .skip(char_count.saturating_sub(4))
                        .collect();
                    format!("{}...{}", start, end)
                }
            }
            MaskMode::Plain => secret_chunk.to_string(),
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
            apply_masks(text, ranges, MaskMode::Redact, DEFAULT_PLACEHOLDER),
            "Hello <REDACTED>"
        );
    }

    #[test]
    fn test_mask_partial() {
        let text = "AKIA1234567890ABCD";
        // Mask whole thing
        let ranges = vec![0..18];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Partial, DEFAULT_PLACEHOLDER),
            "AKIA...ABCD"
        );
    }

    #[test]
    fn test_mask_partial_short() {
        let text = "PWD=1234";
        // Mask "1234" (4..8)
        let ranges = vec![4..8];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Partial, DEFAULT_PLACEHOLDER),
            "PWD=****"
        );
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
            apply_masks(text, ranges, MaskMode::Redact, DEFAULT_PLACEHOLDER),
            "key=<REDACTED> secret=<REDACTED>"
        );
    }

    #[test]
    fn test_mask_ranges_nested() {
        let text = "abcdefg";
        // "abcde" (0..5), "bcd" (1..4)
        let ranges = vec![0..5, 1..4];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Redact, DEFAULT_PLACEHOLDER),
            "<REDACTED>fg"
        );
    }

    #[test]
    fn test_mask_ranges_adjacent() {
        let text = "abcdef";
        // "abc" (0..3), "def" (3..6) -> Should be merged or adjacent?
        // Logic: if next.start <= current.end. 3 <= 3 is True.
        // So they are connected. Union is 0..6.
        // Result: <REDACTED> (merged).
        let ranges = vec![0..3, 3..6];
        assert_eq!(
            apply_masks(text, ranges, MaskMode::Redact, DEFAULT_PLACEHOLDER),
            "<REDACTED>"
        );
    }

    #[test]
    fn test_mask_custom_placeholder() {
        let text = "abc123xyz";
        let ranges = vec![3..6]; // "123"
        let masked = apply_masks(text, ranges, MaskMode::Redact, "****");
        assert_eq!(masked, "abc****xyz");
    }

    #[test]
    fn test_plain_mode_returns_original() {
        let text = "secret: 12345";
        let ranges = vec![8..13];
        // Even with a placeholder provided, Plain mode must return original
        let masked = apply_masks(text, ranges, MaskMode::Plain, "****");
        assert_eq!(masked, text);
    }

    #[test]
    fn test_mask_spans_priority_overlap() {
        // "abcdef"
        // rule 1(low, PII): "abcde" (0..5), placeholder="<PII>" (pri 1)
        // rule 2(high, Secret): "bcd" (1..4), placeholder="<SECRET>" (pri 10)
        // Overlap -> Union 0..5 ("abcde").
        // Winner: Rule 2 (Priority 10 > 1).
        // Result: <SECRET>f

        let text = "abcdef";
        let spans = vec![
            MaskSpan {
                start: 0,
                end: 5,
                placeholder: "<PII>".to_string(),
                priority: 1,
            },
            MaskSpan {
                start: 1,
                end: 4,
                placeholder: "<SECRET>".to_string(),
                priority: 10,
            },
        ];

        let result = apply_masks_spans(text, spans, MaskMode::Redact);
        assert_eq!(result, "<SECRET>f");
    }

    #[test]
    fn test_mask_spans_same_priority() {
        // "abcde"
        // rule 1 (0..3) "abc", pri 1, placeholder "<A>"
        // rule 2 (2..5) "cde", pri 1, placeholder "<B>"
        // Overlap -> Union 0..5.
        // Winner?
        // Priority: Equal (1).
        // Length: Equal (3).
        // Start: 0 < 2. Rule 1 wins.
        // Result: "<A>"

        let text = "abcde";
        let spans = vec![
            MaskSpan {
                start: 0,
                end: 3,
                placeholder: "<A>".to_string(),
                priority: 1,
            },
            MaskSpan {
                start: 2,
                end: 5,
                placeholder: "<B>".to_string(),
                priority: 1,
            },
        ];

        let result = apply_masks_spans(text, spans, MaskMode::Redact);
        assert_eq!(result, "<A>");
    }

    #[test]
    fn test_mask_spans_multi_component() {
        // "SECRET PII"
        // 0123456789
        // Secret(0..6) pri=300
        // PII(7..10) pri=200
        // No overlap.
        // Output: <SECRET> <PII>

        let text = "SECRET PII";
        let spans = vec![
            MaskSpan {
                start: 0,
                end: 6,
                placeholder: "<SECRET>".to_string(),
                priority: 300,
            },
            MaskSpan {
                start: 7,
                end: 10,
                placeholder: "<PII>".to_string(),
                priority: 200,
            },
        ];

        let result = apply_masks_spans(text, spans, MaskMode::Redact);
        assert_eq!(result, "<SECRET> <PII>");
    }

    #[test]
    fn test_mask_spans_obs_pii_overlap() {
        // "sentry_dsn=https://foo@sentry.io/123"
        // Range 1 (OBS): "sentry_dsn" (0..10) Priority 30
        // Range 2 (SECRET): "https://foo@sentry.io/123" (11..36) Priority 300
        // No overlap. Two components.

        // But imagine overlap:
        // "SentryDSN: 12345"
        // Rule OBS (0..9) "SentryDSN" Priority 30
        // Rule PII (7..12) "SN: 1" Priority 20 (hypothetical)
        // Union 0..12 ("SentryDSN: 1").
        // Winner OBS (Pri 30 > 20).
        // Result: <OBS>2345

        let text = "SentryDSN: 12345";
        let spans = vec![
            MaskSpan {
                start: 0,
                end: 9,
                placeholder: "<OBS>".to_string(),
                priority: 30,
            },
            MaskSpan {
                start: 7,
                end: 12,
                placeholder: "<PII>".to_string(),
                priority: 20,
            },
        ];

        let result = apply_masks_spans(text, spans, MaskMode::Redact);
        assert_eq!(result, "<OBS>2345"); // 0..12 replaced by <OBS>. Remainder "2345" (index 12..)
    }
}
