# AGENTS.md - swayve-hud

## Quick Start
- `make all` (from workspace root) -> `check` + `test` + `build` (release binary at `target/release/swayve-hud`)
- `cargo check -p swayve-hud` / `cargo clippy -p swayve-hud --all-targets -- -D warnings`
- `cargo test -p swayve-hud` (unit + integration)
- `cargo run -p swayve-hud` (or `swayve hud`) -> run the HUD against the live socket

## Find Things
- `src/lib.rs` -> crate root, re-exports `client`, `fold`, `types`
- `src/main.rs` -> binary entry point, CLI args (`--socket`, `-h`)
- `src/client.rs` -> socket path resolution, subscribe, backoff loop, ANSI renderer
- `src/fold.rs` -> diff-line folding into the agent map
- `src/types.rs` -> `AgentStatus`, `AgentWindow`, `DiffFrame`
- `tests/` -> integration tests (mock Unix socket)
- Inline unit tests -> `#[cfg(test)] mod tests` at bottom of each `src/*.rs`
- `planning/` -> execution plans, per-task specs, prompt templates
- `README.md` -> user-facing behavior, socket path precedence

## Workflows
- Planned work: `planning/EXECUTION_PLAN_*.md` defines tasks; per-task specs live in `planning/tasks/<id>.todo.md`
- Task lifecycle: read task spec -> implement -> `make all` must pass -> mark task done in its spec
- Session state for multi-wave execution: `planning/STATE_*.md`

## Infrastructure
- Socket path precedence: `$SWAYVE_SOCKET` -> `$XDG_RUNTIME_DIR/swayve/swayve.sock`
- Protocol message sent on connect: `daemon.subscribe_diff` (newline-terminated JSON, see `SUBSCRIBE_DIFF_MSG` in `src/client.rs`)
- Backoff: initial 500ms, max 5000ms (`src/client.rs`)
- Toolchain: Rust, edition 2024 (rustc >= 1.85)
- Dependencies: `Cargo.toml` (`swayve-types` and `serde_json`)
- Build output and `.development/` are git-ignored

## References
- `README.md` for behavior and build prerequisites
- `../../planning/EXECUTION_PLAN_35.md` for the HUD master plan
