#[test]
fn cli_tests() {
    let cases = trycmd::TestCases::new();
    for path in std::fs::read_dir("tests/cmd").unwrap() {
        let path = path.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "toml")
            && path.file_name().unwrap() != "help.toml"
        {
            cases.case(path);
        }
    }
    cases.case("tests/cmd/help.toml");
}

#[test]
fn help_surface_matches_the_selected_profile() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_veil"))
        .args(["--lang", "en", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Local secret scanning and masking"));
    let commands: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("  "))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    for command in ["scan", "config", "rules", "lsp", "update", "verify"] {
        assert!(commands.contains(&command));
    }
    for command in ["detect", "prove", "decide", "case"] {
        assert!(!commands.contains(&command));
    }
    assert!(!commands.contains(&"guardian"));
}
