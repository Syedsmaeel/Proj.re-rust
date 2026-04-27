//! Known translation table: npm/pipx tool name → Rust equivalent

use crate::RustEquivalent;
use std::collections::HashMap;

/// Returns the full known translation map
pub fn known_translations() -> HashMap<&'static str, RustEquivalent> {
    let mut map = HashMap::new();

    // --- NPM TOOLS ---
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
        description: "Typed compiler — replaces tsc".into(),
    });
    map.insert("ts-node", RustEquivalent {
        crate_name: "cargo-script".into(),
        binary: "cargo-script".into(),
        install_cmd: "cargo install cargo-script".into(),
        description: "Run scripts directly — replaces ts-node".into(),
    });
    map.insert("nodemon", RustEquivalent {
        crate_name: "cargo-watch".into(),
        binary: "cargo-watch".into(),
        install_cmd: "cargo install cargo-watch".into(),
        description: "File watcher + rerun — replaces nodemon".into(),
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
        crate_name: "cargo-env".into(),
        binary: "cargo".into(),
        install_cmd: "built into cargo".into(),
        description: "Cross-platform env vars — use cargo directly".into(),
    });

    // --- PIPX TOOLS ---
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
        description: "Fast linter — replaces ruff (ruff itself is Rust, clippy for Rust code)".into(),
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
