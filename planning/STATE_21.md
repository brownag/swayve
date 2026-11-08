# Orchestration 21 State: Diff-Path Hardening and Robustness Follow-ups (G29-G37)

Plan: `planning/EXECUTION_PLAN_21.md`
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Orchestration 20 complete. Code last changed at `b016cbc` (integration test) and `8d6ec67` (routing); later commits are docs and planning only.
- Last serial `make all` (Orchestration 20 wave 3 gate): unit 41 passed / 1 ignored; integration 11; Pi 3/3; hook test passed. No code has changed since, so this stands as the baseline.
- Pre-wave 1 counts: unit 41 / integration 11 / result ok.

## Wave Board

### Wave 1: Measurements, Readiness, Golden and Recovery Tests
- [ ] **Task 21.1.1**: G31 diff-path measurement (`src/daemon.rs` tests)
- [ ] **Task 21.1.2**: G34a `ensure_private_dir_for` (`src/cli.rs`)
- [ ] **Task 21.1.3**: G36 XDG warning (`src/config.rs`)
- [ ] **Task 21.1.4**: G37 startup sleeps to readiness checks (`tests/integration_tests.rs`)
- [ ] **Task 21.1.5**: G29 golden test (`tests/subscribe_golden.rs`, new)
- [ ] **Task 21.1.6**: G30 slow-consumer recovery test (`tests/slow_consumer.rs`, new)

### Wave 1 end gate
- [ ] `make all` passes serially. Expect 47 passed / 2 ignored unit; integration 11; new binaries 1 + 1. Actual: ___
- [ ] Release diff measurement at N=50 K=10 recorded here: ___ µs
- [ ] D3 decision (G32 included or not), per the 1 ms threshold: ___

### Wave 2: Socket Robustness
- [ ] **Task 21.2.1**: G35 non-UTF-8 skip policy and G34b `prepare_socket_path` (`src/daemon.rs`)

### Wave 2 end gate
- [ ] `make all` passes serially. Expect 50 passed. Actual: ___

### Wave 3: Conditional PartialEq and Watch Diff
- [ ] **Task 21.3.1**: G32 conditional `PartialEq` (`src/types.rs`, `src/daemon.rs` `broadcast_diff`). Runs only if D3 threshold met; otherwise record "skipped".
- [ ] **Task 21.3.2**: G33 `status --watch --diff` (`src/cli.rs`)

### Wave 3 end gate
- [ ] `make all` passes serially; `make test` runs Rust, Pi and hook suites. Expect 50 passed. Actual: ___
- [ ] `subscribe_golden` passes unchanged. Actual: ___

### Post-wave
- [ ] ADR 0012 amendment: foreign-owner paths tested via injected uid.
- [ ] ADR 0013 amendment: `to_value` outcome per D3.
- [ ] adr-distillation: G35 skip policy, if it qualifies.
- [ ] GOALS.md: G29–G37 moved to Completed with SHAs.

### Deferred (manual, not gated)
- None.

## Known limits (recorded at plan time)
- The external HUD (outside this repo) is not updated.
- G37 leaves event-ordering sleeps in place by design (D8). The count is reported in 21.1.4.
- G30 relies on a large payload to fill the socket buffer. A kernel buffer size change could alter the number of messages needed.

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._
