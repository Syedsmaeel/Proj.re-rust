//! Executes Rust-native equivalents of foreign tools

use crate::RustEquivalent;
use anyhow::{Context, Result};
use std::process::Command;

/// Check if a binary exists on the system PATH
pub fn is_installed(binary: &str) -> bool {
    // Handle compound commands like "cargo clippy"
    let bin = binary.split_whitespace().next().unwrap_or(binary);
    Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run a Rust equivalent binary with given args
pub fn run(equiv: &RustEquivalent, args: &[String]) -> Result<()> {
    let parts: Vec<&str> = equiv.binary.split_whitespace().collect();
    let (bin, base_args) = parts.split_first()
        .context("empty binary name")?;

    let status = Command::new(bin)
        .args(base_args)
        .args(args)
        .status()
        .with_context(|| format!("failed to run '{}'", equiv.binary))?;

    if !status.success() {
        anyhow::bail!("'{}' exited with status {}", equiv.binary, status);
    }

    Ok(())
}

/// Print install instructions for a missing equivalent
pub fn print_install_hint(equiv: &RustEquivalent) {
    println!("  → Not installed. Run: {}", equiv.install_cmd);
}
