pub fn mask_string(content: &str, range: std::ops::Range<usize>) -> String {
    let mut result = content.to_string();
    result.replace_range(range, "<REDACTED>");
    result
}
