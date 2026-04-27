//! Session store, transcript, memory management
//! Translated from: openclaw/dist/store-*.js, session-utils.fs-*.js, memory-state-*.js

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single message in the session transcript
/// Source: readSessionMessages() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: u64,
    pub channel: Option<String>,
    pub tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Session token usage
/// Source: deriveSessionTotalTokens() + normalizeUsage() in openclaw
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn has_nonzero(&self) -> bool {
        self.total_tokens > 0
    }

    pub fn derive_prompt_tokens(&self) -> u32 {
        self.prompt_tokens
    }

    pub fn total(&self) -> u32 {
        self.prompt_tokens + self.completion_tokens
    }
}

/// Session store entry
/// Source: updateSessionStoreEntry() + resolveGroupSessionKey() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub session_key: String,
    pub agent_id: String,
    pub channel: String,
    pub messages: Vec<SessionMessage>,
    pub usage: TokenUsage,
    pub created_at: u64,
    pub updated_at: u64,
    pub compaction_count: u32,
}

impl SessionEntry {
    pub fn new(session_key: impl Into<String>, agent_id: impl Into<String>, channel: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            session_key: session_key.into(),
            agent_id: agent_id.into(),
            channel: channel.into(),
            messages: vec![],
            usage: TokenUsage::default(),
            created_at: now,
            updated_at: now,
            compaction_count: 0,
        }
    }

    pub fn add_message(&mut self, msg: SessionMessage) {
        self.messages.push(msg);
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Source: incrementCompactionCount() in openclaw
    pub fn increment_compaction(&mut self) {
        self.compaction_count += 1;
    }
}

/// Memory flush plan
/// Source: resolveMemoryFlushPlan() in openclaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFlushPlan {
    pub should_flush: bool,
    pub flush_reason: Option<String>,
    pub target_path: Option<PathBuf>,
}

/// Session file path resolution
/// Source: resolveSessionFilePath() + resolveSessionTranscriptPath() in openclaw
pub fn resolve_session_path(workspace: &PathBuf, session_key: &str) -> PathBuf {
    workspace.join("sessions").join(format!("{}.json", session_key))
}

pub fn resolve_transcript_path(workspace: &PathBuf, session_key: &str) -> PathBuf {
    workspace.join("transcripts").join(format!("{}.transcript.json", session_key))
}
