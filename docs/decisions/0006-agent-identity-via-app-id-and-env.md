---
id: "0006"
title: "Agent Identity via Unique app_id and Injected Environment"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-08-01
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Commits 0efcdd5, 87a5946, 60f0804; src/layout.rs (agent.monitor. prefix); src/daemon.rs spawn env"
---

# 6. Agent Identity via Unique app_id and Injected Environment

## Context and Problem Statement
Sway identifies windows by con_id and app_id. Several agent windows can share one terminal app_id (`foot`), so the daemon cannot tell them apart. A spawned agent also needs to know its own ID and the socket path so its hooks can report back.

## Decision Drivers
* Each agent window must map to one daemon record.
* Agents and hooks must find the socket without configuration.
* Monitor windows (log tails) must not be confused with agents.

## Considered Options
1. Match windows by PID only.
2. Give each spawn a unique app_id and inject identity as environment variables.
3. Use Sway window titles.

## Decision Outcome
Chosen option: "Unique app_id per spawn, plus injected environment", because app_id is set at launch and survives until close, and the environment reaches the agent's own process tree.

Conventions:
* Agent windows get a unique app_id at spawn.
* Log-tail monitors use the `agent.monitor.<agent_id>` prefix. `src/layout.rs` checks for it and routes the window to the matching agent.
* Spawned terminals receive `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, and `SWAYVE_PROJECT_ID` (`src/cli.rs`, spawn command).
* Lifecycle hook commands additionally receive `SWAYVE_AGENT_ID` and `SWAYVE_AGENT_STATUS` (`src/daemon.rs`, `execute_lifecycle_hook`).

### Consequences
* **Positive**: Window-to-agent mapping is an exact string match, not a heuristic.
* **Negative**: The `agent.monitor.` prefix is a string convention. Any new special window kind needs its own prefix, and nothing enforces uniqueness across prefixes.

## Validation & Compliance
Integration tests in `87a5946` cover unique app_id format and multiple instances.
