pub const SERVER_NAME: &str = "veil-lsp";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_name_matches_binary_name() {
        assert_eq!(SERVER_NAME, "veil-lsp");
    }
}
