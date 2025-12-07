use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn init() -> Result<()> {
    let path = Path::new("veil.toml");
    if path.exists() {
        anyhow::bail!("veil.toml already exists!");
    }

    let content = r#"# Veil Configuration

[core]
# Paths to ignore during scanning
ignore = [ 
    "target", 
    ".git", 
    "node_modules", 
    "vendor",
    "dist",
    "build"
]
# Paths to always include (optional)
include = []

# Rule overrides (example)
[rules]
# "password_assignment" = { enabled = true, severity = "Critical" }
"#;

    fs::write(path, content)?;
    println!("Created default configuration: veil.toml");
    Ok(())
}
