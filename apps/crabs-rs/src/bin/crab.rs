use clap::{Parser, Subcommand};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServiceConfig {
    pub image: String,
    pub command: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StackConfig {
    pub name: String,
    pub services: HashMap<String, ServiceConfig>,
}

#[derive(Parser)]
#[command(name = "crab", about = "Unified Orchestrator CLI")]
struct Cli {
    #[command(subcommand)]
    manager: Manager,
}

#[derive(Subcommand)]
enum Manager {
    Pipx { #[command(subcommand)] cmd: Action },
    Npm { #[command(subcommand)] cmd: Action },
    Deb { #[command(subcommand)] cmd: Action },
    Stack { #[command(subcommand)] cmd: StackAction },
    CiGen { #[arg(short, long)] file: String },
}

#[derive(Subcommand)]
enum StackAction {
    Up { #[arg(short, long)] file: String },
    Down { #[arg(short, long)] name: String, #[arg(short, long, value_delimiter = ',')] services: Vec<String> },
}

#[derive(Subcommand)]
enum Action {
    Install { package: String },
    Upgrade { package: String },
    Repair { package: String },
    Reinstall { package: String },
    Translate { package: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.manager {
        Manager::Pipx { cmd } => handle_action("pipx", cmd),
        Manager::Npm { cmd } => handle_action("npm", cmd),
        Manager::Deb { cmd } => handle_action("deb", cmd),
        Manager::Stack { cmd } => handle_stack(cmd),
        Manager::CiGen { file } => handle_ci_gen(file),
    }
}

fn handle_stack(action: StackAction) {
    let client = reqwest::blocking::Client::new();
    let url = match action {
        StackAction::Up { .. } => "http://localhost:3000/api/orchestrate/up",
        StackAction::Down { .. } => "http://localhost:3000/api/orchestrate/down",
    };

    let body = match action {
        StackAction::Up { file } => {
            let content = std::fs::read_to_string(&file).expect("Failed to read stack file");
            if file.ends_with(".ron") {
                let config: StackConfig = ron::from_str(&content).expect("Failed to parse RON");
                serde_json::to_string(&config).expect("Failed to serialize to JSON")
            } else {
                content
            }
        },
        StackAction::Down { name, services } => {
            serde_json::to_string(&serde_json::json!({
                "name": name,
                "services": services
            })).unwrap()
        }
    };

    let res = client.post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .expect("Failed to send request to orchestrator");

    println!("Response: {}", res.text().unwrap());
}

fn handle_ci_gen(file: String) {
    let content = std::fs::read_to_string(&file).expect("Failed to read pipeline config");
    let config: crate::ci_gen::PipelineConfig = ron::from_str(&content).expect("Failed to parse RON");
    let yaml = crate::ci_gen::generate_gitlab_ci(&config);
    std::fs::write(".gitlab-ci.yml", yaml).expect("Failed to write .gitlab-ci.yml");
    println!("Successfully generated .gitlab-ci.yml");
}

fn handle_action(manager: &str, action: Action) {
    let (tool, args) = match (manager, action) {
        ("pipx", Action::Install { package }) => ("pipx", vec!["install".to_string(), package]),
        ("pipx", Action::Upgrade { package }) => ("pipx", vec!["upgrade".to_string(), package]),
        ("pipx", Action::Repair { package }) => ("pipx", vec!["repair".to_string(), package]),
        ("pipx", Action::Reinstall { package }) => ("pipx", vec!["reinstall".to_string(), package]),
        ("pipx", Action::Translate { package }) => ("python3", vec!["translate.py".to_string(), package]),

        ("npm", Action::Install { package }) => {
            let mut parts = vec!["install".to_string()];
            if package.contains("-g") {
                parts.insert(0, "-g".to_string());
            } else {
                parts.push(package);
            }
            ("npm", parts)
        },
        ("npm", Action::Upgrade { package }) => ("npm", vec!["update".to_string(), package]),
        ("npm", Action::Repair { package }) => ("npm", vec!["rebuild".to_string(), package]),
        ("npm", Action::Reinstall { package }) => ("npm", vec!["reinstall".to_string(), package]),
        ("npm", Action::Translate { package }) => ("npx", vec!["transpile-script".to_string(), package]),

        ("deb", Action::Install { package }) => ("sudo", vec!["apt-get".to_string(), "install".to_string(), "-y".to_string(), package]),
        ("deb", Action::Upgrade { package }) => ("sudo", vec!["apt-get".to_string(), "install".to_string(), "--only-upgrade".to_string(), "-y".to_string(), package]),
        ("deb", Action::Repair { package }) => ("sudo", vec!["apt-get".to_string(), "install".to_string(), "--reinstall".to_string(), package]),
        ("deb", Action::Reinstall { package }) => ("sudo", vec!["apt-get".to_string(), "install".to_string(), "--reinstall".to_string(), package]),
        ("deb", Action::Translate { package }) => ("echo", vec!["transpiling deb not supported".to_string(), package]),
        _ => return,
    };

    let _ = Command::new(tool).args(args).status();
}
