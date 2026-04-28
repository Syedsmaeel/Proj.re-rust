use anyhow::{Result, Context};
use boa_engine::{Context as BoaContext, Source};
use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(name = "re-node", about = "Sovereign JS Runtime (V8-free)", version)]
struct Cli {
    /// JavaScript file to run
    file: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let code = fs::read_to_string(&cli.file).context("Failed to read JS file")?;

    let mut context = BoaContext::default();
    let mut canvas = sushi::Canvas::new(400, 400);
    runtimes::re_node::bridge::register_sushi_bridge(&mut context, &mut canvas);
    
    println!("🚀 re-node — Executing JS on Rust Engine...");
    
    match context.eval(Source::from_bytes(&code)) {
        Ok(val) => println!("✅ Result: {:?}", val.display()),
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    Ok(())
}
