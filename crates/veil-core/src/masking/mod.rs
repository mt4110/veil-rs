pub fn mask_string(content: &str, range: std::ops::Range<usize>) -> String {
    let mut result = content.to_string();
    result.replace_range(range, "<REDACTED>");
    result
}

pub fn mask_ranges(content: &str, ranges: Vec<std::ops::Range<usize>>) -> String {
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
        result.push_str("<REDACTED>");
        last_pos = range.end;
    }

    if last_pos < content.len() {
        result.push_str(&content[last_pos..]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_ranges_simple() {
        let text = "Hello World";
        // Mask "World" (6..11)
        let ranges = vec![6..11; 1];
        assert_eq!(mask_ranges(text, ranges), "Hello <REDACTED>");
    }

    #[test]
    fn test_mask_ranges_multiple() {
        let text = "key=123 secret=456";
        // Mask "123" (4..7) and "456" (15..18)
        let ranges = vec![4..7, 15..18];
        assert_eq!(
            mask_ranges(text, ranges),
            "key=<REDACTED> secret=<REDACTED>"
        );
    }

    #[test]
    fn test_mask_ranges_overlapping() {
        let text = "abcdefg";
        // "abc" (0..3), "bcd" (1..4) -> Union is "abcd" (0..4)
        let ranges = vec![0..3, 1..4];
        assert_eq!(mask_ranges(text, ranges), "<REDACTED>efg");
    }

    #[test]
    fn test_mask_ranges_nested() {
        let text = "abcdefg";
        // "abcde" (0..5), "bcd" (1..4)
        let ranges = vec![0..5, 1..4];
        assert_eq!(mask_ranges(text, ranges), "<REDACTED>fg");
    }

    #[test]
    fn test_mask_ranges_adjacent() {
        let text = "abcdef";
        // "abc" (0..3), "def" (3..6) -> Should be merged or adjacent?
        // Logic: if next.start < current.end. 3 < 3 is False.
        // So they are separate. "abc" -> REDACTED, "def" -> REDACTED.
        // Result: <REDACTED><REDACTED>
        let ranges = vec![0..3, 3..6];
        assert_eq!(mask_ranges(text, ranges), "<REDACTED><REDACTED>");
    }
}
