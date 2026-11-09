# AGENTS.md

## Quick Start

- **Build**: `make build` (release binary) or `cargo build`
- **Check**: `make check` (`cargo check`)
- **Test**: `make test` (`cargo test`) — runs integration and unit tests
- **Full gate**: `make all` (check + test + build)
- **Run CLI**: `make run` (`cargo run -- --help`)
- **Daemon**: `make daemon` (live, requires `SWAYSOCK`) or `make daemon-dry` (dry-run, logs commands, offline safe)
- **Reload**: `make reload` (`swaymsg reload`)
- **Clean**: `make clean`

## Find Things

- `src/` — Rust crate source
  - `main.rs` — binary entry point
  - `lib.rs` — module root (`cli`, `config`, `daemon`, `layout`, `sway`, `types`)
  - `cli.rs` — argument parsing (clap)
  - `config.rs` — TOML config loading (`~/.config/swayve/config.toml`)
  - `daemon.rs` — background daemon and IPC
  - `layout.rs` — window/container layout logic
  - `sway.rs` — Sway IPC client (`swayipc-async`)
  - `types.rs` — shared types
- `tests/` — Integration tests (`integration_tests.rs`)
- `integrations/` — Agent-side clients, separate from the Rust crate
  - `claude/` — shell wrapper, hook installer, and hook test script (`test-hooks.sh`)
  - `pi/` — TypeScript client (`src/`, `tests/`), its own `package.json`
  - `antigravity/` — Node.js sidecar client (`sidecar.mjs`, `sidecar.test.mjs`) and README
- `planning/` — Execution plans and state
  - `EXECUTION_PLAN_N.md` — wave plans (current: highest N; older waves in planning/archive/)
  - `PROMPT_TEMPLATE.md` — worker prompt template
  - `STATE_N.md` — execution state (current: highest N)
  - `tasks/` — per-task files, named `<wave>.<task>.todo.md`
- `target/` — Cargo build output (gitignored)
- `.pi-subagents/` — agent run artifacts (gitignored)
- Root design docs: `Sway Multi Agent Architecture Design.md`, `Sway Multi-Agent Viewer Architecture.md`

## Workflows

- **Rust development**: `cargo` via `make`. Edition 2024.
- **TypeScript client** (`integrations/pi/`): `npm run build`, `npm test` (uses `tsx --test`), run from that directory.
- **Claude hook tests**: `integrations/claude/test-hooks.sh` — starts a mock Unix-socket server and checks hook status messages.
- **Antigravity sidecar** (`integrations/antigravity/`): `node --test sidecar.test.mjs`, run from that directory (also included in `make test`).
- **Implementation planning**: Review `planning/` for the active wave plan and the highest-numbered `STATE_N.md` before changing code. Task status lives there, not in this file.
- **Test layout**: Rust unit tests live in a `#[cfg(test)] mod tests` at the bottom of the module they cover (`src/daemon.rs`, `src/layout.rs`, `src/config.rs`). Cross-module and socket-level tests go in `tests/integration_tests.rs`. Each `integrations/` client tests itself in its own directory; `make test` runs the Antigravity tests through the root `Makefile`.
- **CI**: none. There is no `.github/` workflow. `make all` run locally is the only gate.

## Infrastructure

- **Ecosystem**: Rust (Edition 2024), Cargo
- **Key dependencies**: `tokio`, `serde`, `serde_json`, `swayipc-async`, `clap`, `toml`, `anyhow`, `futures-lite`, `shlex`
- **Runtime requirement**: a running Sway session with `SWAYSOCK` set for live mode
- **Config file**: `~/.config/swayve/config.toml`
- **Ignored paths**: `target/`, `.pi-subagents/`, `node_modules/`, `dist/`, `*.js`

## References

- `README.md` — user-facing install, Sway config, CLI reference
- `Sway Multi Agent Architecture Design.md` — high-level design
- `Sway Multi-Agent Viewer Architecture.md` — viewer component architecture
- `integrations/claude/README.md`, `integrations/pi/README.md`, `integrations/antigravity/README.md` — per-integration setup
