use crate::model::FindingSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NormalizedText {
    pub(super) normalized: String,
    index_map: Vec<OriginalSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OriginalSpan {
    normalized_start: usize,
    normalized_end: usize,
    original_start: usize,
    original_end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NormalizationPolicy {
    fullwidth_alnum: bool,
    fullwidth_space: bool,
    hyphen: bool,
    colon: bool,
    parentheses: bool,
}

impl Default for NormalizationPolicy {
    fn default() -> Self {
        Self {
            fullwidth_alnum: true,
            fullwidth_space: true,
            hyphen: true,
            colon: true,
            parentheses: true,
        }
    }
}

impl NormalizedText {
    pub(super) fn original_span(
        &self,
        normalized_start: usize,
        normalized_end: usize,
    ) -> Option<FindingSpan> {
        if normalized_start >= normalized_end || normalized_end > self.normalized.len() {
            return None;
        }

        let first = self.span_containing(normalized_start)?;
        let last = self.span_containing(normalized_end - 1)?;

        Some(FindingSpan {
            byte_start: first.original_start,
            byte_end: last.original_end,
        })
    }

    fn span_containing(&self, normalized_offset: usize) -> Option<OriginalSpan> {
        self.index_map.iter().copied().find(|span| {
            span.normalized_start <= normalized_offset && normalized_offset < span.normalized_end
        })
    }
}

pub(super) fn contains_jp_normalizable_char(input: &str, policy: NormalizationPolicy) -> bool {
    let mut chars = input.chars().peekable();
    let mut previous = None;

    while let Some(ch) = chars.next() {
        let next = chars.peek().copied();
        if normalize_char(ch, policy, previous, next) != ch {
            return true;
        }
        previous = Some(ch);
    }

    false
}

pub(super) fn normalize_jp_text(input: &str, policy: NormalizationPolicy) -> NormalizedText {
    let mut normalized = String::with_capacity(input.len());
    let mut index_map = Vec::new();
    let mut chars = input.char_indices().peekable();
    let mut previous = None;

    while let Some((original_start, ch)) = chars.next() {
        let original_end = original_start + ch.len_utf8();
        let next = chars.peek().map(|(_, ch)| *ch);
        let mapped = normalize_char(ch, policy, previous, next);
        let normalized_start = normalized.len();
        normalized.push(mapped);
        let normalized_end = normalized.len();
        previous = Some(ch);

        index_map.push(OriginalSpan {
            normalized_start,
            normalized_end,
            original_start,
            original_end,
        });
    }

    NormalizedText {
        normalized,
        index_map,
    }
}

fn normalize_char(
    ch: char,
    policy: NormalizationPolicy,
    previous: Option<char>,
    next: Option<char>,
) -> char {
    if policy.fullwidth_alnum {
        if let Some(mapped) = normalize_fullwidth_alnum(ch) {
            return mapped;
        }
    }

    if policy.fullwidth_space && ch == '　' {
        return ' ';
    }

    if policy.hyphen && ch == 'ー' {
        if previous.is_some_and(is_digit_for_separator) && next.is_some_and(is_digit_for_separator)
        {
            return '-';
        }
        return ch;
    }

    if policy.hyphen && matches!(ch, '－' | '―' | '‐' | '‑' | '–' | '—') {
        return '-';
    }

    if policy.colon && ch == '：' {
        return ':';
    }

    if policy.parentheses {
        match ch {
            '（' => return '(',
            '）' => return ')',
            _ => {}
        }
    }

    ch
}

fn is_digit_for_separator(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '０'..='９')
}

fn normalize_fullwidth_alnum(ch: char) -> Option<char> {
    match ch {
        '０'..='９' | 'Ａ'..='Ｚ' | 'ａ'..='ｚ' => char::from_u32(ch as u32 - 0xFEE0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_jp_pii_width_and_separators() {
        let input = "個人番号：１２３４－５６７８ー９０１２ （Ａｂ９）";
        let normalized = normalize_jp_text(input, NormalizationPolicy::default());

        assert_eq!(normalized.normalized, "個人番号:1234-5678-9012 (Ab9)");
    }

    #[test]
    fn preserves_choonpu_inside_jp_keywords() {
        let input = "マイナンバー：１２３４－５６７８ー９０１２ パスポート：ＡＢ１２３４５６７";
        let normalized = normalize_jp_text(input, NormalizationPolicy::default());

        assert_eq!(
            normalized.normalized,
            "マイナンバー:1234-5678-9012 パスポート:AB1234567"
        );
    }

    #[test]
    fn detects_when_jp_normalization_is_needed() {
        let policy = NormalizationPolicy::default();

        assert!(!contains_jp_normalizable_char("abc-123", policy));
        assert!(!contains_jp_normalizable_char("マイナンバー", policy));
        assert!(contains_jp_normalizable_char("１２３", policy));
        assert!(contains_jp_normalizable_char("1234ー5678", policy));
        assert!(contains_jp_normalizable_char("個人番号：1234", policy));
    }

    #[test]
    fn maps_normalized_match_back_to_original_bytes() {
        let input = "prefix 個人番号：１２３４－５６７８－９０１２ suffix";
        let normalized = normalize_jp_text(input, NormalizationPolicy::default());
        let normalized_start = normalized.normalized.find("1234-5678-9012").unwrap();
        let normalized_end = normalized_start + "1234-5678-9012".len();

        let span = normalized
            .original_span(normalized_start, normalized_end)
            .unwrap();

        assert_eq!(
            &input[span.byte_start..span.byte_end],
            "１２３４－５６７８－９０１２"
        );
    }

    #[test]
    fn rejects_empty_or_out_of_bounds_mapping() {
        let normalized = normalize_jp_text("abc", NormalizationPolicy::default());

        assert!(normalized.original_span(1, 1).is_none());
        assert!(normalized.original_span(0, 4).is_none());
    }
}
