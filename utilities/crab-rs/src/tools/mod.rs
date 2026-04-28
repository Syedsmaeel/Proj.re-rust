//! Known translation table: npm/pipx tool name → Rust equivalent
//!
//! Priority: Proj.re-rust own crates first, then external Rust tools.

use crate::RustEquivalent;
use std::collections::HashMap;

/// Returns the full known translation map
pub fn known_translations() -> HashMap<&'static str, RustEquivalent> {
    let mut map = HashMap::new();

    // =========================================================================
    // PROJ.RE-RUST OWN CRATES — highest priority
    // =========================================================================

    // react / react-dom → sushi (React-Fiber reconciler in Rust)
    map.insert("react", RustEquivalent {
        crate_name: "sushi".into(),
        binary: "sushi".into(),
        install_cmd: "cargo build -p sushi (Proj.re-rust)".into(),
        description: "React-Fiber reconciler in Rust with use_state, use_effect, TUI host — replaces react".into(),
    });
    map.insert("react-dom", RustEquivalent {
        crate_name: "sushi".into(),
        binary: "sushi".into(),
        install_cmd: "cargo build -p sushi (Proj.re-rust)".into(),
        description: "React-Fiber TUI renderer in Rust — replaces react-dom".into(),
    });
    map.insert("react-icons", RustEquivalent {
        crate_name: "sushi".into(),
        binary: "sushi".into(),
        install_cmd: "cargo build -p sushi (Proj.re-rust)".into(),
        description: "TUI icon/widget system — replaces react-icons".into(),
    });

    // nodemon / ts-node / tsx → re-run (hot-reload Rust runner)
    map.insert("nodemon", RustEquivalent {
        crate_name: "re-run".into(),
        binary: "re-run".into(),
        install_cmd: "cargo build -p re-run (Proj.re-rust)".into(),
        description: "Hot-reload file watcher + runner — replaces nodemon".into(),
    });
    map.insert("ts-node", RustEquivalent {
        crate_name: "re-run".into(),
        binary: "re-run".into(),
        install_cmd: "cargo build -p re-run (Proj.re-rust)".into(),
        description: "Direct script runner with hot reload — replaces ts-node".into(),
    });
    map.insert("tsx", RustEquivalent {
        crate_name: "re-run".into(),
        binary: "re-run".into(),
        install_cmd: "cargo build -p re-run (Proj.re-rust)".into(),
        description: "Fast script executor with watch mode — replaces tsx".into(),
    });

    // @google/gemini-cli → re-agent (agentic task runner)
    map.insert("@google/gemini-cli", RustEquivalent {
        crate_name: "re-agent".into(),
        binary: "re-agent".into(),
        install_cmd: "cargo build -p re-agent (Proj.re-rust)".into(),
        description: "Rust-native agentic CLI — replaces gemini-cli".into(),
    });
    map.insert("gemini", RustEquivalent {
        crate_name: "re-agent".into(),
        binary: "re-agent".into(),
        install_cmd: "cargo build -p re-agent (Proj.re-rust)".into(),
        description: "Rust-native agentic CLI — replaces gemini".into(),
    });

    // openclaw → re-agent
    map.insert("openclaw", RustEquivalent {
        crate_name: "re-agent".into(),
        binary: "re-agent".into(),
        install_cmd: "cargo build -p re-agent (Proj.re-rust)".into(),
        description: "Sovereign agentic runtime — replaces openclaw".into(),
    });

    // pptxgenjs / docx → re-pack (binary packager/bundler)
    map.insert("pptxgenjs", RustEquivalent {
        crate_name: "re-pack".into(),
        binary: "re-pack".into(),
        install_cmd: "cargo build -p re-pack (Proj.re-rust)".into(),
        description: "Document packager — replaces pptxgenjs".into(),
    });
    map.insert("docx", RustEquivalent {
        crate_name: "re-pack".into(),
        binary: "re-pack".into(),
        install_cmd: "cargo build -p re-pack (Proj.re-rust)".into(),
        description: "Document builder — replaces docx".into(),
    });

    // markdownlint / markdownlint-cli2 → re-lint
    map.insert("markdownlint-cli", RustEquivalent {
        crate_name: "re-lint".into(),
        binary: "re-lint".into(),
        install_cmd: "cargo build -p re-lint (Proj.re-rust)".into(),
        description: "Language-agnostic linter — replaces markdownlint-cli".into(),
    });
    map.insert("markdownlint-cli2", RustEquivalent {
        crate_name: "re-lint".into(),
        binary: "re-lint".into(),
        install_cmd: "cargo build -p re-lint (Proj.re-rust)".into(),
        description: "Language-agnostic linter — replaces markdownlint-cli2".into(),
    });

    // playwright → re-scrape
    map.insert("playwright", RustEquivalent {
        crate_name: "re-scrape".into(),
        binary: "re-scrape".into(),
        install_cmd: "cargo build -p re-scrape (Proj.re-rust)".into(),
        description: "Rust-native web scraper — replaces playwright".into(),
    });

    // sharp → re-llm image pipeline (candle-based)
    map.insert("sharp", RustEquivalent {
        crate_name: "re-llm".into(),
        binary: "re-llm".into(),
        install_cmd: "cargo build -p re-llm (Proj.re-rust)".into(),
        description: "Candle-based image/tensor pipeline — replaces sharp".into(),
    });

    // =========================================================================
    // NPM TOOLS → EXTERNAL RUST EQUIVALENTS
    // =========================================================================

    map.insert("prettier", RustEquivalent {
        crate_name: "dprint".into(),
        binary: "dprint".into(),
        install_cmd: "cargo install dprint".into(),
        description: "Code formatter — replaces prettier".into(),
    });
    map.insert("eslint", RustEquivalent {
        crate_name: "clippy".into(),
        binary: "cargo clippy".into(),
        install_cmd: "rustup component add clippy".into(),
        description: "Linter — replaces eslint".into(),
    });
    map.insert("typescript", RustEquivalent {
        crate_name: "rustc".into(),
        binary: "rustc".into(),
        install_cmd: "rustup install stable".into(),
        description: "Typed compiler — replaces typescript/tsc".into(),
    });
    map.insert("http-server", RustEquivalent {
        crate_name: "miniserve".into(),
        binary: "miniserve".into(),
        install_cmd: "cargo install miniserve".into(),
        description: "Static file server — replaces http-server".into(),
    });
    map.insert("serve", RustEquivalent {
        crate_name: "miniserve".into(),
        binary: "miniserve".into(),
        install_cmd: "cargo install miniserve".into(),
        description: "Static file server — replaces serve".into(),
    });
    map.insert("concurrently", RustEquivalent {
        crate_name: "cargo-make".into(),
        binary: "cargo-make".into(),
        install_cmd: "cargo install cargo-make".into(),
        description: "Parallel task runner — replaces concurrently".into(),
    });
    map.insert("rimraf", RustEquivalent {
        crate_name: "rm-improved".into(),
        binary: "rip".into(),
        install_cmd: "cargo install rm-improved".into(),
        description: "Safe recursive delete — replaces rimraf".into(),
    });
    map.insert("cross-env", RustEquivalent {
        crate_name: "cargo".into(),
        binary: "cargo".into(),
        install_cmd: "built into cargo".into(),
        description: "Cross-platform env vars — use cargo directly".into(),
    });
    map.insert("markdown-toc", RustEquivalent {
        crate_name: "mdbook".into(),
        binary: "mdbook".into(),
        install_cmd: "cargo install mdbook".into(),
        description: "Markdown TOC + book builder — replaces markdown-toc".into(),
    });
    map.insert("marked", RustEquivalent {
        crate_name: "pulldown-cmark".into(),
        binary: "pulldown-cmark".into(),
        install_cmd: "cargo install pulldown-cmark".into(),
        description: "Markdown parser/renderer — replaces marked".into(),
    });
    map.insert("remark-cli", RustEquivalent {
        crate_name: "mdbook".into(),
        binary: "mdbook".into(),
        install_cmd: "cargo install mdbook".into(),
        description: "Markdown processor — replaces remark-cli".into(),
    });
    map.insert("markdown-pdf", RustEquivalent {
        crate_name: "mdbook".into(),
        binary: "mdbook".into(),
        install_cmd: "cargo install mdbook".into(),
        description: "Markdown to PDF — replaces markdown-pdf".into(),
    });
    map.insert("graphviz", RustEquivalent {
        crate_name: "layout-rs".into(),
        binary: "layout".into(),
        install_cmd: "cargo install layout-rs".into(),
        description: "Graph layout engine — replaces graphviz".into(),
    });
    map.insert("@mermaid-js/mermaid-cli", RustEquivalent {
        crate_name: "mermaid-rs".into(),
        binary: "mermaid".into(),
        install_cmd: "cargo install mermaid-rs".into(),
        description: "Diagram renderer — replaces mermaid-cli".into(),
    });

    // =========================================================================
    // PIPX TOOLS → EXTERNAL RUST EQUIVALENTS
    // =========================================================================

    map.insert("black", RustEquivalent {
        crate_name: "dprint".into(),
        binary: "dprint".into(),
        install_cmd: "cargo install dprint".into(),
        description: "Code formatter — replaces black".into(),
    });
    map.insert("httpie", RustEquivalent {
        crate_name: "xh".into(),
        binary: "xh".into(),
        install_cmd: "cargo install xh".into(),
        description: "HTTP client — replaces httpie".into(),
    });
    map.insert("http", RustEquivalent {
        crate_name: "xh".into(),
        binary: "xh".into(),
        install_cmd: "cargo install xh".into(),
        description: "HTTP client — replaces http (httpie)".into(),
    });
    map.insert("poetry", RustEquivalent {
        crate_name: "cargo".into(),
        binary: "cargo".into(),
        install_cmd: "rustup install stable".into(),
        description: "Dependency manager — cargo is the equivalent".into(),
    });
    map.insert("mypy", RustEquivalent {
        crate_name: "rustc".into(),
        binary: "rustc".into(),
        install_cmd: "rustup install stable".into(),
        description: "Type checker — rustc does this natively".into(),
    });
    map.insert("ruff", RustEquivalent {
        crate_name: "clippy".into(),
        binary: "cargo clippy".into(),
        install_cmd: "rustup component add clippy".into(),
        description: "Fast linter — replaces ruff".into(),
    });
    map.insert("twine", RustEquivalent {
        crate_name: "cargo-publish".into(),
        binary: "cargo publish".into(),
        install_cmd: "built into cargo".into(),
        description: "Package publisher — replaces twine".into(),
    });
    map.insert("cookiecutter", RustEquivalent {
        crate_name: "cargo-generate".into(),
        binary: "cargo-generate".into(),
        install_cmd: "cargo install cargo-generate".into(),
        description: "Project scaffolder — replaces cookiecutter".into(),
    });

    map
}
