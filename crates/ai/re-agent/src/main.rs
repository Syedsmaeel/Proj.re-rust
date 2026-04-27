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
        /// Model ref (e.g. local/phi-3-mini-4k)
        #[arg(long, default_value = "local/phi-3-mini-4k")]
        model: String,
        /// Actually run the model. Requires `--features phi` at compile time.
        #[arg(long, default_value_t = false)]
        live: bool,
        /// Give the model the built-in toolbox (file_read, file_write,
        /// list_files, shell_exec, web_fetch) and run a ReAct-style loop:
        /// model emits <tool>{...}</tool>, we execute, feed the result back
        /// in <result>...</result>, repeat. Implies --live.
        #[arg(long, default_value_t = false)]
        with_tools: bool,
        /// Max ReAct loop iterations when --with-tools
        #[arg(long, default_value_t = 6)]
        max_steps: usize,
        /// Max tokens to generate per turn when --live
        #[arg(long, default_value_t = 256)]
        max_tokens: usize,
        /// Sampling temperature when --live
        #[arg(long, default_value_t = 0.7)]
        temperature: f64,
    },

    /// Show translated OpenClaw architecture map
    Architecture,

    /// List the built-in tools the agent will hand to the model when --with-tools
    Tools,
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
            with_tools,
            max_steps,
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

            let go_live = live || with_tools;
            let reply_text = if with_tools {
                run_with_tools(&message, &ctx.model, max_steps, max_tokens, temperature)?
            } else if go_live {
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
            if go_live {
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
            println!("  (new) tool-use loop           → src/tools.rs");
            println!();
            println!("  Source: openclaw/openclaw (MIT)");
            println!("  Translation: Rust/AGPL-3.0 by Syed Ismaeel");
            println!();
        }

        Cmd::Tools => {
            println!("\n🦀 re-agent — Built-in Tools\n");
            for t in re_agent::tools::builtin_tools() {
                let s = t.spec();
                println!("  • {}", s.name);
                println!("      {}", s.description);
                println!("      schema: {}", s.input_schema);
                println!();
            }
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

/// ReAct-style tool loop on top of local Phi.
///
/// Phi-3 doesn't have a native function-calling format like Claude does, so
/// we drive it with a prompt: the system prompt describes the toolbox, and
/// the model is told to emit `<tool>{...}</tool>` to call one. We parse the
/// block, run the tool from `re_agent::tools`, and feed the result back as
/// `<result>...</result>` in the next prompt. Loop until the model stops
/// emitting `<tool>` blocks (i.e. answers normally) or `max_steps` hits.
///
/// Cost: $0. Runs entirely on the user's machine.
#[cfg(feature = "phi")]
fn run_with_tools(
    message: &str,
    model: &re_agent::model::ModelRef,
    max_steps: usize,
    max_tokens: usize,
    temperature: f64,
) -> Result<String> {
    use re_agent::tools::{builtin_tools, dispatch};
    use re_llm::{GenerationConfig, PhiBackend, PhiVariant};
    use serde_json::Value;

    if model.provider != "local" {
        anyhow::bail!(
            "--with-tools currently only supports local/phi-* models; got {}",
            model
        );
    }

    let variant = PhiVariant::parse(&model.model.replace("phi-3-", ""));
    eprintln!(
        "🦀 re-agent — loading {} for tool-use loop…",
        variant.repo_id()
    );

    let mut backend = PhiBackend::load(variant)?;
    let toolbox = builtin_tools();

    // Build the tool catalog the model sees.
    let mut catalog = String::from(
        "You are re-agent, a Rust agentic runner. You can act on the real world by calling tools.\n\n\
         To call a tool, output EXACTLY one block (and nothing else on that line):\n\
         <tool>{\"name\":\"<tool_name>\",\"input\":{...}}</tool>\n\n\
         You will receive the result inside <result>...</result>. Then continue.\n\
         When you have an answer for the user, reply normally without any <tool> block.\n\n\
         Available tools:\n",
    );
    for t in &toolbox {
        let s = t.spec();
        catalog.push_str(&format!(
            "  - {}: {}\n      schema: {}\n",
            s.name,
            s.description,
            s.input_schema
        ));
    }

    let mut transcript = format!("{catalog}\n\nUser: {message}\nAssistant: ");
    let cfg = GenerationConfig {
        max_tokens,
        temperature,
        ..Default::default()
    };

    for step in 0..max_steps {
        let out = backend.generate(&transcript, &cfg)?;
        eprintln!("\n[step {step}] phi → {} chars", out.len());

        // Look for a tool call.
        if let (Some(start), Some(end)) = (out.find("<tool>"), out.find("</tool>")) {
            if start < end {
                let json_str = out[start + "<tool>".len()..end].trim();
                match serde_json::from_str::<Value>(json_str) {
                    Ok(call) => {
                        let name = call
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let input = call.get("input").cloned().unwrap_or(Value::Null);
                        eprintln!("🔧 tool_use: {name}({input})");

                        let result = dispatch(&toolbox, &name, &input)
                            .unwrap_or_else(|e| format!("error: {e}"));
                        let truncated = if result.len() > 4000 {
                            format!("{}…[truncated {} chars]", &result[..4000], result.len() - 4000)
                        } else {
                            result
                        };
                        eprintln!("   → {} chars returned", truncated.len());

                        // Append the model's tool call + the tool result, then
                        // re-cue the assistant.
                        let until_tool_close = &out[..end + "</tool>".len()];
                        transcript.push_str(until_tool_close);
                        transcript.push_str(&format!(
                            "\n<result>{truncated}</result>\nAssistant: "
                        ));
                        continue;
                    }
                    Err(e) => {
                        return Ok(format!(
                            "{out}\n\n[re-agent: failed to parse <tool> block as JSON: {e}]"
                        ));
                    }
                }
            }
        }

        // No tool call → final answer.
        return Ok(out);
    }

    Ok(format!(
        "[re-agent: hit max_steps={max_steps} without a final answer]"
    ))
}

#[cfg(not(feature = "phi"))]
fn run_with_tools(
    _message: &str,
    _model: &re_agent::model::ModelRef,
    _max_steps: usize,
    _max_tokens: usize,
    _temperature: f64,
) -> Result<String> {
    anyhow::bail!(
        "--with-tools requires building with --features phi (e.g. \
         `cargo run -p re-agent --features phi -- send --with-tools ...`)"
    )
}
