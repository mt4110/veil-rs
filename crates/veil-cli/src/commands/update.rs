use anyhow::Result;
use colored::*;

pub fn update() -> Result<()> {
    println!("Checking for updates...");
    println!("(Update check is currently a stub - always latest version in dev)");
    println!("{}", "You are using the latest version.".green());
    Ok(())
}
