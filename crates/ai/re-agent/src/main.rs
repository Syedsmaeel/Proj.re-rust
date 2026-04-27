//! re-agent CLI — Rust-native agentic runner
//! Translated core architecture from OpenClaw (MIT) → Rust (AGPL-3.0)
//! Syed Ismaeel — Lucknow, Est. 2019

use anyhow::Result;
use clap::{Parser, Subcommand};
use re_agent::{
    agent::{AgentContext, record_message},
    channel::MessageChannel,
    config::AgentConfig,
    gateway::{Gateway, InboundMessage, OutboundReply},
    model::ModelRef,
    session::MessageRole,
};

#[derive(Parser)]
#[command(
    name = "re-agent",
    about = "Rust-native agentic runner — OpenClaw architecture translated to Rust",
    version,
    author = "Syed Ismaeel — Lucknow, Est. 2019"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show current config and workspace
    Config,

    /// Show gateway status
    Status,

    /// Send a message through the agent. Dry run by default; pass --live with
    /// the `phi` feature compiled in to actually call Phi-3 via re-llm.
    Send {
        /// Message text
        message: String,
        /// Channel to use (cli, telegram, discord...)
        #[arg(long, default_value = "cli")]
        channel: String,
        /// Model ref (e.g. local/phi-3-mini-4k, anthropic/claude-sonnet-4-6)
        #[arg(long, default_value = "local/phi-3-mini-4k")]
        model: String,
        /// Actually run the model. Requires `--features phi` at compile time.
        #[arg(long, default_value_t = false)]
        live: bool,
        /// Max tokens to generate when --live
        #[arg(long, default_value_t = 256)]
        max_tokens: usize,
        /// Sampling temperature when --live
        #[arg(long, default_value_t = 0.7)]
        temperature: f64,
    },

    /// Show translated OpenClaw architecture map
    Architecture,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Config => {
            let config = AgentConfig::load()?;
            println!("\n🦀 re-agent — Config\n");
            println!("  Workspace:     {:?}", config.workspace_path());
            println!("  Config path:   {:?}", AgentConfig::default_config_path());
            println!("  Primary model: {}", config.primary_model().unwrap_or("not set"));
            println!("  Has fallbacks: {}", config.has_fallbacks());
            println!();
        }

        Cmd::Status => {
            let gateway = Gateway::new();
            println!("\n🦀 re-agent — Gateway Status\n");
            println!("  Draining:  {}", gateway.is_draining);
            println!("  Sessions:  {}", gateway.sessions.len());
            println!("  Status:    ✅ ready");
            println!();
        }

        Cmd::Send {
            message,
            channel,
            model,
            live,
            max_tokens,
            temperature,
        } => {
            let channel = MessageChannel::from_raw(&channel);
            let model_ref = ModelRef::from_ref_string(&model)
                .unwrap_or_else(|| ModelRef::new("local", "phi-3-mini-4k"));

            let mut gateway = Gateway::new();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let inbound = InboundMessage {
                id: uuid_simple(),
                channel: channel.clone(),
                from: "user".to_string(),
                text: message.clone(),
                timestamp: now,
                reply_to: None,
            };

            let session_key = gateway.route(&inbound)?;
            let session =
                gateway.get_or_create_session(&session_key, "default", &channel.to_string());

            let ctx = AgentContext::new("default", &session_key, channel.clone(), model_ref);

            record_message(session, MessageRole::User, &message, &channel);

            println!("\n🦀 re-agent — Message Routed\n");
            println!("  Session key: {}", session_key);
            println!("  Channel:     {}", channel);
            println!("  Model:       {}", ctx.model);
            println!("  Markdown:    {}", channel.is_markdown_capable());
            println!("  Messages:    {}", session.messages.len());
            println!();

            let reply_text = if live {
                run_live(&message, &ctx.model, max_tokens, temperature)?
            } else {
                println!(
                    "  [Dry run — pass --live (and build with --features phi) to call Phi.]\n"
                );
                "[agent reply would go here]".to_string()
            };

            let reply = OutboundReply::new(channel.clone(), "user", &reply_text);
            record_message(session, MessageRole::Assistant, &reply_text, &channel);

            println!("  Reply has content: {}", reply.has_content());
            println!("  Routable channel:  {}", channel.is_routable());
            if live {
                println!();
                println!("  ── reply ──────────────────────────────────────────────");
                println!("{reply_text}");
                println!("  ───────────────────────────────────────────────────────");
            }
        }

        Cmd::Architecture => {
            println!("\n🦀 re-agent — OpenClaw → Rust Architecture Map\n");
            println!("  OpenClaw Module              → re-agent Rust Module");
            println!("  {}", "-".repeat(60));
            println!("  message-channel-*.js         → src/channel/mod.rs");
            println!("  model-selection-*.js          → src/model/mod.rs");
            println!("  model-fallback-*.js           → src/model/mod.rs");
            println!("  agent-runner.runtime-*.js     → src/agent/mod.rs");
            println!("  agent-events-*.js             → src/agent/mod.rs");
            println!("  store-*.js / session-utils-*  → src/session/mod.rs");
            println!("  io-*.js / config-*.js         → src/config/mod.rs");
            println!("  route-reply-*.js / queue-*.js → src/gateway/mod.rs");
            println!("  runs-*.js / approval-gateway  → src/gateway/mod.rs");
            println!();
            println!("  Source: openclaw/openclaw (MIT)");
            println!("  Translation: Rust/AGPL-3.0 by Syed Ismaeel");
            println!();
        }
    }

    Ok(())
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:x}-{:x}", t.as_secs(), t.subsec_nanos())
}

/// Live model call. Behind the `phi` feature so the default build stays light
/// and doesn't pull in the candle / hf-hub stack.
#[cfg(feature = "phi")]
fn run_live(
    message: &str,
    model: &re_agent::model::ModelRef,
    max_tokens: usize,
    temperature: f64,
) -> Result<String> {
    use re_llm::{GenerationConfig, PhiBackend, PhiVariant};

    // Only `local/phi-*` model refs are wired up for now.
    if model.provider != "local" {
        anyhow::bail!(
            "--live currently only supports local/phi-* models; got {}",
            model
        );
    }

    let variant = PhiVariant::parse(&model.model.replace("phi-3-", ""));
    eprintln!(
        "🦀 re-agent — loading {} via re-llm (first run downloads weights)…",
        variant.repo_id()
    );

    let mut backend = PhiBackend::load(variant)?;
    let cfg = GenerationConfig {
        max_tokens,
        temperature,
        ..Default::default()
    };
    backend.generate(message, &cfg)
}

#[cfg(not(feature = "phi"))]
fn run_live(
    _message: &str,
    _model: &re_agent::model::ModelRef,
    _max_tokens: usize,
    _temperature: f64,
) -> Result<String> {
    anyhow::bail!(
        "--live requires building with --features phi (e.g. \
         `cargo run -p re-agent --features phi -- send --live ...`)"
    )
}
