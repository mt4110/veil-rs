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
    let values = digit_values(candidate);
    if !(13..=19).contains(&values.len()) {
        return false;
    }

    let digits = digits_only(candidate);
    if is_known_test_card(&digits) {
        return false;
    }

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
        .map(|digit| char::from(b'0' + digit))
        .collect()
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
    }

    #[test]
    fn luhn_rejects_invalid_lengths_and_known_test_cards() {
        assert!(!luhn("411122223333"));
        assert!(!luhn("4111222233334444"));
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
