# Orchestration 22 State: Backlog Close-Out (G39-G43, circled as G44)

Plan: `planning/EXECUTION_PLAN_22.md`
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `0fd3c63` (main). `origin/main` equals `main` at `fb34936`; `0fd3c63` adds docs only.
- Last serial `make all` (Orchestration 21 final, at `dc2c4e5`): unit 50 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; Pi 3/3; hook test passed; zero warnings.
- Clippy (`cargo clippy --all-targets`): no warnings.

## Measurements taken during planning (2026-11-08, scratch, not committed)
- Release, `measure_broadcast_cost_diff` with N=1000 added:
  - Before (`to_value`): N=1000 K=10 = 11546.6 µs; N=1000 K=1 = 1251.7 µs.
  - After (`PartialEq`), run 1: N=1000 K=10 = 2290.1 µs; run 2: 2165.6 µs.
  - N=200 K=10 went from 2323.3 to 456.9 / 414.1 µs.
- Verdict against the 10 ms bar: `to_value` misses; `PartialEq` passes. Decision D3: land `PartialEq`.

## Wave Board

### Wave 1: Measurement, Fold Test, Sleeps
- [ ] **Task 22.1.1**: G42 `PartialEq` and N=1000 grid (`src/types.rs`, `src/daemon.rs`)
- [ ] **Task 22.1.2**: G39 `fold_diff_line` helper and tests (`src/cli.rs`)
- [ ] **Task 22.1.3**: G40 poll-based sleep replacement (`tests/integration_tests.rs`)

### Wave 1 end gate
- [ ] `make all` passes serially. Expect unit 53 / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1. Actual: ___
- [ ] N=1000 K=10 release line from 22.1.1 (expect about 2.2 ms): ___
- [ ] Sleeps replaced / left, with line numbers, from 22.1.3: ___

### Wave 2: Rate-Limit Scaffold
- [ ] **Task 22.2.1**: G43 `RateLimiter`, wired into `handle_client`; H1 split in `tests/slow_consumer.rs` (`src/ratelimit.rs`, `src/lib.rs`, `src/daemon.rs`, `tests/slow_consumer.rs`)

### Wave 2 end gate
- [ ] `make all` passes serially. Expect unit 56 / 2 ignored. Actual: ___
- [ ] `slow_consumer` passes 5 runs in a row. Actual: ___

### Wave 3: Docs
- [ ] **Task 22.3.1**: ADR 0013 amendment, ADR 0015, catalog row (`docs/decisions/`)

### Wave 3 end gate
- [ ] `make all` and `make test` pass serially. Actual: ___
- [ ] `subscribe_golden` passes unchanged. Actual: ___

### Orchestrator steps
- [ ] G41: `git branch --merged main` confirms the three branches, then `git branch -d` each. Actual: ___
- [ ] GOALS.md: G39–G43 moved to Completed with SHAs; G44 moved to Completed.
- [ ] `git status` clean; `git worktree list` shows only `main`.

### Deferred (manual, not gated)
- None.

## Known limits (recorded at plan time)
- G40 leaves 3 sleeps by design (D2): 616, 621 (Sway event ordering) and 931 (spawned hooks and process execution).
- G43 limits are constants (D1). A client that exceeds them gets no error reply; this is recorded in ADR 0015.
- The G30 test sends 300 lines across two connections after the H1 split. It still overflows the channel, but the overflow depends on the socket buffer size.

## Process notes
- Orchestration 21 had six worktrees created from a stale base (`298d6db`). D8 requires every worktree HEAD to equal `main` before a worker plans. Confirm this before each approval.
- Workers do not write to this file. The orchestrator records every entry.

## Task Reports

_Workers report in their final message. The orchestrator distills each report into this section (15 lines or fewer)._
