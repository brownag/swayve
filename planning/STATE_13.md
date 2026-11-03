# Orchestration 13 State: Daemon Focus Follows Sway's Actual Focus (G1)

Plan: `planning/EXECUTION_PLAN_13.md`
Last Updated: 2026-11-02

## Baseline (recorded 2026-11-02, before Wave 1)
- `cargo test`: 22 unit passed, 9 integration passed, 0 failed.
- Prior orchestration: Wave 12 complete (see `planning/STATE.md`).

## Wave Board

### Wave 1: Focus Follows Sway's Actual Focus
- [x] **Task 13.1.1**: Add `WindowFocused` event and Sway focus listener arm; placeholder daemon arms
- [x] **Task 13.1.2**: Handle `WindowFocused` in the daemon; add `test_focus_follows_sway_focus`

### Wave 1 end gate
- [x] `make all` passes serially (check, test, build) -- 22 unit + 10 integration passed, release build OK

### Deferred (manual, not gated)
- [ ] Live-Sway check: click between agent windows, run `swayve focus --cycle`, confirm the next sibling in pre-order.

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

### 13.1.1
- Status: done. Micro-plan approved, then executed.
- Files: `src/sway.rs` (+ `WindowFocused` variant, `WindowChange::Focus` arm); `src/daemon.rs` (one no-op arm).
- Tests: `cargo check` clean. Orchestrator re-checked the diff.
- Deviations: none.
- Gotchas: `SwayEvent` is matched once in `daemon.rs`.

### 13.1.2
- Status: done. Micro-plan approved, then executed.
- Files: `src/daemon.rs` (`WindowFocused` arm sets `last_focused` for known agents); `tests/integration_tests.rs` (`test_focus_follows_sway_focus`).
- Tests: `cargo test test_focus_follows_sway_focus` passed. Full gate `make all`: 22 unit + 10 integration passed, release build OK.
- Deviations: none.
- Gotchas: the test sleeps 50ms after injected focus events to order them against the socket path.
