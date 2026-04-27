//! Config loading and workspace resolution
//! Translated from: openclaw/dist/io-*.js, config-*.js, paths-*.js

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level agent config
/// Source: loadConfig() + openclaw.json schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub agents: AgentsConfig,
    pub models: ModelsConfig,
    pub workspace: Option<PathBuf>,
    pub log_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    pub defaults: AgentDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentDefaults {
    /// Primary model ref e.g. "anthropic/claude-sonnet-4-6"
    pub model: Option<ModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    pub primary: Option<String>,
    pub fallbacks: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsConfig {
    pub providers: std::collections::HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
}

impl AgentConfig {
    /// Load config from ~/.re-agent/config.toml or TOML file path
    /// Source: loadConfig() in openclaw
    pub fn load() -> Result<Self> {
        let path = Self::default_config_path();
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("reading config at {:?}", path))?;
            let config: AgentConfig = toml::from_str(&raw)
                .with_context(|| "parsing config TOML")?;
            Ok(config)
        } else {
            Ok(AgentConfig::default())
        }
    }

    pub fn default_config_path() -> PathBuf {
        dirs_path().join("config.toml")
    }

    pub fn workspace_path(&self) -> PathBuf {
        self.workspace.clone().unwrap_or_else(|| dirs_path().join("workspace"))
    }

    pub fn primary_model(&self) -> Option<&str> {
        self.agents.defaults.model.as_ref()?.primary.as_deref()
    }

    pub fn fallback_models(&self) -> Vec<&str> {
        self.agents.defaults.model.as_ref()
            .and_then(|m| m.fallbacks.as_ref())
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Source: hasConfiguredModelFallbacks() in openclaw
    pub fn has_fallbacks(&self) -> bool {
        !self.fallback_models().is_empty()
    }
}

fn dirs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".re-agent")
}
