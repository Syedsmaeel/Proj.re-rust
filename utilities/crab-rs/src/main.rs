use anyhow::Result;
use clap::{Parser, Subcommand};
use crab_rs::scanner::WorkspaceScanner;

#[derive(Parser)]
#[command(name = "crab-rs", about = "Connective Tissue for Sovereignty Stack", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan current workspace for projects (Rust, Node, Python)
    Scan {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Download a remote asset (e.g. model weights)
    Get {
        url: String,
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Scan { path } => {
            println!("🦀 crab-rs — Scanning Workspace: {}", path);
            let scanner = WorkspaceScanner::new(&path);
            let projects = scanner.scan_projects();
            for p in projects { println!("  • {}", p); }
        }
        Cmd::Get { url, output } => {
            crab_rs::downloader::Downloader::download(&url, &output)?;
        }
    }
    Ok(())
}
