//! Crab-rs CLI — detects npm/pipx tools and maps them to Rust-native equivalents

use anyhow::Result;
use clap::{Parser, Subcommand};
use crab_rs::{mapper, runner, scanner};

#[derive(Parser)]
#[command(
    name = "crab",
    about = "Detects npm/pipx tools and runs Rust-native equivalents",
    version,
    author = "Syed Ismaeel — Lucknow, Est. 2019"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan system for installed npm and pipx tools
    Scan {
        /// Show only tools with known Rust equivalents
        #[arg(long)]
        known: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show the full translation table
    Table,
    /// Check if Rust equivalents are installed
    Check,
    /// Run a Rust equivalent for a given npm/pipx tool name
    Run {
        /// The npm/pipx tool name to replace (e.g. "prettier", "httpie")
        tool: String,
        /// Arguments to pass to the Rust equivalent
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Scan { known, json } => {
            let tools = scanner::scan_all()?;
            let mut translations = mapper::translate_all(&tools);

            if known {
                translations = mapper::known_only(translations);
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&translations)?);
            } else {
                println!("\n🦀 Crab-rs — Tool Scanner\n");
                if translations.is_empty() {
                    println!("  No npm or pipx tools found.");
                }
                for t in &translations {
                    let source = format!("{:?}", t.foreign.source).to_lowercase();
                    let version = t.foreign.version.as_deref().unwrap_or("?");
                    print!("  [{source}] {} v{version}", t.foreign.name);

                    if let Some(eq) = &t.equivalent {
                        println!(" → {} ({})", eq.binary, eq.crate_name);
                    } else {
                        println!(" → ⚠ no Rust equivalent known");
                    }
                }
                println!();
            }
        }

        Cmd::Table => {
            use crab_rs::tools::known_translations;
            let map = known_translations();
            println!("\n🦀 Crab-rs — Known Translations\n");
            println!("  {:<20} {:<20} {}", "Foreign Tool", "Rust Binary", "Install");
            println!("  {}", "-".repeat(70));
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            for (name, eq) in entries {
                println!("  {:<20} {:<20} {}", name, eq.binary, eq.install_cmd);
            }
            println!();
        }

        Cmd::Check => {
            use crab_rs::tools::known_translations;
            let map = known_translations();
            println!("\n🦀 Crab-rs — Checking installed Rust equivalents\n");
            let mut seen = std::collections::HashSet::new();
            for eq in map.values() {
                if seen.contains(&eq.binary) { continue; }
                seen.insert(eq.binary.clone());
                let installed = runner::is_installed(&eq.binary);
                let status = if installed { "✅" } else { "❌" };
                println!("  {status} {} — {}", eq.binary, eq.install_cmd);
            }
            println!();
        }

        Cmd::Run { tool, args } => {
            use crab_rs::tools::known_translations;
            let map = known_translations();

            match map.get(tool.as_str()) {
                None => {
                    eprintln!("❌ No Rust equivalent known for '{tool}'");
                    eprintln!("   Run `crab table` to see all known translations.");
                    std::process::exit(1);
                }
                Some(eq) => {
                    if !runner::is_installed(&eq.binary) {
                        eprintln!("❌ '{}' is not installed.", eq.binary);
                        runner::print_install_hint(eq);
                        std::process::exit(1);
                    }
                    println!("🦀 Running '{}' instead of '{tool}'", eq.binary);
                    runner::run(eq, &args)?;
                }
            }
        }
    }

    Ok(())
}
