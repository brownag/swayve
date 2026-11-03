# Orchestration 15 State: Config Parse Errors Reported; Lowest-Survivor Rehome Covered (G3, G4)

Plan: `planning/EXECUTION_PLAN_15.md`
Last Updated: 2026-11-02

## Baseline (before Wave 1)
- Orchestration 14 complete, committed as `874d7db`. No code changed since Orchestration 13 (`436b190`).
- `cargo test` (last recorded at Orchestration 13): 22 unit passed, 10 integration passed.

## Wave Board

### Wave 1: Config Parse Errors and Lowest-Survivor Rehome Test
- [x] **Task 15.1.1**: Report malformed config on stderr; add parse-failure unit test (`src/config.rs`)
- [x] **Task 15.1.2**: Add three-child rehome test (`src/layout.rs`)

### Wave 1 end gate
- [x] `make all` passes serially (check, test, build) -- 24 unit + 10 integration passed, release build OK

### Deferred (manual, not gated)
- [ ] Malformed config, live: bad `~/.config/swayve/config.toml`, run `swayve status`, confirm stderr message and command still runs.
- [ ] Live-Sway focus check from plan 13 (outstanding).

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

### 15.1.1
- Status: done. Micro-plan approved, then executed.
- Files: `src/config.rs` (`parse_config_file` helper; `load_config` prints `[config] ...; using defaults` on parse/read error; NotFound stays silent).
- Tests: `cargo test --lib config::` 4 passed. Orchestrator re-ran and reviewed the diff.
- Deviations: none.

### 15.1.2
- Status: done. Micro-plan approved (after orchestrator confirmed the lowest-con_id rule in code), then executed.
- Files: `src/layout.rs` (new test `test_window_destroyed_rehomes_to_lowest_survivor`, ~lines 359-379).
- Tests: targeted test 1 passed. Full gate `make all`: 24 unit + 10 integration passed, release build OK.
- Deviations: none. No non-test code changed.
