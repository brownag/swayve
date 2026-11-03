---
id: "0003"
title: "Daemon Owns Agent State; Clients Route Through Its Socket"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-29
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_WAVE4.md D2 (2026-07-29); commits 1410acc, ec8d829, 05d761f"
---

# 3. Daemon Owns Agent State; Clients Route Through Its Socket

## Context and Problem Statement
The CLI and external agents can both change agent state: spawning windows, reporting status, moving focus. If the CLI drives Sway directly while the daemon keeps its own registry, the two disagree about which window is focused and what the hierarchy is.

## Decision Drivers
* One source of truth for the agent hierarchy.
* Focus must follow the hierarchy, which only the daemon knows.
* Clients must still work when the daemon is down.

## Considered Options
1. CLI acts independently on Sway; daemon only tracks status.
2. CLI sends requests to the daemon, which owns registry and focus; CLI executes nothing itself.
3. Shared state file read by all processes.

## Decision Outcome
Chosen option: "CLI sends requests to the daemon over the Unix socket", because the daemon is the only process that holds the parent/child tree. Wave 4 D2 records this as "CLI does not execute focus directly."

### Consequences
* **Positive**: `swayve focus --cycle` sends `daemon.focus` to the socket (`src/cli.rs` `Commands::Focus`). The daemon chooses the next sibling.
* **Negative**: When the socket is unreachable, the CLI falls back to a **flat** cycle over the Sway tree and prints `Daemon unreachable. Cycling focus directly.` The hierarchy is lost in that path. Wave 4's hazard notes called this "graceful degradation"; that is what it does, with the reduced behaviour above.
* **Negative**: Every focus action needs a live daemon for full behaviour.

## Validation & Compliance
Integration test `05d761f` covers focus cycling over the daemon socket. The fallback path is not covered by a test that checks hierarchy, so it stays a known gap.
