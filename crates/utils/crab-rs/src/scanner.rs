//! Scans the system for installed npm and pipx tools

use crate::{ForeignTool, ToolSource};
use anyhow::Result;
use std::process::Command;

/// Detect all globally installed npm packages
pub fn scan_npm() -> Result<Vec<ForeignTool>> {
    let output = Command::new("npm")
        .args(["list", "-g", "--depth=0", "--json"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(vec![]), // npm not installed
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let mut tools = vec![];

    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, meta) in deps {
            let version = meta.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            tools.push(ForeignTool {
                name: name.clone(),
                source: ToolSource::Npm,
                version,
            });
        }
    }

    Ok(tools)
}

/// Detect all globally installed pipx packages
pub fn scan_pipx() -> Result<Vec<ForeignTool>> {
    let output = Command::new("pipx")
        .args(["list", "--json"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(vec![]), // pipx not installed
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

    let mut tools = vec![];

    if let Some(venvs) = json.get("venvs").and_then(|v| v.as_object()) {
        for (name, meta) in venvs {
            let version = meta
                .get("metadata")
                .and_then(|m| m.get("main_package"))
                .and_then(|p| p.get("package_version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            tools.push(ForeignTool {
                name: name.clone(),
                source: ToolSource::Pipx,
                version,
            });
        }
    }

    Ok(tools)
}

/// Scan both npm and pipx
pub fn scan_all() -> Result<Vec<ForeignTool>> {
    let mut all = scan_npm()?;
    all.extend(scan_pipx()?);
    Ok(all)
}
