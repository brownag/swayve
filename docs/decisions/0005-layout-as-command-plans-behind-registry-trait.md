---
id: "0005"
title: "Layout Emits Sway Command Plans Behind an AgentRegistry Trait"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-27
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Commits 9cdad50, cc0bf67, cfd8f4d; src/layout.rs compute_commands; src/sway.rs SwayCommand; Wave 3 task 3.1"
---

# 5. Layout Emits Sway Command Plans Behind an AgentRegistry Trait

## Context and Problem Statement
Layout decisions (where a new window goes, when to tab a group) depend on agent state. Sway actions have side effects. Layout code that called Sway directly would be hard to test, and it created an import cycle with the daemon (`daemon` -> `layout` -> `daemon`).

## Decision Drivers
* Layout logic must be unit-testable without a Sway session.
* No circular module dependency.
* Dry-run must not need a Sway connection.

## Considered Options
1. `LayoutManager` calls Sway directly.
2. `LayoutManager` returns a plan of `SwayCommand` values; a separate executor applies them.
3. Split the layout into a separate process.

## Decision Outcome
Chosen option: "`LayoutManager` returns `SwayCommand` values through a channel to `SwayManager`". Layout reads agents through an `AgentRegistry` trait with one method, `agents()`, that `DaemonState` implements (commit `cfd8f4d`). This is the trait that broke the cycle.

### Consequences
* **Positive**: `compute_commands` is pure over `(app_id, agents)`. Tests build agent lists directly. Dry-run swaps the executor's Sway connection for logging (`src/sway.rs`, `dry_run`).
* **Negative**: Two hops between a window event and a Sway command. A bug can sit in the plan or in the executor.
* **Pattern**: New layout behaviour goes in `compute_commands` as a `SwayCommand` variant, not as a direct Sway call.

## Validation & Compliance
Wave 3 task 3.1 and `cfd8f4d` record the trait extraction. `make daemon-dry` runs the same path without Sway.
