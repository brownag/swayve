---
id: "0002"
title: "Implement the Daemon in Rust on Tokio with swayipc-async"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-26
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Initial commit 6c3585c; Cargo.toml; Sway Multi Agent Architecture Design.md (names the daemon sway-agentd)"
---

# 2. Implement the Daemon in Rust on Tokio with swayipc-async

## Context and Problem Statement
The daemon must hold agent state, serve a Unix socket, and react to Sway events concurrently, without blocking on any one of them. The design doc named the binary `sway-agentd`. The project ships as `swayve`.

## Decision Drivers
* Concurrent socket clients, Sway event stream, and command execution in one process.
* A typed Sway IPC client, not hand-rolled `swaymsg` parsing.
* Single static binary for a Wayland desktop session.

## Considered Options
1. Shell scripts around `swaymsg` and `nc`.
2. Python daemon with `i3ipc`.
3. Rust with `tokio` and `swayipc-async`.

## Decision Outcome
Chosen option: "Rust with `tokio` and `swayipc-async`", because the daemon needs typed IPC and long-running concurrent tasks (event listener, command executor, socket listener) connected by channels.

### Consequences
* **Positive**: Event listener, command executor, and socket handler run as separate tasks joined by `mpsc` channels (`src/sway.rs`, `src/daemon.rs`, `src/cli.rs`).
* **Negative**: Rust toolchain needed to build (`cargo`, Edition 2024). Heavier than the shell option.
* **Naming**: The design doc's `sway-agentd` is superseded by the `swayve` name. Treat the design doc as historical.

## Validation & Compliance
`Cargo.toml` pins `tokio`, `swayipc-async`, `serde`, `clap`, `toml`, `anyhow`. `make test` runs the suite.
