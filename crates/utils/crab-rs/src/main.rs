//! Crab-rs CLI — detects npm/pipx tools and maps them to Rust-native equivalents

use anyhow::Result;
use clap::{Parser, Subcommand};
use crab_rs::{downloader, mapper, runner, scanner};

#[derive(Parser)]
#[command(
    name = "crab",
    about = "Detect, download, and replace npm/pipx tools with Rust-native equivalents",
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
        #[arg(long)]
        known: bool,
        #[arg(long)]
        json: bool,
    },

    /// Show the full translation table
    Table,

    /// Check if Rust equivalents are installed
    Check,

    /// Run a Rust equivalent for a given npm/pipx tool name
    Run {
        tool: String,
        args: Vec<String>,
    },

    /// Download (install) a package from npm or pipx
    Download {
        #[command(subcommand)]
        subcmd: DownloadCmd,
    },

    /// Uninstall a package from npm or pipx
    Remove {
        #[command(subcommand)]
        subcmd: RemoveCmd,
    },

    /// List installed packages from npm or pipx
    List {
        #[command(subcommand)]
        subcmd: ListCmd,
    },

    /// Show info about a package from npm or pipx registry
    Info {
        #[command(subcommand)]
        subcmd: InfoCmd,
    },
}

#[derive(Subcommand)]
enum DownloadCmd {
    /// Install a package globally via npm
    Npm {
        /// Package name (e.g. prettier)
        package: String,
        /// Optional version (e.g. 3.0.0)
        #[arg(long, short)]
        version: Option<String>,
        /// Also show the Rust equivalent after install
        #[arg(long)]
        show_equivalent: bool,
    },
    /// Install a package via pipx
    Pipx {
        /// Package name (e.g. httpie)
        package: String,
        /// Optional version (e.g. 3.2.1)
        #[arg(long, short)]
        version: Option<String>,
        /// Also show the Rust equivalent after install
        #[arg(long)]
        show_equivalent: bool,
    },
}

#[derive(Subcommand)]
enum RemoveCmd {
    /// Uninstall a global npm package
    Npm { package: String },
    /// Uninstall a pipx package
    Pipx { package: String },
}

#[derive(Subcommand)]
enum ListCmd {
    /// List globally installed npm packages
    Npm,
    /// List pipx installed packages
    Pipx,
    /// List both
    All,
}

#[derive(Subcommand)]
enum InfoCmd {
    /// Show npm registry info for a package
    Npm { package: String },
    /// Show pipx/pip info for a package
    Pipx { package: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // ── SCAN ──────────────────────────────────────────────────────────
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

        // ── TABLE ─────────────────────────────────────────────────────────
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

        // ── CHECK ─────────────────────────────────────────────────────────
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

        // ── RUN ───────────────────────────────────────────────────────────
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

        // ── DOWNLOAD ──────────────────────────────────────────────────────
        Cmd::Download { subcmd } => {
            match subcmd {
                DownloadCmd::Npm { package, version, show_equivalent } => {
                    downloader::npm_install(&package, version.as_deref())?;
                    if show_equivalent {
                        show_rust_equivalent(&package);
                    }
                }
                DownloadCmd::Pipx { package, version, show_equivalent } => {
                    downloader::pipx_install(&package, version.as_deref())?;
                    if show_equivalent {
                        show_rust_equivalent(&package);
                    }
                }
            }
        }

        // ── REMOVE ────────────────────────────────────────────────────────
        Cmd::Remove { subcmd } => {
            match subcmd {
                RemoveCmd::Npm { package } => downloader::npm_uninstall(&package)?,
                RemoveCmd::Pipx { package } => downloader::pipx_uninstall(&package)?,
            }
        }

        // ── LIST ──────────────────────────────────────────────────────────
        Cmd::List { subcmd } => {
            match subcmd {
                ListCmd::Npm => {
                    println!("\n📦 Global npm packages:\n");
                    downloader::npm_list()?;
                }
                ListCmd::Pipx => {
                    println!("\n📦 Pipx packages:\n");
                    downloader::pipx_list()?;
                }
                ListCmd::All => {
                    println!("\n📦 Global npm packages:\n");
                    let _ = downloader::npm_list();
                    println!("\n📦 Pipx packages:\n");
                    let _ = downloader::pipx_list();
                }
            }
        }

        // ── INFO ──────────────────────────────────────────────────────────
        Cmd::Info { subcmd } => {
            match subcmd {
                InfoCmd::Npm { package } => downloader::npm_info(&package)?,
                InfoCmd::Pipx { package } => downloader::pipx_info(&package)?,
            }
        }
    }

    Ok(())
}

/// Print the Rust equivalent of a tool after installing it
fn show_rust_equivalent(package: &str) {
    use crab_rs::tools::known_translations;
    let map = known_translations();
    if let Some(eq) = map.get(package) {
        println!("\n💡 Rust equivalent available:");
        println!("   {} → {}", package, eq.binary);
        println!("   {}", eq.description);
        println!("   Install with: {}", eq.install_cmd);
        println!("   Then use: crab run {}", package);
    }
}
