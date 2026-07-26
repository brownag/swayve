# AGENTS.md

## Quick Start

- **Build**: `cargo build`
- **Run**: `cargo run`
- **Test**: `cargo test`

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
