use anyhow::Result;
use clap::{Parser, Subcommand};
use re_llm::{GenerationConfig, PhiBackend, PhiVariant};
use std::io::{Write, stdout};

#[derive(Parser)]
#[command(name = "re-llm", about = "Local LLM inference (Phi-3)", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Chat {
        prompt: String,
        /// System instruction (e.g. 'You are a pirate')
        #[arg(long)]
        system: Option<String>,
        #[arg(long, default_value = "mini-4k")]
        variant: String,
        #[arg(long, default_value_t = 512)]
        max_tokens: usize,
        #[arg(long, default_value_t = 0.7)]
        temperature: f64,
        #[arg(long, default_value_t = 0.9)]
        top_p: f64,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    Models,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Chat { prompt, system, variant, max_tokens, temperature, top_p, seed } => {
            let variant = PhiVariant::parse(&variant);
            let mut backend = PhiBackend::load(variant)?;
            let cfg = GenerationConfig { max_tokens, temperature, top_p: Some(top_p), seed };

            backend.generate_streaming(system.as_deref(), &prompt, &cfg, |token| {
                print!("{token}");
                stdout().flush()?;
                Ok(())
            })?;
            println!();
        }
        Cmd::Models => {
            println!("Supported Phi-3 variants: mini-4k, mini-128k, medium-4k");
        }
    }
    Ok(())
}
