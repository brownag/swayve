# Orchestration 20 State: Public-Release Readiness: Diff Subscriptions (G9) and XDG Config Path (G10)

Plan: `planning/EXECUTION_PLAN_20.md`
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Orchestration 19 complete. HEAD `e8c1610` (G28 README fix). Working tree clean.
- Last serial `make all` at `e8c1610`: 35 unit / 10 integration / Pi 3/3 / hook test passed. No Rust code has changed since that run (the G28 change touched README only), so this stands as the baseline.
- Pre-wave 1 counts: unit 35 / integration 10 / result ok.

## Wave Board

### Wave 1: Measurement and Config Path
- [ ] **Task 20.1.1**: G9 broadcast-cost measurement (ignored test, `src/daemon.rs` tests)
- [ ] **Task 20.1.2**: G10 `config_path_from` and tests (`src/config.rs`)
- [ ] **Task 20.1.3**: G10 docs (README Configuration note, ADR 0010 amendment)

### Wave 1 end gate
- [ ] `make all` passes serially. Expect 39 unit (35 + 4) and 10 integration. Actual: ___
- [ ] Orchestrator reviews 20.1.1 numbers and records the D7 decision here: ___

### Wave 2: Diff Core (blocked until D7 recorded)
- [ ] **Task 20.2.1**: `SubscribeDiff` message and diff broadcast core (`src/types.rs`, `src/daemon.rs`)

### Wave 2 end gate
- [ ] `make all` passes serially. Expect 39 unit. Actual: ___

### Wave 3: Routing, Integration Test, Docs
- [ ] **Task 20.3.1**: `handle_client` routing and snapshot-after-drop unit tests
- [ ] **Task 20.3.2**: Integration test for diff stream (depends on 20.3.1)
- [ ] **Task 20.3.3**: G9 docs (README Socket Contract, ADR 0008 amendment)

### Wave 3 end gate
- [ ] `make all` passes serially; `make test` runs Rust, Pi and hook suites. Expect 41 unit, 11 integration. Actual: ___

### Post-wave
- [ ] adr-distillation: ADR 0013 (diff subscription protocol), index updated.
- [ ] GOALS.md: G10 and G9 moved to Completed.

### Deferred (manual, not gated)
- None.

## Known limits (recorded at plan time)
- The external HUD (outside this repo) is not updated. The diff stream is documented for it, but no in-repo client consumes it.
- `status --watch` stays on the full stream (D1).

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._
