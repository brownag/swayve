---
id: "0013"
title: "Opt-In Diff Status Subscription Alongside the Full-List Stream"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-08
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Orchestration 20 (EXECUTION_PLAN_20.md D1-D3, D7; commits 9d97456, 8d6ec67, b016cbc, 53ae8c9); src/daemon.rs broadcast_diff; src/types.rs SubscribeDiff"
---

# 13. Opt-In Diff Status Subscription Alongside the Full-List Stream

## Context and Problem Statement
[0008](0008-push-status-subscription-for-hud.md) pushes the full agent list on every change. The list grows with agent count, and every subscriber receives the whole thing. ADR 0008 names diffs as the upgrade path. The full-list format is also parsed by `swayve status --watch` and may be relied on by external clients, so changing it in place would break them for a public release.

## Decision Drivers
* Existing `daemon.subscribe` clients must keep working with no change.
* Measured cost (Orchestration 20, task 20.1.1) should drop for subscribers that can use deltas.
* A slow subscriber must recover without a client-side sequence protocol.

## Considered Options
1. Replace the full-list stream with a diff stream.
2. Keep the full-list stream and add an opt-in diff method (`daemon.subscribe_diff`).
3. Add sequence numbers and a client-driven resync request to the diff stream.

## Decision Outcome
Chosen option: option 2. `daemon.subscribe` keeps its bare-array line format. `daemon.subscribe_diff` (`AgentIPCMessage::SubscribeDiff`) streams one object per line: `{"snapshot":bool,"upsert":[AgentWindow...],"remove":[AgentId...]}`. The first line after subscribing is a snapshot. Each later line carries the agents that changed since the last line this subscriber received, and the ids that were removed. A line with nothing to report is not sent.

When a diff subscriber's channel is full, the daemon drops that line and sets `needs_snapshot` for that subscriber. Its next line is a full snapshot. A snapshot replaces the client's whole view, so the client does not need sequence numbers or a resync request. Per-subscriber state is the last-sent map (`sent`), which the daemon compares against the current agents.

### Consequences
* **Positive**: existing clients are unaffected. Diff subscribers receive only what changed.
* **Positive**: a dropped line is repaired by the next snapshot, so the client protocol stays a plain line reader that replaces or patches its map.
* **Negative**: each diff subscriber keeps its own copy of the last-sent state, so memory grows with subscriber count times agent count.
* **Negative**: the daemon serializes agents to compare them (`serde_json::to_value`), because `AgentWindow` has no `PartialEq`. This costs CPU per broadcast per diff subscriber. Measured in release at 50 agents and 10 diff subscribers: 621 µs per broadcast (200 single-agent updates, 171-byte diff lines). That is under the 1 ms threshold set for this decision, so the `to_value` comparison is kept and `PartialEq` is not derived. The cost grows roughly linearly: 2.4 ms at 200 agents and 10 subscribers. Revisit if the measured cost goes past the threshold.
* **Negative**: two protocols now exist on the same socket. Clients must pick one.

## Validation & Compliance
* `src/daemon.rs`: `test_diff_subscriber_gets_snapshot_then_diffs`, `test_diff_subscriber_recovers_with_snapshot_after_drop`.
* `tests/integration_tests.rs`: `test_diff_stream_matches_daemon_state` replays the stream into a map and compares it with the daemon's agent list.
* `README.md` Socket Contract and ADR 0008 (Orchestration 20 amendment) describe both formats.
