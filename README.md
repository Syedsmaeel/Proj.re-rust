# Proj.re-rust

**The infrastructure you use is not neutral. Node.js reports home. Docker extracts rent. Build pipelines rot under corporate licensing. This project builds the replacements — in Rust, owned outright, AGPL-3.0.**

> *Every tool here exists because the original was either too slow, too bloated, or owned by someone who doesn't share your interests.*

**Author:** Syed Ismaeel — Lucknow, Est. 2019  
**License:** AGPL-3.0 — if you improve it, the improvement belongs to everyone  
**Language:** Rust (2021 edition)  
**Structure:** Single Cargo workspace — one repo, multiple crates

---

## What This Is

`Proj.re-rust` is a **Rust-native sovereignty stack**: a collection of drop-in rewrites of tools that developers depend on but don't control, plus original AI utilities and utility boxes built on top.

Three pillars:

| Pillar | What it kills | Status |
|---|---|---|
| **Runtime Replacements** | Node.js, Docker, npm, POSIX shell | In progress |
| **Build Tools** | Make, CI runners, linters, bundlers | Planned |
| **AI + Utility Boxes** | LLM inference, scraping, encryption, agentic runners | Planned |

---

## Repository Structure

```
Proj.re-rust/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── re-core/            # Shared primitives: config, logging, error handling, IPC, CLI
│   │
│   ├── runtime/
│   │   ├── re-node/        # JavaScript runtime (V8-free, Deno-inspired but leaner)
│   │   ├── re-docker/      # OCI-compatible container engine
│   │   ├── re-npm/         # Package registry client
│   │   └── re-shell/       # POSIX shell implementation
│   │
│   ├── build/
│   │   ├── re-make/        # Task runner — kills Makefile complexity
│   │   ├── re-ci/          # Local-first CI pipeline engine
│   │   ├── re-lint/        # Language-agnostic linter
│   │   └── re-pack/        # Binary packager and bundler
│   │
│   └── ai/
│       ├── re-llm/         # AI model inference CLI (local + API)
│       ├── re-scrape/      # Web scraper and parser
│       ├── re-crypt/       # Encryption utility box
│       └── re-agent/       # Agentic task runner
│
├── docs/                   # Architecture decisions, protocol specs
├── tests/                  # Integration tests (cross-crate)
└── scripts/                # Dev tooling, release automation
```

---

## Design Principles

**1. No hidden dependencies.**  
Every crate declares what it needs. Nothing phones home. Nothing pulls in a 400-package tree to `console.log`.

**2. Single binary per tool.**  
Each crate compiles to a standalone binary. No runtime. No interpreter. No JVM.

**3. Composable by default.**  
Every tool reads from stdin, writes to stdout, exits with a meaningful code. They chain. They script. They don't require a GUI.

**4. AGPL-3.0 or nothing.**  
Improvements must flow back. This is non-negotiable. If you build on it, it stays open.

---

## Getting Started

```bash
# Clone
git clone https://github.com/Syedsmaeel/Proj.re-rust
cd Proj.re-rust

# Build everything
cargo build --workspace

# Build a specific crate
cargo build -p re-node

# Run tests
cargo test --workspace
```
**Requirements:** Rust 1.75+ (stable), Cargo

## Contributing

This is a solo research project. External contributions are welcome but must:

- Pass `cargo clippy -- -D warnings`
- Include tests
- Not introduce dependencies without documented justification
- Remain AGPL-3.0 compatible

---

## Why Rust

Because memory safety is not a feature — it's the floor. Because the binary ships without a runtime. Because `cargo` is the build system that doesn't require a PhD. And because the tools that will outlast the current infrastructure cycle will be written in languages that don't garbage-collect at the wrong moment.

---

*Lucknow, Est. 2019 — Cipher Nazr*
