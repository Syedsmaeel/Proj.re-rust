use axum::{routing::{get, post}, Json, Router, http::StatusCode};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::process::Command;

pub mod models;
pub mod orchestrator;
pub mod claude_interop;
pub mod grep;
pub mod translator;

use models::{StackConfig, ServiceConfig};
use orchestrator::Orchestrator;
use claude_interop::TENGU_SYSTEM_PROMPT;
use grep::GrepEngine;
use translator::Translator;

#[derive(Deserialize)]
struct TranslateRequest {
    code: String,
    language: String, // "js" or "python"
}

async fn run_translate(Json(payload): Json<TranslateRequest>) -> Json<OrchestrateResponse> {
    let output = match payload.language.as_str() {
        "js" => Translator::parse_js(&payload.code),
        "python" => Translator::parse_python(&payload.code),
        _ => "Unsupported language".to_string(),
    };

    Json(OrchestrateResponse {
        status: "success".to_string(),
        output,
    })
}

#[derive(Deserialize)]
struct OrchestrateRequest {
    task: String,
    language: String,
    transpile: bool,
}

#[derive(Serialize)]
struct OrchestrateResponse {
    status: String,
    output: String,
}
async fn run_task(Json(payload): Json<OrchestrateRequest>) -> Result<Json<OrchestrateResponse>, StatusCode> {
    // SECURITY PATCH: RCE vulnerability mitigated. 
    // Raw command execution is removed. Only allowlisted commands are permitted.
    let allowed_commands = ["git status", "ls -la", "cargo check"];

    if !allowed_commands.contains(&payload.task.as_str()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(&payload.task)
        .output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(OrchestrateResponse {
        status: "completed".to_string(),
        output: String::from_utf8_lossy(&output.stdout).to_string(),
    }))
}

async fn stack_up(Json(payload): Json<StackConfig>) -> Result<Json<OrchestrateResponse>, StatusCode> {
    let orch = Orchestrator::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let network_name = format!("{}_network", payload.name);
    let _ = orch.create_network(&network_name).await;

    for (name, config) in &payload.services {
        orch.start_service(name, config, Some(&network_name)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(OrchestrateResponse {
        status: "success".to_string(),
        output: format!("Stack '{}' is up on network '{}'.", payload.name, network_name),
    }))
}

#[derive(Deserialize)]
struct StackDownRequest {
    name: String,
    services: Vec<String>,
}

async fn stack_down(Json(payload): Json<StackDownRequest>) -> Result<Json<OrchestrateResponse>, StatusCode> {
    let orch = Orchestrator::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for name in &payload.services {
        let _ = orch.stop_service(name).await;
    }
    let network_name = format!("{}_network", payload.name);
    let _ = orch.remove_network(&network_name).await;
    Ok(Json(OrchestrateResponse {
        status: "success".to_string(),
        output: format!("Stack '{}' is down.", payload.name),
    }))
}

#[derive(Deserialize)]
struct SearchRequest {
    pattern: String,
    path: String,
}

async fn run_search(Json(payload): Json<SearchRequest>) -> Json<Vec<String>> {
    let results = GrepEngine::search(&payload.pattern, &payload.path);
    Json(results)
}

#[derive(Deserialize)]
struct ThinkRequest {
    prompt: String,
}

#[derive(Serialize)]
struct ThinkResponse {
    system_prompt: String,
    agent_instruction: String,
}

async fn claude_think(Json(payload): Json<ThinkRequest>) -> Json<ThinkResponse> {
    Json(ThinkResponse {
        system_prompt: TENGU_SYSTEM_PROMPT.to_string(),
        agent_instruction: format!("Plan for: {}", payload.prompt),
    })
}

#[tokio::main]
async fn main() {
    let public_dir = PathBuf::from("public");
    
    let app = Router::new()
        .fallback_service(ServeDir::new(public_dir))
        .route("/api/posts", get(|| async { "[]" }))
        .route("/api/auth", get(|| async { "{}" }))
        .route("/api/dashboard", get(|| async { "{}" }))
        .route("/api/orchestrate/run-task", post(run_task))
        .route("/api/orchestrate/up", post(stack_up))
        .route("/api/orchestrate/down", post(stack_down))
        .route("/api/search", post(run_search))
        .route("/api/claude/think", post(claude_think))
        .route("/api/translate", post(run_translate));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Unified Orchestrator listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
