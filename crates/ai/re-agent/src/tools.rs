//! Built-in tools the agent can hand to Claude (or any tool-use-capable
//! model). Mirrors the rough shape of Claude Code's tool surface:
//! file_read, file_write, list_files, shell_exec, web_fetch.
//!
//! Tools are dynamic-dispatch via the `Tool` trait so adding a new one is a
//! single `impl Tool` block.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    fn run(&self, input: &Value) -> Result<String>;
}

/// Default toolbox handed to Claude. Add or remove freely.
pub fn builtin_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(FileRead),
        Box::new(FileWrite),
        Box::new(ListFiles),
        Box::new(ShellExec),
        Box::new(WebFetch),
    ]
}

/// Find the named tool and run it with the supplied JSON input.
pub fn dispatch(tools: &[Box<dyn Tool>], name: &str, input: &Value) -> Result<String> {
    for t in tools {
        if t.spec().name == name {
            return t.run(input);
        }
    }
    anyhow::bail!("unknown tool: {name}")
}

// ─── file_read ─────────────────────────────────────────────────────────────
pub struct FileRead;
impl Tool for FileRead {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_read".into(),
            description: "Read the contents of a UTF-8 text file from disk.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute or working-directory-relative path" }
                },
                "required": ["path"]
            }),
        }
    }
    fn run(&self, input: &Value) -> Result<String> {
        let path = input["path"].as_str().context("missing 'path'")?;
        Ok(std::fs::read_to_string(path)?)
    }
}

// ─── file_write ────────────────────────────────────────────────────────────
pub struct FileWrite;
impl Tool for FileWrite {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "file_write".into(),
            description: "Write content to a UTF-8 text file. Overwrites any existing file. Creates parent directories if needed.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path":    { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
        }
    }
    fn run(&self, input: &Value) -> Result<String> {
        let path = input["path"].as_str().context("missing 'path'")?;
        let content = input["content"].as_str().context("missing 'content'")?;
        if let Some(parent) = PathBuf::from(path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(path, content)?;
        Ok(format!("wrote {} bytes to {path}", content.len()))
    }
}

// ─── list_files ────────────────────────────────────────────────────────────
pub struct ListFiles;
impl Tool for ListFiles {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "list_files".into(),
            description: "List immediate entries (files and directories) at a path.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }),
        }
    }
    fn run(&self, input: &Value) -> Result<String> {
        let path = input["path"].as_str().context("missing 'path'")?;
        let mut out = String::new();
        for entry in std::fs::read_dir(path)? {
            let e = entry?;
            let kind = if e.file_type()?.is_dir() { "dir " } else { "file" };
            out.push_str(&format!("{kind}  {}\n", e.path().display()));
        }
        Ok(out)
    }
}

// ─── shell_exec ────────────────────────────────────────────────────────────
pub struct ShellExec;
impl Tool for ShellExec {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "shell_exec".into(),
            description: "Run a shell command (`sh -c`) and return combined stdout + stderr + exit code. Runs with the user's permissions.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "command": { "type": "string" } },
                "required": ["command"]
            }),
        }
    }
    fn run(&self, input: &Value) -> Result<String> {
        let cmd = input["command"].as_str().context("missing 'command'")?;
        let out = Command::new("sh").arg("-c").arg(cmd).output()?;
        let mut s = String::new();
        s.push_str(&String::from_utf8_lossy(&out.stdout));
        if !out.stderr.is_empty() {
            s.push_str("\n[stderr]\n");
            s.push_str(&String::from_utf8_lossy(&out.stderr));
        }
        s.push_str(&format!("\n[exit {}]", out.status.code().unwrap_or(-1)));
        Ok(s)
    }
}

// ─── web_fetch ─────────────────────────────────────────────────────────────
pub struct WebFetch;
impl Tool for WebFetch {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "web_fetch".into(),
            description: "HTTP GET a URL and return the response body as text.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "url": { "type": "string" } },
                "required": ["url"]
            }),
        }
    }
    fn run(&self, input: &Value) -> Result<String> {
        let url = input["url"].as_str().context("missing 'url'")?;
        let body = reqwest::blocking::get(url)?.text()?;
        Ok(body)
    }
}
