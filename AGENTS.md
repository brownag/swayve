# AGENTS.md

## Quick Start

- **Build**: `make build` (or `cargo build`)
- **Run**: `make run` (or `cargo run`)
- **Test**: `make test` (or `cargo test`)
- **Daemon**: `make daemon` (starts live daemon) or `make daemon-dry` (starts dry-run mock daemon)
- **Reload**: `make reload` (reloads Sway configuration)


## Find Things

- `src/` — Source code (`main.rs`, `lib.rs`, `cli.rs`, `config.rs`, `daemon.rs`, `sway.rs`, `types.rs`)
- `tests/` — Integration tests (`integration_tests.rs`)
- `planning/` — Implementation plans and state (`EXECUTION_PLAN.md`, `PROMPT_TEMPLATE.md`, `STATE.md`)
- `target/` — Build artifacts and dependency information

## Workflows

- **Development**: Use `cargo` for building, running, and testing.
- **Implementation Planning**: Review `planning/` for active execution plans and state before making changes.

## Infrastructure

- **Ecosystem**: Rust (Edition 2024)
- **Build Tool**: Cargo
- **Key Dependencies**: `tokio`, `serde`, `swayipc-async`, `clap`, `toml`, `anyhow`, `futures-lite`

## References

- `Sway Multi Agent Architecture Design.md` — High-level design documentation.
- `Sway Multi-Agent Viewer Architecture.md` — Viewer component architecture.
