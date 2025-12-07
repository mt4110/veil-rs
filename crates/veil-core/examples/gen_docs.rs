use veil_core::get_default_rules;

fn main() {
    let rules = get_default_rules();

    println!("# Default Rules Reference\n");
    println!("Total rules: {}\n", rules.len());

    println!("| ID | Severity | Description |");
    println!("|----|----------|-------------|");

    for rule in rules {
        println!(
            "| `{}` | **{}** | {} |",
            rule.id, rule.severity, rule.description
        );
    }
}
