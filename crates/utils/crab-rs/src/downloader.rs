//! Downloads (installs) packages from npm and pipx registries

use anyhow::{Context, Result};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub enum Registry {
    Npm,
    Pipx,
}

impl std::fmt::Display for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Registry::Npm => write!(f, "npm"),
            Registry::Pipx => write!(f, "pipx"),
        }
    }
}

/// Check if npm is available on the system
pub fn npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if pipx is available on the system
pub fn pipx_available() -> bool {
    Command::new("pipx")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Install a package globally via npm
pub fn npm_install(package: &str, version: Option<&str>) -> Result<()> {
    if !npm_available() {
        anyhow::bail!(
            "npm is not installed or not in PATH.\n  \
             Install Node.js from https://nodejs.org or use `crab run` for Rust equivalents."
        );
    }

    let pkg = match version {
        Some(v) => format!("{}@{}", package, v),
        None => package.to_string(),
    };

    println!("📦 Installing '{}' via npm...\n", pkg);

    let status = Command::new("npm")
        .args(["install", "-g", &pkg])
        .status()
        .with_context(|| format!("failed to run npm install -g {}", pkg))?;

    if !status.success() {
        anyhow::bail!("npm install failed for '{}'", pkg);
    }

    println!("\n✅ '{}' installed successfully via npm.", pkg);
    Ok(())
}

/// Install a package via pipx
pub fn pipx_install(package: &str, version: Option<&str>) -> Result<()> {
    if !pipx_available() {
        anyhow::bail!(
            "pipx is not installed or not in PATH.\n  \
             Install it with: pip install --user pipx"
        );
    }

    let pkg = match version {
        Some(v) => format!("{}=={}", package, v),
        None => package.to_string(),
    };

    println!("📦 Installing '{}' via pipx...\n", pkg);

    let status = Command::new("pipx")
        .args(["install", &pkg])
        .status()
        .with_context(|| format!("failed to run pipx install {}", pkg))?;

    if !status.success() {
        anyhow::bail!("pipx install failed for '{}'", pkg);
    }

    println!("\n✅ '{}' installed successfully via pipx.", pkg);
    Ok(())
}

/// Uninstall a package via npm
pub fn npm_uninstall(package: &str) -> Result<()> {
    if !npm_available() {
        anyhow::bail!("npm is not installed or not in PATH.");
    }

    println!("🗑  Uninstalling '{}' via npm...\n", package);

    let status = Command::new("npm")
        .args(["uninstall", "-g", package])
        .status()
        .with_context(|| format!("failed to run npm uninstall -g {}", package))?;

    if !status.success() {
        anyhow::bail!("npm uninstall failed for '{}'", package);
    }

    println!("\n✅ '{}' uninstalled via npm.", package);
    Ok(())
}

/// Uninstall a package via pipx
pub fn pipx_uninstall(package: &str) -> Result<()> {
    if !pipx_available() {
        anyhow::bail!("pipx is not installed or not in PATH.");
    }

    println!("🗑  Uninstalling '{}' via pipx...\n", package);

    let status = Command::new("pipx")
        .args(["uninstall", package])
        .status()
        .with_context(|| format!("failed to run pipx uninstall {}", package))?;

    if !status.success() {
        anyhow::bail!("pipx uninstall failed for '{}'", package);
    }

    println!("\n✅ '{}' uninstalled via pipx.", package);
    Ok(())
}

/// List globally installed npm packages
pub fn npm_list() -> Result<()> {
    if !npm_available() {
        anyhow::bail!("npm is not installed or not in PATH.");
    }

    Command::new("npm")
        .args(["list", "-g", "--depth=0"])
        .status()
        .context("failed to run npm list")?;

    Ok(())
}

/// List pipx installed packages
pub fn pipx_list() -> Result<()> {
    if !pipx_available() {
        anyhow::bail!("pipx is not installed or not in PATH.");
    }

    Command::new("pipx")
        .arg("list")
        .status()
        .context("failed to run pipx list")?;

    Ok(())
}

/// Search npm registry for a package (shows info)
pub fn npm_info(package: &str) -> Result<()> {
    if !npm_available() {
        anyhow::bail!("npm is not installed or not in PATH.");
    }

    println!("🔍 Fetching npm info for '{}'...\n", package);

    Command::new("npm")
        .args(["info", package, "name", "version", "description", "homepage"])
        .status()
        .with_context(|| format!("failed to run npm info {}", package))?;

    Ok(())
}

/// Show pipx package info
pub fn pipx_info(package: &str) -> Result<()> {
    if !pipx_available() {
        anyhow::bail!("pipx is not installed or not in PATH.");
    }

    println!("🔍 Fetching pipx info for '{}'...\n", package);

    Command::new("pip")
        .args(["index", "versions", package])
        .status()
        .with_context(|| format!("failed to fetch pip info for {}", package))?;

    Ok(())
}
