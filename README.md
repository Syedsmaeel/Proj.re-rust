# Proj.re-rust: The Re-Rust Ecosystem

<p align="center">
  <img src="https://www.rust-lang.org/static/images/rust-logo-blk.svg" alt="Rust Logo" width="100" height="100">
  <img src="https://rustacean.net/assets/rustacean-flat-noshadow.svg" alt="Ferris the Crab" width="100" height="100">
  <img src="https://upload.wikimedia.org/wikipedia/commons/3/35/Tux.svg" alt="Tux the Penguin" width="100" height="100">
</p>

**A sovereign, high-performance system architecture built entirely in Rust.**

This repository is a monorepo containing the **Re-Rust** ecosystem—a suite of projects ranging from a custom kernel with a modern privilege model to high-performance AI inference runtimes and specialized development tools.

---

## 🏗 Workspace Architecture

The ecosystem is organized into several functional domains:

### 🛡 Kernel & Core
- **`kernel/timux`**: A sovereign kernel featuring a hybrid ring + capability privilege model. It moves beyond traditional Unix/Windows models to provide fine-grained, capability-gated access control across 5 rings (0-4).
- **`core/re-core`**: The foundational library providing shared types, traits, and logic for the entire ecosystem.

### 🧠 AI & Intelligence
- **`ai-models/re-llm`**: High-performance LLM inference engine built on HuggingFace's **Candle** framework, supporting GGUF and advanced quantization.
- **`ai-models/re-agent`**: Framework for building autonomous agents integrated directly into the Re-Rust runtime.

### 🚀 Runtimes
- **`runtimes/re-node`**: Specialized node runtime for the ecosystem.
- **`runtimes/re-docker`**: Integration layers for containerized Re-Rust environments.
- **`runtimes/re-npm`**: Package management and distribution for Re-Rust modules.

### 🛠 Build Tools
- **`re-make`**: A custom build system tailored for the Re-Rust workspace.
- **`re-ci` / `re-lint`**: Automated quality assurance and continuous integration tools.
- **`re-pack`**: Deployment and distribution packager.

### 🧰 Utilities
- **`re-crypt`**: Cryptographic primitives and security utilities.
- **`re-scrape`**: High-performance data ingestion and scraping engine.
- **`sushi`**: A custom macro-based framework (including `sushi-macros`).
- **`crab-rs` / `qual-sea` / `re-run`**: Specialized utility crates for various system tasks.

---

## 🛠 Getting Started

### Prerequisites
- **Rust**: Version 1.75 or higher is required.
- **Target Toolchains** (for Timux kernel):
  ```bash
  rustup target add x86_64-unknown-none riscv64gc-unknown-none-elf aarch64-unknown-none
  ```

### Building the Workspace
To build all members of the workspace:
```bash
cargo build
```

To build a specific component (e.g., the Timux kernel):
```bash
cargo build -p timux
```

---

## 📜 License
This project is licensed under the **AGPL-3.0**.

**Author:** Syed Ismaeel — Lucknow, Est. 2019
