---
id: "0016"
title: "Configurable Per-Client IPC Rate Limit"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-08
authors:
  - "Andrew G. Brown"
supersedes: "0015"
superseded_by: null
source_context: "Backlog goal G48 (planning/GOALS.md); reverses Orchestration 22 D1 as recorded in 0015; src/ratelimit.rs; src/config.rs DaemonConfig"
---

# 16. Configurable Per-Client IPC Rate Limit

## Context and Problem Statement
[0015](0015-ipc-rate-limit-drop-and-log.md) set the per-client IPC limit as constants: 100 lines per second sustained, burst 200. Its decision outcome said "the limit is a constant, not config". The user has now asked for the values to be configurable. A fixed number cannot fit every machine or agent mix, and changing it currently needs a rebuild.

## Decision Drivers
* A user can tune the limit without a rebuild.
* Defaults stay exactly as 0015 set them, so a config file that does not mention the keys behaves as before.
* Bad values must not take the daemon down or disable the guard. This follows the config rule in [0010](0010-toml-config-in-home-config-dir.md) and the fallback behavior from G3.

## Considered Options
1. Keep the constants (0015).
2. Two keys under `[daemon]`, validated, with defaults and a warning on bad values.
3. Allow a value that disables the limit.

## Decision Outcome
Chosen option: option 2.

* `[daemon] ipc_lines_per_sec` (float, default `100.0`): the sustained refill rate.
* `[daemon] ipc_burst` (float, default `200.0`): the bucket capacity.
* Both use serde defaults, so the existing config file and the existing tests need no change.
* A value that is not finite, or `ipc_lines_per_sec <= 0`, or `ipc_burst < 1`, is replaced by its default and logged once with `[config]`, as 0010 does for other bad values.
* Option 3 was rejected. The limit is a guard against flooding, and a switch that turns it off removes that guard.
* The drop-and-log behavior from 0015 does not change. Only the two numbers become configurable.

### Consequences
* **Positive**: users can tune the limit to their agent mix without a rebuild.
* **Positive**: the code path is unchanged. The limiter still takes its two numbers at connection time.
* **Negative**: a user can set a limit that is too low for real bursts and lose lines. The log line shows each drop, so this is visible.
* **Negative**: config changes apply to new connections only. The daemon does not reload config, so an existing connection keeps the limit it started with. A daemon restart applies a change to all connections.

## Validation & Compliance
* Config tests in `src/config.rs`: defaults when the keys are absent; each invalid value falls back to its default.
* `src/ratelimit.rs` tests are unchanged; the constructor still takes plain numbers.
* G46 (socket-level test) uses a configured limit to show that the value takes effect.
