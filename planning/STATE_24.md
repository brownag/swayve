# Orchestration 24 State: Spawn Registration Wait (G52, re-scoped) and FocusCycle (G58)

Plan: `planning/EXECUTION_PLAN_24.md` (awaiting approval)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `805fcea` (main), equal to `origin/main` after this session's push.
- Last serial `make all` (Orchestration 23 final): unit 67 passed / 2 ignored; integration 11; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1; `diff_golden` 1; Pi and hook tests passed; zero warnings.
- Clippy (`cargo clippy --all-targets`): no warnings.

## Wave Board

### Wave 1: Spawn wait, FocusCycle
- [ ] **Task 24.1.1**: G52 re-scoped, spawn waits for registration by polling (`src/cli.rs`, `tests/spawn_register.rs`)
- [ ] **Task 24.1.2**: G58 `FocusCycle` owns the cursor (`src/daemon.rs`)

### Wave 1 end gate
- [ ] `make all` passes serially. Expect unit 71 / 2 ignored; integration 11; `spawn_register` 1; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1; `diff_golden` 1. Actual: ___
- [ ] `subscribe_golden` and `diff_golden` unchanged. Actual: ___
- [ ] Integration suite passes 5 runs (H4). Actual: ___

### Orchestrator steps
- [ ] `planning/GOALS.md`: G52 moved to Completed, re-scoped, wire ack pruned. G58 moved to Completed.
- [ ] `git status` clean. Nothing pushed.

## Known limits (recorded at plan time)
- The spawn wait is best-effort. A daemon that takes more than 2 s launches the window anyway, with a warning (D2).
- The wire ack from G52 is pruned. ADR 0009 and ADR 0014 stay as they are.

## Task Reports

_Workers report in their final message. The orchestrator distills each report into this section (15 lines or fewer)._
