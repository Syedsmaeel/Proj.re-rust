//! Agent runner — core execution engine
//! Translated from: openclaw/dist/agent-runner.runtime-*.js
//! Key functions: runWithModelFallback, buildAgentRuntimeDeliveryPlan,
//!                buildAgentRuntimeOutcomePlan, createTypingSignaler

use crate::{
    channel::MessageChannel,
    model::{build_fallback_notice, FallbackAttempt, FallbackReason, ModelRef},
    session::{MessageRole, SessionEntry, SessionMessage, TokenUsage},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Agent execution context
/// Source: registerAgentRunContext() + resolveSessionAgentId() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub agent_id: String,
    pub session_key: String,
    pub channel: MessageChannel,
    pub model: ModelRef,
    pub verbose: bool,
}

impl AgentContext {
    pub fn new(
        agent_id: impl Into<String>,
        session_key: impl Into<String>,
        channel: MessageChannel,
        model: ModelRef,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            session_key: session_key.into(),
            channel,
            model,
            verbose: false,
        }
    }
}

/// Outcome of a single agent run
/// Source: buildAgentRuntimeOutcomePlan() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutcome {
    pub success: bool,
    pub reply: Option<String>,
    pub usage: TokenUsage,
    pub fallback_notice: Option<String>,
    pub error: Option<String>,
}

impl RunOutcome {
    pub fn ok(reply: impl Into<String>, usage: TokenUsage) -> Self {
        Self {
            success: true,
            reply: Some(reply.into()),
            usage,
            fallback_notice: None,
            error: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            reply: None,
            usage: TokenUsage::default(),
            fallback_notice: None,
            error: Some(error.into()),
        }
    }

    pub fn with_fallback(mut self, notice: Option<String>) -> Self {
        self.fallback_notice = notice;
        self
    }
}

/// Silent reply token detection
/// Source: isSilentReplyText() + SILENT_REPLY_TOKEN in openclaw
const SILENT_REPLY_TOKEN: &str = "\u{200B}"; // zero-width space

pub fn is_silent_reply(text: &str) -> bool {
    text.trim() == SILENT_REPLY_TOKEN || text.trim().is_empty()
}

pub fn strip_silent_token(text: &str) -> &str {
    text.trim_start_matches(SILENT_REPLY_TOKEN)
}

/// Classify transient vs fatal errors
/// Source: isTransientHttpError() in openclaw
pub fn classify_error(status: Option<u16>, message: &str) -> FallbackReason {
    let msg = message.to_lowercase();
    if msg.contains("rate limit") || msg.contains("429") || msg.contains("too many requests") {
        return FallbackReason::RateLimit;
    }
    if msg.contains("overloaded") || msg.contains("503") || msg.contains("service unavailable") {
        return FallbackReason::Overloaded;
    }
    if msg.contains("timeout") || msg.contains("timed out") {
        return FallbackReason::Timeout;
    }
    if msg.contains("billing") || msg.contains("quota") || msg.contains("payment") {
        return FallbackReason::BillingError;
    }
    if msg.contains("context") && (msg.contains("overflow") || msg.contains("length")) {
        return FallbackReason::ContextOverflow;
    }
    if let Some(s) = status {
        if s == 429 { return FallbackReason::RateLimit; }
        if s >= 500 { return FallbackReason::Overloaded; }
    }
    FallbackReason::Other(message.to_string())
}

/// Run with model fallback
/// Source: runWithModelFallback() in openclaw
pub async fn run_with_fallback<F, Fut>(
    ctx: &AgentContext,
    fallback_models: &[ModelRef],
    run_fn: F,
) -> Result<RunOutcome>
where
    F: Fn(ModelRef) -> Fut,
    Fut: std::future::Future<Output = Result<RunOutcome>>,
{
    let mut attempts: Vec<FallbackAttempt> = vec![];
    let primary = ctx.model.clone();
    let mut all_models = vec![primary.clone()];
    all_models.extend_from_slice(fallback_models);

    for model_ref in &all_models {
        match run_fn(model_ref.clone()).await {
            Ok(outcome) => {
                let notice = build_fallback_notice(&primary, model_ref, &attempts);
                return Ok(outcome.with_fallback(notice));
            }
            Err(e) => {
                let reason = classify_error(None, &e.to_string());
                if !reason.is_transient() {
                    return Err(e);
                }
                attempts.push(FallbackAttempt {
                    provider: model_ref.provider.clone(),
                    model: model_ref.model.clone(),
                    reason: Some(reason),
                    status: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Err(anyhow::anyhow!(
        "All models exhausted. Attempts: {}",
        attempts.len()
    ))
}

/// Record a message into the session
pub fn record_message(
    session: &mut SessionEntry,
    role: MessageRole,
    content: impl Into<String>,
    channel: &MessageChannel,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    session.add_message(SessionMessage {
        role,
        content: content.into(),
        timestamp: now,
        channel: Some(channel.to_string()),
        tokens: None,
    });
}
