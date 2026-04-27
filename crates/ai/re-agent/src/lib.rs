//! re-agent: Rust-native agentic runner
//! Translated from OpenClaw (openclaw/openclaw) — MIT licensed source
//! Re-implemented in Rust under AGPL-3.0 by Syed Ismaeel, Lucknow Est. 2019
//!
//! Architecture mirrors OpenClaw's core:
//! - gateway  : message routing + client session management
//! - agent    : agent runner, fallback logic, execution context
//! - channel  : message channel abstraction (CLI, Telegram, Discord, etc.)
//! - model    : model selection, auth profiles, fallback chains
//! - session  : session store, transcript, memory flush
//! - config   : config loading, workspace resolution

pub mod agent;
pub mod channel;
pub mod config;
pub mod gateway;
pub mod model;
pub mod session;
