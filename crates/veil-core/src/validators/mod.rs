pub mod jp;

pub type ValidatorFn = fn(&str) -> bool;

pub fn resolve_validator(id: &str) -> Option<ValidatorFn> {
    match id {
        "jp_mynumber_len12" => Some(jp::mynumber_len12),
        "jp_phone_mobile" => Some(jp::phone_mobile),
        "luhn" => Some(luhn),
        _ => None,
    }
}

pub fn luhn(candidate: &str) -> bool {
    let chars: Vec<char> = candidate.chars().collect();
    chars
        .iter()
        .enumerate()
        .filter(|(_, ch)| digit_value(**ch).is_some())
        .filter(|(index, _)| !has_card_run_prefix(&chars, *index))
        .filter_map(|(index, _)| card_digit_run_from(&chars, index))
        .any(|values| luhn_values(&values))
}

fn luhn_values(values: &[u8]) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for digit in values.iter().rev() {
        let mut value = u32::from(*digit);
        if double {
            value *= 2;
            if value > 9 {
                value -= 9;
            }
        }
        sum += value;
        double = !double;
    }

    sum % 10 == 0
}

fn card_digit_run_from(chars: &[char], start: usize) -> Option<Vec<u8>> {
    let mut current = Vec::new();
    let mut group_len = 0usize;
    let mut first_group_len = None;
    let mut saw_separator = false;
    let mut last_was_separator = false;

    for ch in &chars[start..] {
        if let Some(digit) = digit_value(*ch) {
            current.push(digit);
            group_len += 1;
            last_was_separator = false;
            if current.len() > 19 {
                return None;
            }
            continue;
        }

        if is_card_separator(*ch) && !current.is_empty() && !last_was_separator {
            if first_group_len.is_none() {
                first_group_len = Some(group_len);
            }
            group_len = 0;
            saw_separator = true;
            last_was_separator = true;
            continue;
        }

        break;
    }

    if last_was_separator || !is_luhn_candidate(&current) {
        return None;
    }

    if saw_separator && first_group_len.unwrap_or(group_len) < 4 {
        return None;
    }

    Some(current)
}

fn has_card_run_prefix(chars: &[char], start: usize) -> bool {
    if start == 0 {
        return false;
    }

    if digit_value(chars[start - 1]).is_some() {
        return true;
    }

    let mut index = start;
    while index > 0 && is_card_separator(chars[index - 1]) {
        index -= 1;
    }

    if index == start {
        return false;
    }

    let prefix_len = digit_group_len_before(chars, index);
    prefix_len >= 4 && prefix_len + card_digit_run_len_from(chars, start) <= 19
}

fn digit_group_len_before(chars: &[char], end: usize) -> usize {
    let mut index = end;
    let mut len = 0;
    while index > 0 {
        index -= 1;
        if digit_value(chars[index]).is_some() {
            len += 1;
        } else {
            break;
        }
    }

    len
}

fn card_digit_run_len_from(chars: &[char], start: usize) -> usize {
    let mut len = 0;
    let mut last_was_separator = false;

    for ch in &chars[start..] {
        if digit_value(*ch).is_some() {
            len += 1;
            last_was_separator = false;
            continue;
        }

        if is_card_separator(*ch) && len > 0 && !last_was_separator {
            last_was_separator = true;
            continue;
        }

        break;
    }

    len
}

fn is_card_separator(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '-' | '\u{3000}')
}

fn is_luhn_candidate(values: &[u8]) -> bool {
    if !(13..=19).contains(&values.len()) {
        return false;
    }

    !is_known_test_card(&digits_from_values(values))
}

fn is_known_test_card(digits: &str) -> bool {
    matches!(
        digits,
        "4111111111111111"
            | "4242424242424242"
            | "4000000000000002"
            | "5555555555554444"
            | "5105105105105100"
            | "378282246310005"
            | "371449635398431"
            | "6011111111111117"
            | "30569309025904"
            | "3530111333300000"
            | "3566002020360505"
    )
}

pub(crate) fn digits_only(candidate: &str) -> String {
    digit_values(candidate)
        .into_iter()
        .map(ascii_digit)
        .collect()
}

fn digits_from_values(values: &[u8]) -> String {
    values.iter().copied().map(ascii_digit).collect()
}

fn ascii_digit(digit: u8) -> char {
    char::from(b'0' + digit)
}

fn digit_values(candidate: &str) -> Vec<u8> {
    candidate.chars().filter_map(digit_value).collect()
}

fn digit_value(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some(ch as u8 - b'0'),
        '\u{ff10}'..='\u{ff19}' => Some((ch as u32 - 0xff10) as u8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luhn_accepts_valid_non_test_card_candidates() {
        assert!(luhn("VISA: 4111222233334448"));
        assert!(luhn("AMEX 371112345678902"));
        assert!(luhn("JCB: 3511111122223333"));
        assert!(luhn("card_1: 4111222233334448"));
        assert!(luhn("card 2 4111222233334448"));
        assert!(luhn("card 2 ... 4111222233334448"));
        assert!(luhn("card exp 2026: 4111222233334448"));
        assert!(luhn("card exp 2026 4111222233334448"));
        assert!(luhn("VISA: 4111-2222-3333-4448"));
        assert!(luhn("VISA: 4111 2222 3333 4448"));
    }

    #[test]
    fn luhn_rejects_invalid_lengths_and_known_test_cards() {
        assert!(!luhn("411122223333"));
        assert!(!luhn("4111222233334444"));
        assert!(!luhn("card: 4000000000000000"));
        assert!(!luhn("batch 12344111222233334448"));
        assert!(!luhn("4111111111111111"));
        assert!(!luhn("5555555555554444"));
    }

    #[test]
    fn resolver_is_allowlisted() {
        assert!(resolve_validator("jp_mynumber_len12").is_some());
        assert!(resolve_validator("jp_phone_mobile").is_some());
        assert!(resolve_validator("luhn").is_some());
        assert!(resolve_validator("unknown").is_none());
    }
}
