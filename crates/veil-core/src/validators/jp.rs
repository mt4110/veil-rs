use crate::scanner::jp_normalize::{normalize_jp_text, NormalizationPolicy};

use super::{digit_value, is_card_separator};

const PREFECTURES: &[&str] = &[
    "北海道",
    "青森県",
    "岩手県",
    "宮城県",
    "秋田県",
    "山形県",
    "福島県",
    "茨城県",
    "栃木県",
    "群馬県",
    "埼玉県",
    "千葉県",
    "東京都",
    "神奈川県",
    "新潟県",
    "富山県",
    "石川県",
    "福井県",
    "山梨県",
    "長野県",
    "岐阜県",
    "静岡県",
    "愛知県",
    "三重県",
    "滋賀県",
    "京都府",
    "大阪府",
    "兵庫県",
    "奈良県",
    "和歌山県",
    "鳥取県",
    "島根県",
    "岡山県",
    "広島県",
    "山口県",
    "徳島県",
    "香川県",
    "愛媛県",
    "高知県",
    "福岡県",
    "佐賀県",
    "長崎県",
    "熊本県",
    "大分県",
    "宮崎県",
    "鹿児島県",
    "沖縄県",
];

pub fn address_prefecture_city_block(candidate: &str) -> bool {
    let normalized = normalize_jp_text(candidate, NormalizationPolicy::default()).normalized;
    let Some((_, prefecture_end)) = find_prefecture(&normalized) else {
        return false;
    };

    let after_prefecture = &normalized[prefecture_end..];
    let municipality_window_end = byte_index_after_chars(after_prefecture, 40);
    let municipality_window = &after_prefecture[..municipality_window_end];
    let Some((municipality_start, municipality_marker)) = municipality_window
        .char_indices()
        .find(|(_, ch)| is_municipality_marker(*ch))
    else {
        return false;
    };
    let after_municipality_start = municipality_start + municipality_marker.len_utf8();
    let after_municipality = &after_prefecture[after_municipality_start..];
    let block_window_end = byte_index_after_chars(after_municipality, 40);
    let address_window = &after_municipality[..block_window_end];

    contains_block_number(address_window)
}

pub fn mynumber_len12(candidate: &str) -> bool {
    candidate
        .chars()
        .chain(std::iter::once('\0'))
        .scan(Vec::new(), |digits, ch| {
            if let Some(digit) = digit_value(ch) {
                digits.push(digit);
                return Some(None);
            }

            if is_mynumber_separator(ch) && !digits.is_empty() {
                return Some(None);
            }

            let len = digits.len();
            digits.clear();
            Some(Some(len))
        })
        .flatten()
        .any(|len| len == 12)
}

pub fn phone_mobile(candidate: &str) -> bool {
    let digits = super::digits_only(candidate);
    digits.len() == 11
        && (digits.starts_with("070") || digits.starts_with("080") || digits.starts_with("090"))
}

fn find_prefecture(candidate: &str) -> Option<(usize, usize)> {
    PREFECTURES
        .iter()
        .filter_map(|prefecture| {
            candidate
                .find(prefecture)
                .map(|start| (start, start + prefecture.len()))
        })
        .min_by_key(|(start, _)| *start)
}

fn contains_block_number(content: &str) -> bool {
    let chars: Vec<char> = content.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if !chars[index].is_ascii_digit() {
            index += 1;
            continue;
        }

        let start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        let end = index;

        if block_marker_after(&chars, end) || block_separator_around(&chars, start, end) {
            return true;
        }
    }

    false
}

fn block_marker_after(chars: &[char], index: usize) -> bool {
    chars
        .get(index)
        .is_some_and(|ch| matches!(ch, '丁' | '番' | '号' | '条'))
}

fn block_separator_around(chars: &[char], start: usize, end: usize) -> bool {
    let previous_hyphen_connects_digits = start
        .checked_sub(1)
        .is_some_and(|hyphen| chars.get(hyphen) == Some(&'-') && digit_before(chars, hyphen));
    let next_hyphen_connects_digits =
        chars.get(end) == Some(&'-') && chars.get(end + 1).is_some_and(|ch| ch.is_ascii_digit());

    previous_hyphen_connects_digits || next_hyphen_connects_digits
}

fn digit_before(chars: &[char], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|previous| chars.get(previous))
        .is_some_and(|ch| ch.is_ascii_digit())
}

fn byte_index_after_chars(content: &str, count: usize) -> usize {
    content
        .char_indices()
        .nth(count)
        .map(|(index, _)| index)
        .unwrap_or(content.len())
}

fn is_municipality_marker(ch: char) -> bool {
    matches!(ch, '市' | '区' | '町' | '村')
}

fn is_mynumber_separator(ch: char) -> bool {
    is_card_separator(ch) || matches!(ch, '－' | '―' | '‐' | '‑' | '–' | '—' | 'ー')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_prefecture_city_block_accepts_normalized_block_numbers() {
        let late_block = format!("東京都{}区丸の内1丁目", "あ".repeat(39));

        assert!(address_prefecture_city_block(
            "住所：東京都千代田区丸の内１－１－１"
        ));
        assert!(address_prefecture_city_block("大阪府大阪市北区梅田1丁目"));
        assert!(address_prefecture_city_block(
            "北海道札幌市中央区北１条西２丁目"
        ));
        assert!(address_prefecture_city_block(&late_block));
    }

    #[test]
    fn address_prefecture_city_block_rejects_labels_or_partial_addresses() {
        let distant_municipality = format!("東京都{}千代田区丸の内1-1-1", "あ".repeat(41));
        let later_prefecture_has_address =
            format!("神奈川県{}東京都千代田区丸の内1-1-1", "あ".repeat(41));

        assert!(!address_prefecture_city_block("住所："));
        assert!(!address_prefecture_city_block("東京都"));
        assert!(!address_prefecture_city_block("東京都千代田区丸の内"));
        assert!(!address_prefecture_city_block("東京都市町村コード: 131016"));
        assert!(!address_prefecture_city_block("東京都千代田区 version 2"));
        assert!(!address_prefecture_city_block("東京都千代田区 version-21"));
        assert!(!address_prefecture_city_block("千代田区丸の内1-1-1"));
        assert!(!address_prefecture_city_block(&distant_municipality));
        assert!(!address_prefecture_city_block(
            &later_prefecture_has_address
        ));
    }

    #[test]
    fn mynumber_len12_counts_digits_after_separator_removal() {
        assert!(mynumber_len12("マイナンバー: 1234-5678-9012"));
        assert!(mynumber_len12("個人番号 １２３４ ５６７８ ９０１２"));
        assert!(mynumber_len12("個人番号 １２３４－５６７８ー９０１２"));
        assert!(mynumber_len12("個人番号（第１号）: 1234-5678-9012"));
        assert!(!mynumber_len12("1234-5678"));
        assert!(!mynumber_len12("1234-5678-9012-3"));
    }

    #[test]
    fn phone_mobile_accepts_jp_mobile_prefixes_only() {
        assert!(phone_mobile("090-1234-5678"));
        assert!(phone_mobile("０８０ １２３４ ５６７８"));
        assert!(phone_mobile("07012345678"));
        assert!(!phone_mobile("050-1234-5678"));
        assert!(!phone_mobile("090-123-4567"));
    }
}
