//! re-llm CLI — local inference for Phi-3.
//!
//! Examples:
//!   re-llm chat "Explain the AGPL in two sentences."
//!   re-llm chat --variant mini-128k --max-tokens 512 "Summarise this..."
//!   re-llm models

use anyhow::Result;
use clap::{Parser, Subcommand};
use re_llm::{GenerationConfig, PhiBackend, PhiVariant};

#[derive(Parser)]
#[command(
    name = "re-llm",
    about = "Local LLM inference (Phi-3 via candle)",
    version,
    author = "Syed Ismaeel — Lucknow, Est. 2019"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a single prompt through Phi-3
    Chat {
        /// User message
        prompt: String,
        /// Model variant: mini-4k (default), mini-128k, medium-4k
        #[arg(long, default_value = "mini-4k")]
        variant: String,
        /// Max tokens to generate
        #[arg(long, default_value_t = 256)]
        max_tokens: usize,
        /// Sampling temperature (0.0 = greedy)
        #[arg(long, default_value_t = 0.7)]
        temperature: f64,
        /// Nucleus sampling cutoff
        #[arg(long, default_value_t = 0.9)]
        top_p: f64,
        /// RNG seed
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    /// List supported model variants
    Models,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Chat {
            prompt,
            variant,
            max_tokens,
            temperature,
            top_p,
            seed,
        } => {
            let variant = PhiVariant::parse(&variant);
            eprintln!(
                "🦀 re-llm — loading {} (first run downloads weights into HF cache)…",
                variant.repo_id()
            );

            let mut backend = PhiBackend::load(variant)?;
            let cfg = GenerationConfig {
                max_tokens,
                temperature,
                top_p: Some(top_p),
                seed,
            };

            let out = backend.generate(&prompt, &cfg)?;
            println!("{out}");
        }

        Cmd::Models => {
            println!("Supported Phi-3 variants:");
            println!("  mini-4k     → microsoft/Phi-3-mini-4k-instruct      [default]");
            println!("  mini-128k   → microsoft/Phi-3-mini-128k-instruct");
            println!("  medium-4k   → microsoft/Phi-3-medium-4k-instruct");
        }
    }

    Ok(())
}
