# Execution Plan 13: Daemon Focus Follows Sway's Actual Focus (Goal G1)

## Goal & Scope

**Goal**: `swayve focus --cycle` continues from the agent window the user is actually on, even after focus moved outside the daemon (mouse click, Sway keybinding). Today `last_focused` (`src/daemon.rs`, `run_daemon_loops`) changes only when the daemon itself issues a focus, so cycling after a manual focus change starts from a stale window.

**In scope:**
- Subscribe to Sway window-focus events on the existing window subscription in `src/sway.rs`.
- Update `last_focused` in `src/daemon.rs` from those events when the focused window is a registered agent.
- An integration test that injects focus events and checks the next `FocusWindow`.

**Out of scope:**
- Ignoring or clearing focus on non-agent windows: non-agent focus is ignored, `last_focused` is kept. (Grill decision 1.)
- Real-Sway verification. Deferred to a manual check, below. Not a worker task.
- Recording the focus-state decision as an ADR (G2), and correcting ADR 0007's stale children-only description. Both are later goals.
- Daemon-down fallback (G6), `config.toml` error handling (G3), mark-reapply test (G4), channel drop behavior (G5).

## Decision Log

- **D1: Non-agent focus is ignored** (2026-11-02). When focus lands on a non-agent window, `last_focused` keeps the last agent, so cycling resumes from it. (vs. clearing it to restart at the top.)
- **D2: Synthetic test plus deferred manual check** (2026-11-02). `cargo test` injects `WindowFocused` events through the channel, with no Sway. Live-Sway behavior is checked by hand and is not gated by this wave.
- **D3: Two sequential tasks** (2026-11-02). 13.1 adds the event and listener. 13.2 handles it and adds the test. 13.2 depends on 13.1 for the new variant.
- **D4: Mid-tier model for both workers** (2026-11-02). Skips the small-model attempt. The escalation ladder still applies.
- **D5: Acceptance bar** (2026-11-02, recommended default, not separately confirmed). Each task runs its targeted check. At wave end, `make all` runs serially.

## Hazard Notes

- **Exhaustive match on `SwayEvent`**: `src/daemon.rs` matches `SwayEvent` at ~line 320 (`WindowCreated`) and ~line 392 (`WindowDestroyed`). Adding a variant in 13.1 fails the build unless `daemon.rs` gets a no-op arm for it in that same task. This is why 13.1 touches `daemon.rs`.
- **Focus events from the daemon's own commands**: when the daemon issues `FocusWindow`, Sway also emits a focus event. It names the same agent, so updating `last_focused` again is harmless.
- **Closed windows**: `next_focus_target` already falls back to the top when the focused agent no longer exists. No change needed for destroyed windows.
- **`last_focused` is loop-local** in `run_daemon_loops`. Only the Sway-event arm and the `DaemonEvent::Focus` arm may write it.
- **Shared test socket paths**: integration tests use fixed `/tmp/swayve_test_*.sock` paths. The new test must use a unique path and remove it on exit.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **13.1.1** | Add `WindowFocused` event and Sway focus listener arm; placeholder daemon arm | `src/sway.rs`, `src/daemon.rs` | None |
| **Wave 1** | **13.1.2** | Handle `WindowFocused` in the daemon and add the focus-follows test | `src/daemon.rs`, `tests/integration_tests.rs` | 13.1.1 |

Tasks run sequentially even though both are in Wave 1: both touch `src/daemon.rs`, so they cannot run in parallel. Same-file tasks are sequenced, never parallelized (wave-orchestration rule).

---

## Task Specifications

### Task 13.1.1: Add `WindowFocused` Event and Sway Focus Listener

**ID**: `13.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/sway.rs`, `src/daemon.rs`
- can create: none
- scratch: `.development/` only, if needed
**Depends-on**: None

**Spec prose**:
1. In `src/sway.rs`, add to `enum SwayEvent` (~line 53): `WindowFocused { con_id: i64 },`.
2. In `run_event_listener` (~line 114), the existing `match win_event.change` handles `New` and `Close`. Add a `WindowChange::Focus` arm that sends `SwayEvent::WindowFocused { con_id: win_event.container.id }` on `event_tx`, using `let _ = ... .await` like the others.
3. In `src/daemon.rs`, both `match` blocks on `SwayEvent` (~lines 320 and 392) must still compile. Add a no-op arm `SwayEvent::WindowFocused { .. } => {}` to each. Task 13.1.2 replaces the one in the `DaemonEvent::Sway` handler.
4. Do not change `run_event_listener`'s dry-run behavior or its subscription list (`EventType::Window` already covers focus).

