# Orchestration 22 State: Backlog Close-Out (G39-G43, circled as G44)

Plan: `planning/EXECUTION_PLAN_22.md`
Last Updated: 2026-11-08 (Orchestration 22 complete)

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
- [x] **Task 22.1.1**: G42 `PartialEq` and N=1000 grid (`src/types.rs`, `src/daemon.rs`)
- [x] **Task 22.1.2**: G39 `fold_diff_line` helper and tests (`src/cli.rs`)
- [x] **Task 22.1.3**: G40 poll-based sleep replacement (`tests/integration_tests.rs`)

### Wave 1 end gate
- [x] `make all` passes serially. Expect unit 53 / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1. Actual: exit 0; unit 53 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1 (2026-11-08, at `ddb9ed2`).
- [x] N=1000 K=10 release line from 22.1.1 (expect about 2.2 ms): `N=1000 K=10 mean_us=2339.6 mode=diff` (2.34 ms; under the 10 ms bar, see report 22.1.1).
- [x] Sleeps replaced / left, with line numbers, from 22.1.3: 7 replaced (original lines 170, 226, 316, 409, 511, 597, 693); 3 left (616, 621, 931).

### Wave 2: Rate-Limit Scaffold
- [x] **Task 22.2.1**: G43 `RateLimiter`, wired into `handle_client`; H1 split in `tests/slow_consumer.rs` (`src/ratelimit.rs`, `src/lib.rs`, `src/daemon.rs`, `tests/slow_consumer.rs`)

### Wave 2 end gate
- [x] `make all` passes serially. Expect unit 56 / 2 ignored. Actual: exit 0; unit 56 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; clippy `--all-targets` clean.
- [x] `slow_consumer` passes 5 runs in a row. Actual: 5 of 5 passed.

### Wave 3: Docs
- [x] **Task 22.3.1**: ADR 0013 amendment, ADR 0015, catalog row (`docs/decisions/`)

### Wave 3 end gate
- [x] `make all` and `make test` pass serially. Actual: both exit 0; unit 56 / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; hook test passed.
- [x] `subscribe_golden` passes unchanged. Actual: passed; `tests/subscribe_golden.rs` not modified.

### Orchestrator steps
- [x] G41: `git branch --merged main` confirms the three branches, then `git branch -d` each. Actual: deleted `docs/adr-distillation` (was 2d8dcc7), `docs/agents-md-structure` (was f706a3b), `docs/task-4.1-checkoff` (was b30672c).
- [x] GOALS.md: G39–G43 moved to Completed with SHAs; G44 moved to Completed, with its push item still open (see below).
- [x] `git status` clean; `git worktree list` shows only `main`.
- [ ] Push to `origin/main`: not done. No remote writes without the user's go-ahead.

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

_Process note: this orchestration ran inline on `main` (one agent, one commit per task), not with spawned workers and worktrees. D8 (worktree base check) did not apply. The task specs, gates and commit discipline were followed._

### 22.1.1 (G42) -- done, `446d93c`
- Files: `src/types.rs` (derive line 30), `src/daemon.rs` (`broadcast_diff` comparison; grid at `measure_broadcast_cost_diff`, now `[5, 20, 50, 200, 1000]`).
- Tests: `cargo test --lib diff` 2 passed / 1 ignored; `subscribe_golden` 1 passed.
- Measurement: `N=1000 K=10 mean_us=2339.6 mode=diff lines=200 avg_line_bytes=171`. Plan expected 2165 to 2290 µs; this run is 2.34 ms. Still far under 10 ms.
- Deviations: none in code. The grid line is the only change in `mod tests`.

### 22.1.2 (G39) -- done, `d5eac18`
- Files: `src/cli.rs` (`fold_diff_line` next to `run_status_watch`; diff branch calls it; three tests in `mod tests`).
- Tests: `cargo test --lib test_fold_diff` 3 passed; `test_status_watch` 1 passed; `cargo check` no warnings.
- Deviations: the tests assert the folded map, not the rendered list (D4 chose a pure helper). Rendering is unchanged.

### 22.1.3 (G40) -- done, `ddb9ed2`
- Files: `tests/integration_tests.rs` only (import line; new helper `wait_for_listed` after `wait_for_socket`; 7 sleeps replaced).
- Tests: `cargo test --test integration_tests` 11 passed, 5 runs in a row.
- Replaced (original lines): 170, 226, 316, 409, 511, 597, 693. Left: 616, 621, 931.
- Deviations: all seven were registration waits, not status waits. Each polls `daemon.subscribe` until the last registered agent id is listed. The helper is named `wait_for_listed`, not `wait_for_status`. The plan's note on 409 and 511/597/693 said "status"; the code shows registrations.

### 22.2.1 (G43) -- done, `5d7ef87`
- Files: `src/ratelimit.rs` (new, with 3 tests), `src/lib.rs` (`pub mod ratelimit;`), `src/daemon.rs` (`handle_client` limiter and check), `tests/slow_consumer.rs` (send loop).
- Tests: `cargo test --lib ratelimit` 3 passed; `slow_consumer` 5 of 5 runs passed; wave gate `make all` exit 0.
- Deviations (H1): the 300 updates use three connections (register alone; 150 updates; 150 updates). The second batch starts only after the daemon lists `s149`, so the order is kept. The plan said "split" without a count.

### 22.3.1 (docs) -- done, `8a7f002`
- Files: `docs/decisions/0013-opt-in-diff-status-subscription.md` (Negative bullet only), `docs/decisions/0015-ipc-rate-limit-drop-and-log.md` (new), `docs/decisions/README.md` (row 0015).
- Gate: `make all` and `make test` exit 0.
- Deviations: ADR 0013 cites the three runs (2.17, 2.29, 2.34 ms) and not "about 2.2 ms" alone.

### Orchestrator
- G41 branches deleted (see wave board). GOALS.md updated: G39–G44 in Completed. G44's exit criterion "`origin/main` contains the work" is open: nothing is pushed.
