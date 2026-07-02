use super::digits_only;

pub fn mynumber_len12(candidate: &str) -> bool {
    digits_only(candidate).len() == 12
}

pub fn phone_mobile(candidate: &str) -> bool {
    let digits = digits_only(candidate);
    digits.len() == 11
        && (digits.starts_with("070") || digits.starts_with("080") || digits.starts_with("090"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mynumber_len12_counts_digits_after_separator_removal() {
        assert!(mynumber_len12("マイナンバー: 1234-5678-9012"));
        assert!(mynumber_len12("個人番号 １２３４ ５６７８ ９０１２"));
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