**Constraints (do not touch)**:
- Do not edit `next_focus_target`, `last_focused`, or any focus logic in `daemon.rs`. That is 13.1.2.
- Do not edit `tests/`.
- Do not add dependencies.

**Grep anchors**:
- `enum SwayEvent` (`src/sway.rs`)
- `WindowChange::Close` (`src/sway.rs`, the arm to model the new one on)
- `SwayEvent::WindowDestroyed { con_id }` (`src/daemon.rs`, two match sites)

**Reference implementation**: the existing `WindowChange::Close` arm in `src/sway.rs`.

**Acceptance**: `cargo check` passes with no new warnings.
**Targeted test**: none beyond compilation; this is a plumbing change.

---

### Task 13.1.2: Handle `WindowFocused` in the Daemon and Add the Focus-Follows Test

**ID**: `13.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs`, `tests/integration_tests.rs`
- can create: none
- scratch: `.development/` only, if needed
**Depends-on**: 13.1.1

**Spec prose**:
1. In `src/daemon.rs`, replace the placeholder `SwayEvent::WindowFocused { .. } => {}` arm in the `DaemonEvent::Sway` handler (~line 320 block) with: look up `state.con_to_agent.get(&con_id)`; if it is an agent, set `last_focused = Some(agent_id.clone())`. If it is not an agent, do nothing (D1).
2. Leave the placeholder arm in the `WindowDestroyed` block (~line 392) as a no-op.
3. Add `test_focus_follows_sway_focus` to `tests/integration_tests.rs`, modeled on `test_focus_survives_parent_destruction` (~line 459). Use the socket path `/tmp/swayve_test_focus_follows.sock` and remove it at the end.
   - Register `parent-f` (no parent), `child-a` and `child-b` (parent `parent-f`), with `app_id` `agent.pi.<id>`.
   - Send `WindowCreated` for each: `con_id` 6000 (parent-f), 6001 (child-a), 6002 (child-b). Drain commands.
   - Marks are `mark_<id with - replaced by _>` (`src/types.rs:8`): `mark_parent_f`, `mark_child_a`, `mark_child_b`.
   - Pre-order is `[parent_f, child_a, child_b]`. Assert the full sequence of `FocusWindow` marks:
     1. `Focus` → `mark_parent_f` (no prior focus, start at top).
     2. `Focus` → `mark_child_a`.
     3. Inject `WindowFocused { con_id: 6002 }` (manual focus on child-b), then `Focus` → `mark_parent_f` (wrap after child-b).
     4. Inject `WindowFocused { con_id: 9999 }` (not an agent, ignored per D1), then `Focus` → `mark_child_a` (continues from parent-f, the last agent).

   Check the sequence against `next_focus_target` before writing the assertions; if the test disagrees with the code, stop and report rather than changing the code.

**Constraints (do not touch)**:
- Do not change `next_focus_target` or the `DaemonEvent::Focus` arm.
- Do not change `src/sway.rs` (13.1.1 owns it).
- Do not change existing tests.

**Grep anchors**:
- `SwayEvent::WindowFocused` (`src/daemon.rs`, the placeholder to replace)
- `last_focused` (`src/daemon.rs`, `run_daemon_loops`)
- `fn next_focus_target` (`src/daemon.rs`, the pre-order rule being tested)
- `test_focus_survives_parent_destruction` (`tests/integration_tests.rs`, the reference test)

**Reference implementation**: `test_focus_survives_parent_destruction` for setup, `DaemonEvent::Sway(SwayEvent::...)` injection, and `recv_command` usage.

**Acceptance**: `cargo test test_focus_follows_sway_focus -- --nocapture` passes.
**Targeted test**: the same command. Then at wave end, the full `make all` suite serially.

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 13.1.1 | Add focus event and listener | Low (plumbing) | ~15 LOC |
| 13.1.2 | Update `last_focused` and test | Medium (the test sequence) | ~60 LOC |

## Manual Verification (deferred, not a worker task)

After Wave 1 merges: run `make daemon` under a live Sway session with two or more agent windows. Click between them, then run `swayve focus --cycle`, and confirm it lands on the next sibling in pre-order from the window you clicked.
