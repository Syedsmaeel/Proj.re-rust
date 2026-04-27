# Phi-3 + OpenClaw (Rust port)

Two crates, one pipeline: an OpenClaw-shaped agent talking to a local Phi-3
model with no Python and no remote API.

```
                ┌───────────────────────────┐
   user msg ──▶ │  re-agent  (OpenClaw → RS)│
                │  • gateway / channel       │
                │  • session store           │
                │  • model fallback          │
                └─────────────┬──────────────┘
                              │ run_live()
                              ▼
                ┌───────────────────────────┐
                │  re-llm                    │
                │  • candle (pure Rust ML)   │
                │  • Phi-3 inference         │
                │  • HF cache via hf-hub     │
                └───────────────────────────┘
```

## Why this combo

- **OpenClaw** (`openclaw/openclaw`, MIT, TypeScript) is the architecture
  we're translating into Rust under AGPL-3.0. The shape — gateway, channel
  abstraction, session store, model fallback — survives; the runtime changes.
- **Phi-3-mini** (Microsoft, MIT-licensed open weights) is small enough to
  run on a laptop CPU and capable enough to be useful as the agent's brain.
- **candle** is Hugging Face's Rust ML framework. No PyTorch, no Python
  interpreter, no `node_modules`. Single static binary, the way the rest of
  Proj.re-rust works.

## Crates

| Crate | Path | Role |
|---|---|---|
| `re-llm` | `crates/ai/re-llm` | Loads Phi-3 weights, runs autoregressive generation. CLI + library. |
| `re-agent` | `crates/ai/re-agent` | OpenClaw architecture in Rust. Calls `re-llm` for replies when built with the `phi` feature. |

## Quick start

```bash
# Standalone Phi-3 inference (downloads ~7 GB on first run into the HF cache)
cargo run -p re-llm --release -- chat "Explain the AGPL in two sentences."

# Full agent loop (gateway → channel → session → re-llm → reply)
cargo run -p re-agent --features phi --release -- send --live \
    --model local/phi-3-mini-4k \
    "Plan a 3-step deploy for a Rust binary."
```

The default `cargo build -p re-agent` does **not** pull in candle / hf-hub —
the `phi` feature gates that whole stack so the dry-run agent stays cheap to
build and test.

## Supported variants

| Variant flag | HF repo | Params | Context |
|---|---|---|---|
| `mini-4k`   *(default)* | `microsoft/Phi-3-mini-4k-instruct`     | 3.8B | 4k    |
| `mini-128k`             | `microsoft/Phi-3-mini-128k-instruct`   | 3.8B | 128k  |
| `medium-4k`             | `microsoft/Phi-3-medium-4k-instruct`   | 14B  | 4k    |

Backends: CPU by default. GPU (CUDA / Metal) requires enabling the matching
feature on the `candle-core` dependency in `re-llm/Cargo.toml`.

## What's translated vs. what's still TS

Already in Rust (skeletons + this Phi backend):

- gateway / inbound routing / session keys
- channel abstraction (CLI, Telegram, Discord, Slack, …)
- session store + transcript
- model fallback chain (transient vs fatal error classification)
- silent-reply tokens, markdown capability per channel

Still upstream-only (TypeScript) — porting next:

- skill registry (`clawhub`)
- channel adapters' actual transports (the wire protocols)
- Control UI
- daemon / cron scheduling
- approval gateway

## License notes

OpenClaw upstream is MIT. This port is AGPL-3.0. The architecture and module
boundaries are reproduced; no upstream source is copied verbatim. Improvements
to this port flow back under AGPL-3.0 — see the repo `LICENSE`.
