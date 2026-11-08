# Orchestration 24 State: Spawn Registration Wait (G52, re-scoped) and FocusCycle (G58)

Plan: `planning/EXECUTION_PLAN_24.md` (approved and executed inline)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `805fcea` (main), equal to `origin/main` after this session's push.
- Last serial `make all` (Orchestration 23 final): unit 67 passed / 2 ignored; integration 11; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1; `diff_golden` 1; Pi and hook tests passed; zero warnings.
- Clippy (`cargo clippy --all-targets`): no warnings.

## Wave Board

### Wave 1: Spawn wait, FocusCycle
- [x] **Task 24.1.1**: G52 re-scoped, spawn waits for registration by polling (`src/cli.rs`, `tests/spawn_register.rs`)
- [x] **Task 24.1.2**: G58 `FocusCycle` owns the cursor (`src/daemon.rs`)

### Wave 1 end gate
- [x] `make all` passes serially. Actual: exit 0; unit 71 / 2 ignored; integration 11; `spawn_register` 1; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1; `diff_golden` 1; zero warnings; clippy clean.
- [x] `subscribe_golden` and `diff_golden` unchanged. Actual: no diff since `805fcea`.
- [x] Integration suite passes 5 runs (H4). Actual: 5 of 5, 11 passed each.

### Orchestrator steps
- [x] `planning/GOALS.md`: G52 moved to Completed, re-scoped, wire ack pruned. G58 moved to Completed.
- [x] `git status` clean. Nothing pushed.

## Deviations
- 24.1.2: the plan listed a fourth test, `test_focus_cycle_next_requires_agent_in_map`. It is dropped. `next_focus_target` only returns agents taken from the map, so that branch cannot be reached. The three tests cover the rule.
- 24.1.1: `register_and_wait` is `pub` in `src/cli.rs` so `tests/spawn_register.rs` can call it.

## Known limits (recorded at plan time)
- The spawn wait is best-effort. A daemon that takes more than 2 s launches the window anyway, with a warning (D2).
- The wire ack from G52 is pruned. ADR 0009 and ADR 0014 stay as they are.

## Task Reports

_Executed inline by the orchestrator, one commit per task (D6)._

- **24.1.2 (G58)** `1012fd5`. `FocusCycle { last }` with `observe()` and `next()` in `src/daemon.rs`. Loop uses `focus.observe` and `focus.next`. 3 unit tests. Integration suite 5 of 5.
- **24.1.1 (G52, re-scoped)** commit `git log --grep='spawn waits'`. `register_and_wait` in `src/cli.rs` writes the register line, then polls `daemon.subscribe` every 10 ms up to 2 s. `spawn` warns and launches on timeout. 1 unit test, 1 integration test (`tests/spawn_register.rs`), 5 of 5 runs.
