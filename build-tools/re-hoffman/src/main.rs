use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "hoffman")]
#[command(about = "Hoffman Script 2.0 — Sovereign OS-as-Code Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Grows a sovereign OS from a grass.hoffman blueprint
    Grow {
        /// Path to the grass.hoffman file
        #[arg(short, long, default_value = "grass.hoffman")]
        grass: PathBuf,

        /// Output target (iso, img, or tmx)
        #[arg(short, long, default_value = "tmx")]
        target: String,
    },
    /// Verifies the integrity of a grass.lock file
    Verify {
        #[arg(short, long, default_value = "grass.lock")]
        lock: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Grow { grass, target } => {
            println!("🌱 Growing sovereign OS from {:?} into {} format...", grass, target);
            // Evaluation logic will go here
        }
        Commands::Verify { lock } => {
            println!("🛡️ Verifying integrity of {:?}...", lock);
            // Integrity logic will go here
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::Bootstrap;

    #[test]
    fn test_grafting() {
        let kernel = include_bytes!("../dummy_kernel.bin");
        let bootstrap = Bootstrap::new();
        let image = bootstrap.seed(kernel);
        assert_eq!(image.len(), 1024);
        println!("󰒋 Sovereign Grafting Successful!");
    }
}
