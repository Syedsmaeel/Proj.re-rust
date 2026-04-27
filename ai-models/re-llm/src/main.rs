use anyhow::Result;
use clap::{Parser, Subcommand};
use re_llm::{GenerationConfig, PhiBackend, PhiVariant, GgufBackend};
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
        /// Path to a local GGUF file (from qual-sea)
        #[arg(long)]
        gguf: Option<String>,
        /// Path to tokenizer.json (required if using --gguf)
        #[arg(long)]
        tokenizer: Option<String>,
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
        Cmd::Chat { prompt, gguf, tokenizer, system, variant, max_tokens, temperature, top_p, seed } => {
            if let Some(path) = gguf {
                let tok_path = tokenizer.expect("Please provide --tokenizer path when using --gguf");
                println!("🌊 re-llm — Loading local GGUF model via qual-sea engine...");
                let mut backend = GgufBackend::load(&path, &tok_path)?;
                backend.generate_streaming(&prompt, max_tokens, temperature, |token| {
                    print!("{token}");
                    stdout().flush()?;
                    Ok(())
                })?;
            } else {
                let variant = PhiVariant::parse(&variant);
                let mut backend = PhiBackend::load(variant)?;
                let cfg = GenerationConfig { max_tokens, temperature, top_p: Some(top_p), seed };
                backend.generate_streaming(system.as_deref(), &prompt, &cfg, |token| {
                    print!("{token}");
                    stdout().flush()?;
                    Ok(())
                })?;
            }
            println!();
        }
        Cmd::Models => {
            println!("Supported: Phi-3 (Native HF or Local GGUF via qual-sea)");
        }
    }
    Ok(())
}
