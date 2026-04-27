//! Model selection, auth profiles, fallback chains
//! Translated from: openclaw/dist/model-selection-*.js, model-fallback-*.js

use serde::{Deserialize, Serialize};

/// A model reference in provider/model format
/// Source: formatProviderModelRef() in openclaw
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
}

impl ModelRef {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self { provider: provider.into(), model: model.into() }
    }

    /// Format as "provider/model"
    pub fn to_ref_string(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }

    /// Parse from "provider/model" string
    pub fn from_ref_string(s: &str) -> Option<Self> {
        let (provider, model) = s.split_once('/')?;
        Some(Self::new(provider, model))
    }
}

impl std::fmt::Display for ModelRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.provider, self.model)
    }
}

/// Supported model providers
/// Source: model-providers in openclaw docs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Anthropic,
    Openai,
    Google,
    Ollama,
    Lmstudio,
    Custom(String),
}

impl Provider {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "anthropic" => Self::Anthropic,
            "openai"    => Self::Openai,
            "google"    => Self::Google,
            "ollama"    => Self::Ollama,
            "lmstudio"  => Self::Lmstudio,
            other       => Self::Custom(other.to_string()),
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Ollama | Self::Lmstudio)
    }
}

/// Fallback attempt record
/// Source: buildFallbackReasonSummary() + formatFallbackAttemptReason() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackAttempt {
    pub provider: String,
    pub model: String,
    pub reason: Option<FallbackReason>,
    pub status: Option<u16>,
    pub error: Option<String>,
}

/// Why a fallback was triggered
/// Source: TRANSIENT_FALLBACK_REASONS in openclaw
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackReason {
    RateLimit,
    Overloaded,
    Timeout,
    BillingError,
    ContextOverflow,
    Other(String),
}

impl FallbackReason {
    /// Source: TRANSIENT_FALLBACK_REASONS set in openclaw
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::RateLimit | Self::Overloaded | Self::Timeout)
    }
}

impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimit     => write!(f, "rate_limit"),
            Self::Overloaded    => write!(f, "overloaded"),
            Self::Timeout       => write!(f, "timeout"),
            Self::BillingError  => write!(f, "billing_error"),
            Self::ContextOverflow => write!(f, "context_overflow"),
            Self::Other(s)      => write!(f, "{}", s),
        }
    }
}

/// Build a fallback summary string from attempts
/// Source: buildFallbackReasonSummary() in openclaw
pub fn build_fallback_summary(attempts: &[FallbackAttempt]) -> String {
    let first = attempts.first();
    let reason = first
        .and_then(|a| a.reason.as_ref())
        .map(|r| r.to_string())
        .unwrap_or_else(|| "selected model unavailable".to_string());

    let more = if attempts.len() > 1 {
        format!(" (+{} more attempts)", attempts.len() - 1)
    } else {
        String::new()
    };

    format!("{}{}", truncate(&reason, 80), more)
}

/// Build a model fallback notice string
/// Source: buildFallbackNotice() in openclaw
pub fn build_fallback_notice(
    selected: &ModelRef,
    active: &ModelRef,
    attempts: &[FallbackAttempt],
) -> Option<String> {
    if selected == active {
        return None;
    }
    Some(format!(
        "↪️ Model Fallback: {} (selected {}; {})",
        active,
        selected,
        build_fallback_summary(attempts)
    ))
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.len() <= max {
        s
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

/// Auth profile for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    pub provider: Provider,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}
