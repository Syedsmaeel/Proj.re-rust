# openclaw-patched

OpenClaw v2026.4.24 — patched by crab-rs to remove native C dependencies.

## What was patched

| Dependency | Problem | Fix |
|---|---|---|
| `sharp` | Native C image lib — fails to compile in restricted envs | Stubbed with pure-JS fallback (resize/metadata no-ops) |
| `sqlite-vec` | Native SQLite extension — requires binary compilation | Graceful degradation (vector recall disabled, rest works) |
| `@lydell/node-pty` | Native PTY bindings | Stubbed — PTY unavailable, terminal UI falls back |

## Run it

```bash
npm install --ignore-scripts
node openclaw.mjs --version
node openclaw.mjs --help
node openclaw.mjs configure
```

## What works without native deps

- All CLI commands
- All agent/channel configuration
- Gateway daemon
- MCP integrations
- Messaging (Telegram, Discord, Slack, WhatsApp)
- Web browsing, file ops, shell execution
- Cron jobs, reminders, briefings

## What is degraded

- Image resizing/conversion (stub returns original buffer)
- Vector memory recall (sqlite-vec disabled — semantic search degraded)
- PTY terminal UI (falls back to basic chat mode)

AGPL-3.0 — Syed Ismaeel, Lucknow Est. 2019
