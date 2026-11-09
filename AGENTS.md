# AGENTS.md

## Quick Start

- **Build**: `make build` (release binaries across workspace) or `cargo build --workspace`
- **Check**: `make check` (`cargo check --workspace` + `cargo clippy --workspace --all-targets -- -D warnings`)
- **Test**: `make test` (`cargo test --workspace` + integration suites) — runs integration and unit tests
- **Full gate**: `make all` (check + test + build)
- **Run CLI**: `make run` (`cargo run -p swayve -- --help`)
- **Daemon**: `make daemon` (live, requires `SWAYSOCK`) or `make daemon-dry` (dry-run, logs commands, offline safe)
- **HUD**: `make hud` (`cargo run -p swayve-hud`) or `cargo run -p swayve -- hud`
- **Reload**: `make reload` (`swaymsg reload`)
- **Clean**: `make clean`

## Find Things

- `Cargo.toml` — Virtual Cargo workspace (`crates/swayve-types`, `crates/swayve`, `crates/swayve-hud`)
- `crates/swayve-types/` — Shared protocol and wire types crate (`AgentId`, `ProjectId`, `AgentStatus`, `AgentWindow`, `AgentIPCMessage`, `DiffFrame`)
- `crates/swayve/` — Main `swayve` daemon and CLI crate
  - `src/main.rs` — binary entry point
  - `src/lib.rs` — module root (`cli`, `config`, `daemon`, `layout`, `ratelimit`, `sway`, `types`)
  - `src/cli.rs` — argument parsing (clap) and subcommands (`init-shell`, `daemon`, `spawn`, `panel`, `status`, `focus`, `hud`)
  - `src/config.rs` — TOML config loading (`~/.config/swayve/config.toml`)
  - `src/daemon.rs` — background daemon and IPC
  - `src/layout.rs` — window/container layout logic
  - `src/sway.rs` — Sway IPC client (`swayipc-async`)
  - `src/types.rs` — re-exports `swayve-types`
  - `tests/` — Integration and golden tests (`integration_tests.rs`, `diff_golden.rs`, `subscribe_golden.rs`, `slow_consumer.rs`, `rate_limit_socket.rs`, `spawn_register.rs`)
- `crates/swayve-hud/` — Terminal Heads-Up Display (`swayve-hud`) crate (see `crates/swayve-hud/AGENTS.md` and `crates/swayve-hud/README.md`)
  - `src/lib.rs` — library root (`client`, `fold`, `types`)
  - `src/main.rs` — standalone `swayve-hud` binary entry point
  - `src/client.rs` — socket path resolution, `daemon.subscribe_diff` client, exponential backoff loop, ANSI renderer
  - `src/fold.rs` — diff-line folding into the agent map
  - `tests/integration_test.rs` — mock Unix socket integration tests
- `integrations/` — Agent-side clients, separate from the Rust crates
  - `claude/` — shell wrapper, hook installer, and hook test script (`test-hooks.sh`)
  - `pi/` — TypeScript client (`src/`, `tests/`), its own `package.json`
  - `antigravity/` — Node.js sidecar client (`sidecar.mjs`, `sidecar.test.mjs`) and README
- `planning/` — Execution plans and state
  - `EXECUTION_PLAN_N.md` — wave plans (current: highest N; older waves in planning/archive/)
  - `PROMPT_TEMPLATE.md` — worker prompt template
  - `STATE_N.md` — execution state (current: highest N)
  - `tasks/` — per-task files, named `<wave>.<task>.todo.md`
- `target/` — Cargo workspace build output (gitignored)
- `.pi-subagents/`, `.development/` — agent run and scratch artifacts (gitignored)
- Design docs: `planning/Sway Multi Agent Architecture Design.md`, `planning/Sway Multi-Agent Viewer Architecture.md`

## Workflows

- **Rust development**: `cargo` via `make`. Edition 2024.
- **TypeScript client** (`integrations/pi/`): `npm run build`, `npm test` (uses `tsx --test`), run from that directory.
- **Claude hook tests**: `integrations/claude/test-hooks.sh` — starts a mock Unix-socket server and checks hook status messages.
- **Antigravity sidecar** (`integrations/antigravity/`): `node --test sidecar.test.mjs`, run from that directory (also included in `make test`).
- **Implementation planning**: Review `planning/` for the active wave plan and the highest-numbered `STATE_N.md` before changing code. Task status lives there, not in this file.
- **Test layout**: Rust unit tests live in a `#[cfg(test)] mod tests` at the bottom of the module they cover (`crates/swayve/src/daemon.rs`, `crates/swayve/src/layout.rs`, `crates/swayve/src/config.rs`, `crates/swayve-hud/src/*.rs`, `crates/swayve-types/src/lib.rs`). Cross-module and socket-level tests go in `crates/swayve/tests/` and `crates/swayve-hud/tests/`. Each `integrations/` client tests itself in its own directory; `make test` runs the Antigravity tests through the root `Makefile`.
- **CI**: none. There is no `.github/` workflow. `make all` run locally is the only gate.

## Infrastructure

- **Ecosystem**: Rust (Edition 2024), Cargo virtual workspace
- **Key dependencies**: `tokio`, `serde`, `serde_json`, `swayipc-async`, `clap`, `toml`, `anyhow`, `futures-lite`, `shlex`
- **Runtime requirement**: a running Sway session with `SWAYSOCK` set for live mode
- **Config file**: `~/.config/swayve/config.toml`
- **Ignored paths**: `target/`, `.pi-subagents/`, `.development/`, `node_modules/`, `dist/`, `*.js`

## References

- `README.md` — user-facing install, Sway config, CLI reference
- `crates/swayve-hud/README.md`, `crates/swayve-hud/AGENTS.md` — HUD crate reference
- `planning/Sway Multi Agent Architecture Design.md` — high-level design
- `planning/Sway Multi-Agent Viewer Architecture.md` — viewer component architecture
- `integrations/claude/README.md`, `integrations/pi/README.md`, `integrations/antigravity/README.md` — per-integration setup
