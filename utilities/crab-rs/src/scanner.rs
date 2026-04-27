use crate::{ForeignTool, ToolSource};
use anyhow::Result;
use ignore::WalkBuilder;
use std::process::Command;
use std::path::Path;

/// New: Workspace-wide scanner that respects .gitignore
pub struct WorkspaceScanner {
    root: String,
}

impl WorkspaceScanner {
    pub fn new(root: &str) -> Self {
        Self { root: root.to_string() }
    }

    /// Scan for project types (Rust, Node, Python) while respecting .gitignore
    pub fn scan_projects(&self) -> Vec<String> {
        let mut projects = vec![];
        for result in WalkBuilder::new(&self.root).build() {
            if let Ok(entry) = result {
                let path = entry.path();
                if path.file_name() == Some("Cargo.toml".as_ref()) {
                    projects.push(format!("Rust: {:?}", path.parent().unwrap_or(Path::new(""))));
                } else if path.file_name() == Some("package.json".as_ref()) {
                    projects.push(format!("Node: {:?}", path.parent().unwrap_or(Path::new(""))));
                } else if path.file_name() == Some("requirements.txt".as_ref()) {
                    projects.push(format!("Python: {:?}", path.parent().unwrap_or(Path::new(""))));
                }
            }
        }
        projects
    }
}

pub fn scan_npm() -> Result<Vec<ForeignTool>> {
    let output = Command::new("npm").args(["list", "-g", "--depth=0", "--json"]).output();
    let output = match output { Ok(o) => o, Err(_) => return Ok(vec![]) };
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let mut tools = vec![];
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, meta) in deps {
            let version = meta.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
            tools.push(ForeignTool { name: name.clone(), source: ToolSource::Npm, version });
        }
    }
    Ok(tools)
}

pub fn scan_pipx() -> Result<Vec<ForeignTool>> {
    let output = Command::new("pipx").args(["list", "--json"]).output();
    let output = match output { Ok(o) => o, Err(_) => return Ok(vec![]) };
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let mut tools = vec![];
    if let Some(venvs) = json.get("venvs").and_then(|v| v.as_object()) {
        for (name, _) in venvs {
            tools.push(ForeignTool { name: name.clone(), source: ToolSource::Pipx, version: None });
        }
    }
    Ok(tools)
}
