//! Crab-rs: detects npm & pipx tools and maps them to Rust-native equivalents.
//!
//! # Architecture
//! - `scanner`    — detects what npm/pipx tools are installed on the system
//! - `mapper`     — maps each detected tool to its Rust equivalent
//! - `runner`     — executes the Rust equivalent with the same intent
//! - `downloader` — installs/uninstalls packages from npm and pipx registries
//! - `tools`      — known translation table (npm tool → rust crate/binary)

pub mod downloader;
pub mod mapper;
pub mod runner;
pub mod scanner;
pub mod tools;

/// A detected foreign tool (npm or pipx)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForeignTool {
    pub name: String,
    pub source: ToolSource,
    pub version: Option<String>,
}

/// Where the tool came from
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ToolSource {
    Npm,
    Pipx,
}

/// The Rust-native equivalent of a foreign tool
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RustEquivalent {
    pub crate_name: String,
    pub binary: String,
    pub install_cmd: String,
    pub description: String,
}

/// Full translation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Translation {
    pub foreign: ForeignTool,
    pub equivalent: Option<RustEquivalent>,
}
