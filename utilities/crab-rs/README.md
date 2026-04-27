# 🦀 crab-rs

**crab-rs** is the high-performance scanning and automation utility for the **Proj.re-rust** Sovereignty Stack. 

It is designed to be a "Swiss Army Knife" for repository analysis, asset downloading, and automated task execution. It follows the Unix philosophy: do one thing well, and work seamlessly with pipes.

## 🚀 Key Modules

-   **Scanner:** Recursively analyzes directory structures and identifies project types, dependencies, and potential issues.
-   **Downloader:** A resilient, multi-threaded asset downloader for pulling models, weights, and external resources.
-   **Mapper:** Generates architectural maps of codebases (used to build the context for `re-agent`).
-   **Runner:** A lightweight task execution engine for running scripts and binaries across the stack.

## 🛠️ Usage

### Scan a directory
```bash
cargo run -p crab-rs -- scan --path .
```

## 📜 License
Part of the Proj.re-rust stack. Licensed under **AGPL-3.0**. 

---
*Syed Ismaeel — Lucknow, Est. 2019*
