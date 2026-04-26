# Project: Crab-SCM (Unified Rust VCS)

## Mission
Build a memory-safe, high-performance VCS in Rust that bridges Git and Mercurial ecosystems without relying on unstable Python-based extensions.

## Core Objectives
1.  **Architecture:** Implement a Rust-native object store that abstracts Git/Hg structures into a common DAG model.
2.  **Compatibility:** Achieve native read/write support for existing Git/Mercurial repositories.
3.  **Stability:** Eliminate the "internal API dependency" issues by implementing core VCS logic in pure, dependency-checked Rust.
4.  **Performance:** Leverage Rust's concurrency and memory safety to outperform current Python-based VCS extensions.

## Phase 1: Architectural Foundation
- Define the core Object Data Model (Commit, Tree, Blob).
- Implement a parser for the Git/Mercurial object format.
- Create a storage backend (Sled or similar) to manage the local object database.

## Phase 2: Interop Layer
- Port the functionality of the `hg-git` bridge (which currently relies on broken `mercurial.bundle2` internals).
- Replace the Python-based API coupling with direct Rust-to-Repository interaction.

## Phase 3: CLI & CLI Orchestration
- Build the `crab-scm` CLI (mirroring `hg` and `git` commands).
- Implement high-level operations (`init`, `commit`, `push`, `pull`).

## Next Steps
1. Create the `crab-scm` repository structure in `UnifiedPlatform/packages/`.
2. Analyze the Mercurial `bundle2` specification to define our native Rust parser.
3. Establish a baseline object model (DAG).
