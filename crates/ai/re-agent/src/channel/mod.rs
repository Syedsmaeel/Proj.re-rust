//! Message channel abstraction
//! Translated from: openclaw/dist/message-channel-C2Lnao8s.js

use serde::{Deserialize, Serialize};

/// All supported message channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageChannel {
    Cli,
    Telegram,
    Discord,
    Slack,
    Whatsapp,
    Webchat,
    Imessage,
    Signal,
    Internal,
    Unknown(String),
}

impl MessageChannel {
    pub fn from_raw(raw: &str) -> Self {
        match raw.to_lowercase().trim() {
            "cli"      => Self::Cli,
            "telegram" => Self::Telegram,
            "discord"  => Self::Discord,
            "slack"    => Self::Slack,
            "whatsapp" => Self::Whatsapp,
            "webchat"  => Self::Webchat,
            "imessage" => Self::Imessage,
            "signal"   => Self::Signal,
            "internal" => Self::Internal,
            other      => Self::Unknown(other.to_string()),
        }
    }

    /// Source: isMarkdownCapableMessageChannel() in openclaw
    pub fn is_markdown_capable(&self) -> bool {
        matches!(self, Self::Cli | Self::Webchat | Self::Discord | Self::Slack)
    }

    /// Source: isInternalMessageChannel() in openclaw
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal)
    }

    /// Source: isRoutableChannel() in openclaw
    pub fn is_routable(&self) -> bool {
        !self.is_internal()
    }
}

impl std::fmt::Display for MessageChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Cli        => "cli",
            Self::Telegram   => "telegram",
            Self::Discord    => "discord",
            Self::Slack      => "slack",
            Self::Whatsapp   => "whatsapp",
            Self::Webchat    => "webchat",
            Self::Imessage   => "imessage",
            Self::Signal     => "signal",
            Self::Internal   => "internal",
            Self::Unknown(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GatewayClientMode {
    Webchat,
    Cli,
    Ui,
    Backend,
    Node,
    Probe,
    Test,
}
