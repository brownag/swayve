# Orchestration 16 State: Subscriber Drop-on-Full Recorded and Tested (G5)

Plan: `planning/EXECUTION_PLAN_16.md`
Last Updated: 2026-11-07

## Baseline (before Wave 1)
- Orchestration 15 complete, committed as `57dbbc5` (docs) after `5788c3e` and `3422164`.
- `make all` (last recorded at Orchestration 15): 24 unit passed, 10 integration passed, release build OK.
- Working tree clean at plan time.

## Wave Board

### Wave 1: Drop-on-Full Test and ADR 0008 Amendment
- [x] **Task 16.1.1**: Add `test_daemon_broadcast_drops_newest_when_subscriber_full` (`src/daemon.rs`); amend ADR 0008 (`docs/decisions/0008-push-status-subscription-for-hud.md`)

### Wave 1 end gate
- [x] `make all` passes serially (check, test, build); 25 unit + 10 integration passed, release build OK, no warnings

### Deferred (manual, not gated)
- [ ] Live-Sway focus check from plan 13 (outstanding).
- [ ] Malformed-config live check from plan 15 (outstanding).

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

## 16.1.1
- Status: done, not committed.
- Added `test_daemon_broadcast_drops_newest_when_subscriber_full` to `mod tests` in `src/daemon.rs` (+34 lines, test code only).
- Amended third Negative consequence in `docs/decisions/0008-push-status-subscription-for-hud.md` with the drop-on-full policy; status stays "Accepted".
- `cargo test --lib test_daemon_broadcast_drops_newest_when_subscriber_full`: 1 passed.
- Greps: "should be confirmed..." count 0; `status: "Accepted"` one line (line 4).
- `cargo check`: clean, no warnings.
- Full suite not run (per instruction). `planning/GOALS.md` diff predates this task.
- Discovered: none.
