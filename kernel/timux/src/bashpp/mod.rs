//! Bash++ — Sovereign TUI Shell
//!
//! A ring-2 sub-kernel service inside Timux.
//!
//! ## Architecture
//!
//! ```
//! input → lexer → parser → AST → runtime → TUI renderer
//!                                    ↕
//!                              compat layer (bash scripts)
//! ```
//!
//! ## Features
//! - Own shell language (Bash++ syntax)
//! - Bash compatibility layer (.sh scripts run unmodified)
//! - Full TUI: tabs, split panes, panels, icons, colors, borders
//! - Capability-gated — runs as ring-2 SystemService
//! - Pluggable widgets: file manager, process viewer, editor, clock

pub mod compat;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod tui;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, Ast, Command};
pub use runtime::{Runtime, Env, ExitCode};
pub use tui::{TuiShell, Pane, Tab, Widget};
