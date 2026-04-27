//! Gateway — message routing and session management
//! Translated from: openclaw/dist/approval-gateway-*.js, route-reply-*.js,
//!                  queue-*.js, runs-*.js

use crate::{channel::MessageChannel, session::SessionEntry};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gateway draining error — raised when gateway shuts down mid-run
/// Source: GatewayDrainingError in openclaw
#[derive(Debug, thiserror::Error)]
#[error("Gateway is draining — run aborted")]
pub struct GatewayDrainingError;

/// Command lane cleared error
/// Source: CommandLaneClearedError in openclaw
#[derive(Debug, thiserror::Error)]
#[error("Command lane cleared")]
pub struct CommandLaneClearedError;

/// Inbound message from any channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub id: String,
    pub channel: MessageChannel,
    pub from: String,
    pub text: String,
    pub timestamp: u64,
    pub reply_to: Option<String>,
}

/// Outbound reply to a channel
/// Source: resolveSendableOutboundReplyParts() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundReply {
    pub channel: MessageChannel,
    pub to: String,
    pub text: String,
    pub reply_to: Option<String>,
    pub is_markdown: bool,
}

impl OutboundReply {
    pub fn new(channel: MessageChannel, to: impl Into<String>, text: impl Into<String>) -> Self {
        let is_markdown = channel.is_markdown_capable();
        Self {
            channel,
            to: to.into(),
            text: text.into(),
            reply_to: None,
            is_markdown,
        }
    }

    /// Source: hasOutboundReplyContent() in openclaw
    pub fn has_content(&self) -> bool {
        !self.text.trim().is_empty()
    }
}

/// Gateway state
pub struct Gateway {
    pub sessions: HashMap<String, SessionEntry>,
    pub is_draining: bool,
}

impl Gateway {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            is_draining: false,
        }
    }

    /// Route an inbound message to the correct session/agent
    /// Source: routeReply() + resolveOriginMessageProvider() in openclaw
    pub fn route(&self, msg: &InboundMessage) -> Result<String> {
        if self.is_draining {
            return Err(GatewayDrainingError.into());
        }
        // session key = channel + from
        Ok(format!("{}:{}", msg.channel, msg.from))
    }

    /// Get or create session for a key
    pub fn get_or_create_session(
        &mut self,
        session_key: &str,
        agent_id: &str,
        channel: &str,
    ) -> &mut SessionEntry {
        self.sessions
            .entry(session_key.to_string())
            .or_insert_with(|| SessionEntry::new(session_key, agent_id, channel))
    }

    /// Drain the gateway — stops accepting new runs
    /// Source: GatewayDrainingError in openclaw
    pub fn drain(&mut self) {
        self.is_draining = true;
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}
