---
id: "0011"
title: "Focus Cycle State Is Fed by Sway Window-Focus Events"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-02
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_13.md D1; Orchestration 13 tasks 13.1.1 and 13.1.2; commit 436b190; src/daemon.rs last_focused, next_focus_target; src/sway.rs SwayEvent::WindowFocused"
---

# 11. Focus Cycle State Is Fed by Sway Window-Focus Events

## Context and Problem Statement
`focus --cycle` continues from the last agent the daemon focused. Focus changes made outside the daemon (mouse click, Sway binding) were not seen, so the next cycle started from a stale window.

## Decision Drivers
* Cycling must resume from the window the user is on.
* No per-cycle Sway query.
* No new daemon fields unless needed.

## Considered Options
1. Query Sway's tree on each `Focus` event, as the CLI fallback does.
2. A last-focused tracker updated from Sway focus events.
3. Leave `last_focused` as only the daemon's own last focus.

## Decision Outcome
Chosen option: "a last-focused tracker updated from Sway focus events", because it keeps cycle state current without a Sway round-trip. Option 3 is the bug being fixed.

Rules:
* `last_focused` is the cycle's source of truth for `next_focus_target`.
* `run_daemon_loops` sets it when a Sway window-focus event (`SwayEvent::WindowFocused`) names a registered agent, mapped through `con_to_agent`.
* Focus on a non-agent window is ignored, so cycling resumes from the last agent (Orchestration 13 D1).
* When the focused agent is destroyed, `next_focus_target` falls back to the top of the cycle.

### Consequences
* **Positive**: No Sway round-trip per cycle. Manual focus is respected.
* **Negative**: `last_focused` is loop-local in `run_daemon_loops`, so only that loop's arms may write it. The window-focus subscription must stay active; it shares the `EventType::Window` stream.
* **Negative**: A daemon-down CLI fallback still cycles flat (see [0003](0003-daemon-owns-agent-state.md)).
* **Neutral**: Daemon-initiated focus also produces a focus event. It names the same agent, so it is harmless.

## Validation & Compliance
Integration test `test_focus_follows_sway_focus` in `tests/integration_tests.rs`. Unit tests `test_hierarchical_focus` and `test_focus_cycles_between_roots` in `src/daemon.rs`. See also [0007](0007-agent-hierarchy-and-sibling-focus.md) for the cycle order.
