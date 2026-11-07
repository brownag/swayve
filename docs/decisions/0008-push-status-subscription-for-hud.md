---
id: "0008"
title: "Push Agent Status to Subscribers over the Socket for the HUD"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-08-02
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN.md (Wave 12) D1, D2; commits 5914a63, 4016a1f, 3fa4005; STATE.md Wave 12"
---

# 8. Push Agent Status to Subscribers over the Socket for the HUD

## Context and Problem Statement
The HUD dashboard must show the live agent tree and state. Polling the daemon wastes work and lags behind state changes.

## Decision Drivers
* Dashboard updates must follow state changes without a poll interval.
* Subscribers must not block the daemon's event loop.
* Reuse the existing socket and line-delimited protocol ([0004](0004-newline-delimited-json-ipc.md)).

## Considered Options
1. Dashboard polls `status` on a timer.
2. Client sends `daemon.subscribe`; daemon pushes the full agent list as a JSON line on every change.
3. Separate status socket.

## Decision Outcome
Chosen option: "`daemon.subscribe` on the existing socket; daemon pushes full `Vec<AgentWindow>` lines on each change", because it reuses the socket and wire format, and one subscription gives the client everything it needs.

Behaviour:
* Each subscriber gets a bounded channel (32). The daemon spawns a writer task per subscriber (`src/daemon.rs`, `handle_client`).
* The payload is the whole list, not a diff.

### Consequences
* **Positive**: Live HUD with no poll interval. The client is a plain line reader.
* **Negative**: Full-state push on every change grows with agent count. Diffs are the upgrade path if it does.
* **Negative**: Slow subscribers are a risk. The bounded channel (32) bounds memory. When a subscriber's channel is full, `broadcast_status` drops the newest snapshot, keeps the subscriber registered, and logs a line. Because each snapshot is the full list, the next successful broadcast restores the subscriber's view, so a slow subscriber can show stale state until the next change. A closed channel prunes the subscriber. Covered by `test_daemon_broadcast_drops_newest_when_subscriber_full` in `src/daemon.rs`.

## Validation & Compliance
Integration test `3fa4005` checks the dashboard subscription stream. `42ba751` covers hooks and monitoring.
