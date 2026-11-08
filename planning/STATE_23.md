# Orchestration 23 State: Open Backlog Bundle

Plan: `planning/EXECUTION_PLAN_23.md` (awaiting approval)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `48b8d61` (main). Local only; `origin/main` is at `96f18af`, so this plan's commits are unpushed.
- Last serial `make all` (at `48b8d61`): unit 60 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; Pi and hook tests passed; zero warnings.
- Clippy (`cargo clippy --all-targets`): no warnings.

## Wave Board

### Wave 1: Tests, Spawn Plan, CI
- [ ] **Task 23.1.1**: G46 socket-level over-limit test (`tests/rate_limit_socket.rs`)
- [ ] **Task 23.1.2**: G50 poll hook flag files (`tests/integration_tests.rs`)
- [ ] **Task 23.1.3**: G57 spawn plan pure function (`src/cli.rs`)
- [ ] **Task 23.1.4**: G51 CI workflow file (`.github/workflows/ci.yml`)

### Wave 1 end gate
- [ ] `make all` passes serially. Expect unit 60+ / 2 ignored; integration 11; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1. Actual: ___

### Wave 2: Depth Test, README
- [ ] **Task 23.2.1**: G55 depth-limit test can fail (`src/daemon.rs`)
- [ ] **Task 23.2.2**: G47 Socket Contract bullets (`README.md`)

### Wave 2 end gate
- [ ] `make all` passes serially. Actual: ___
- [ ] `cargo test --lib depth` passes; removing the depth check fails it (checked by hand). Actual: ___

### Wave 3: Barrier, Diff Golden
- [ ] **Task 23.3.1**: G49 barrier replaces focus sleeps (`src/daemon.rs`, `tests/integration_tests.rs`)
- [ ] **Task 23.3.2**: G56 `subscribe_diff` golden, captured before change (`tests/diff_golden.rs`)

### Wave 3 end gate
- [ ] `make all` passes serially. Actual: ___
- [ ] Integration suite passes 5 runs. Actual: ___
- [ ] `diff_golden` captured at SHA: ___

### Wave 4: DiffFrame
- [ ] **Task 23.4.1**: G56 `DiffFrame` shared type (`src/daemon.rs`, `src/cli.rs`, `src/types.rs`)

### Wave 4 end gate
- [ ] `make all` passes serially; `diff_golden` and `subscribe_golden` unchanged. Actual: ___

### Wave 5: Window Lifecycle
- [ ] **Task 23.5.1**: G54 window lifecycle in `DaemonState` (`src/daemon.rs`)

### Wave 5 end gate
- [ ] `make all` and clippy pass serially. Actual: ___
- [ ] Integration suite passes 5 runs. Actual: ___

### Orchestrator steps
- [ ] `planning/GOALS.md`: G46, G47, G49, G50, G51, G54, G55, G56, G57 moved to Completed with SHAs.
- [ ] `git status` clean. Nothing pushed.

### Deferred (manual, not gated)
- G52, G58: deferred by trigger, not in this plan.
- G53, G25, G15: external, prompts in `planning/external/`.

## Known limits (recorded at plan time)
- G51 is a file only. It runs after a push, which needs the user's go-ahead.
- G54's merge of the two auto-monitor gates (D3) is pending the user's approval.

## Task Reports

_Workers report in their final message. The orchestrator distills each report into this section (15 lines or fewer)._
