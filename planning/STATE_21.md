# Orchestration 21 State: Diff-Path Hardening and Robustness Follow-ups (G29-G37)

Plan: `planning/EXECUTION_PLAN_21.md`
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Orchestration 20 complete. Code last changed at `b016cbc` (integration test) and `8d6ec67` (routing); later commits are docs and planning only.
- Last serial `make all` (Orchestration 20 wave 3 gate): unit 41 passed / 1 ignored; integration 11; Pi 3/3; hook test passed. No code has changed since, so this stands as the baseline.
- Pre-wave 1 counts: unit 41 / integration 11 / result ok.
- Re-verified by orchestrator at start of run: `make all` green (unit 41 passed / 1 ignored; integration 11; Pi 3/3; hook test passed).

## Wave Board

### Wave 1: Measurements, Readiness, Golden and Recovery Tests
- [x] **Task 21.1.1**: G31 diff-path measurement (`src/daemon.rs` tests). Commit `3c2475f`.
- [x] **Task 21.1.2**: G34a `ensure_private_dir_for` (`src/cli.rs`). Commit `ab74cba`. Deviation: added shared test helper `epdf_dir` inside `mod tests`.
- [x] **Task 21.1.3**: G36 XDG warning (`src/config.rs`). Commit `7d003c7`.
- [x] **Task 21.1.4**: G37 startup sleeps to readiness checks (`tests/integration_tests.rs`). Commit `82f8355`. 11 startup sleeps replaced; 10 event-ordering sleeps left at lines 170, 226, 316, 409, 511, 597, 616, 621, 693, 931.
- [x] **Task 21.1.5**: G29 golden test (`tests/subscribe_golden.rs`, new). Commit `cab1daf`. Accepts two byte-exact orderings (`HashMap` order is not stable).
- [x] **Task 21.1.6**: G30 slow-consumer recovery test (`tests/slow_consumer.rs`, new). Commit `b78343b`.

Process note: all six worktrees were created from a stale base (`298d6db`). Four had finished and two were still running; each was fast-forwarded to `main` before its re-plan. Workers did not append to this file; the orchestrator recorded these entries.

### Wave 1 end gate
- [x] `make all` passes serially. Actual: unit 47 passed / 2 ignored; integration 11 passed; `slow_consumer` 1 passed; `subscribe_golden` 1 passed; Pi 3/3; hook test passed.
- [x] Release diff measurement at N=50 K=10 recorded here: **621.4 µs** (debug 3253.1 µs). Full grid, release: N=5 K=1 8.6; N=5 K=10 69.2; N=20 K=1 26.9; N=20 K=10 251.3; N=50 K=1 66.4; N=200 K=1 252.3; N=200 K=10 2386.6 (all µs, 200 lines of 171 bytes each).
- [x] D3 decision: 621.4 µs is below the 1 ms threshold. **21.3.1 (G32) is skipped; G32 recorded as "not included".** ADR 0013 records the measured numbers.

### Wave 2: Socket Robustness
- [x] **Task 21.2.1**: G35 non-UTF-8 skip policy and G34b `prepare_socket_path` (`src/daemon.rs`). Commit `235adf3`. Deviation: `prepare_socket_path` is `async fn` (the fs calls are `tokio::fs`); callers `.await` it. Refusal message prints `path.display()`.

### Wave 2 end gate
- [x] `make all` passes serially. Actual: unit 50 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; Pi 3/3; hook test passed.

### Wave 3: Conditional PartialEq and Watch Diff
- [x] **Task 21.3.1**: G32 conditional `PartialEq`. **Skipped** per D3 (621.4 µs < 1 ms at N=50 K=10). G32 recorded as "not included".
- [x] **Task 21.3.2**: G33 `status --watch --diff` (`src/cli.rs`). Commit `459c583`. `cargo check` clean; `cargo test --lib test_status_watch` 1 passed (covers both new parse cases).

### Wave 3 end gate
- [x] `make all` passes serially; `make test` runs Rust, Pi and hook suites. Actual: unit 50 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; Pi 3/3; hook test passed.
- [x] `subscribe_golden` passes unchanged (not edited in Wave 3). Actual: 1 passed.

### Post-wave
- [x] ADR 0012 amendment: foreign-owner paths tested via injected uid (`docs/decisions/0012-runtime-socket-private-dir-contract.md`).
- [x] ADR 0013 amendment: `to_value` outcome per D3 (kept; measured 621 µs at N=50 K=10 in release).
- [ ] adr-distillation: G35 skip policy, if it qualifies. **Not run in this orchestration.** It is a candidate, so it stays open.
- [x] GOALS.md: G29–G37 moved to Completed with SHAs; G32 recorded as "not included".

### Process notes
- Six Wave 1 worktrees and the 21.2.1 and 21.3.2 worktrees were created from a stale base (`298d6db`). Each was fast-forwarded to `main` before work started (one worker was re-run after its worktree was removed). Confirm the base before approving any plan.
- Workers did not write to this file; the orchestrator recorded all entries.

### Deferred (manual, not gated)
- None.

## Known limits (recorded at plan time)
- The external HUD (outside this repo) is not updated.
- G37 leaves event-ordering sleeps in place by design (D8). The count is reported in 21.1.4.
- G30 relies on a large payload to fill the socket buffer. A kernel buffer size change could alter the number of messages needed.

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._
