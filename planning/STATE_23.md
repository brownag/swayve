# Orchestration 23 State: Open Backlog Bundle

Plan: `planning/EXECUTION_PLAN_23.md` (approved 2026-11-08; executed inline, one commit per task)
Last Updated: 2026-11-08 (complete)

## Baseline (before Wave 1)
- Starting commit: `48b8d61` (main). Commits from this plan are local; not pushed.
- Last serial `make all` (at `48b8d61`): unit 60 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; Pi and hook tests passed; zero warnings.
- Clippy (`cargo clippy --all-targets`): no warnings.

## Wave Board

### Wave 1: Tests, Spawn Plan, CI
- [x] **Task 23.1.1**: G46 socket-level over-limit test (`tests/rate_limit_socket.rs`)
- [x] **Task 23.1.2**: G50 poll hook flag files (`tests/integration_tests.rs`)
- [x] **Task 23.1.3**: G57 spawn plan pure function (`src/cli.rs`)
- [x] **Task 23.1.4**: G51 CI workflow file (`.github/workflows/ci.yml`)

### Wave 1 end gate
- [x] `make all` passes serially. Actual: exit 0; unit 64 / 2 ignored; integration 11; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1 (`0d985c7`-era tree, at `602ffa1`).

### Wave 2: Depth Test, README
- [x] **Task 23.2.1**: G55 depth-limit test can fail (`src/daemon.rs`)
- [x] **Task 23.2.2**: G47 Socket Contract bullets (`README.md`)

### Wave 2 end gate
- [x] `make all` passes serially. Actual: exit 0.
- [x] `cargo test --lib depth` passes; removing the depth check fails it (checked by hand). Actual: failed with the check removed, passes when restored.

### Wave 3: Barrier, Diff Golden
- [x] **Task 23.3.1**: G49 barrier replaces focus sleeps (`src/daemon.rs`, `tests/integration_tests.rs`)
- [x] **Task 23.3.2**: G56 `subscribe_diff` golden, captured before change (`tests/diff_golden.rs`)

### Wave 3 end gate
- [x] `make all` passes serially. Actual: exit 0; unit 64 / 2 ignored.
- [x] Integration suite passes 5 runs. Actual: 5 of 5, 11 passed each.
- [x] `diff_golden` captured at SHA `f1b5a37`, before any change.

### Wave 4: DiffFrame
- [x] **Task 23.4.1**: G56 `DiffFrame` shared type (`src/daemon.rs`, `src/cli.rs`, `src/types.rs`)

### Wave 4 end gate
- [x] `make all` passes serially; `diff_golden` and `subscribe_golden` unchanged. Actual: exit 0; both pass unchanged.

### Wave 5: Window Lifecycle
- [x] **Task 23.5.1**: G54 window lifecycle in `DaemonState` (`src/daemon.rs`)

### Wave 5 end gate
- [x] `make all` and clippy pass serially. Actual: exit 0; unit 67 / 2 ignored; clippy 0 warnings.
- [x] Integration suite passes 5 runs. Actual: 5 of 5.

### Orchestrator steps
- [x] `planning/GOALS.md`: G46, G47, G49, G50, G51, G54, G55, G56, G57 moved to Completed with SHAs.
- [x] `git status` clean. Nothing pushed.

### Deferred (manual, not gated)
- G52, G58: deferred by trigger, not in this plan.
- G53, G25, G15: external, prompts in `planning/external/`.

## Known limits (recorded at execution)
- G51 is a file only. It runs after a push, which needs the user's go-ahead. The job runs `npm ci` for the Pi client first, because `make test` needs its `node_modules`.
- G54 window path: the auto-monitor branch runs only with `/proc` (process-tree audit). It has no unit test; it's covered by the integration suite and by the register-path tests.
- G54 order change: the register path runs the on_agent_spawn hook before the monitor terminal. Before, the monitor spawned first. No test asserts that order.
- G56 type change: a diff line that is not a valid frame is rejected whole (previously its valid fields were folded). Non-JSON lines are still ignored, as before.
- G49 barrier: `DaemonEvent::Sync` is internal; the wire format is unchanged.
- G46 test sends 250 lines; the delivered count range is 200 to 205 (refill during the flood).

## Task Reports

_Process note: executed inline by the orchestrator on `main`, one commit per task (D4). No worker agents or worktrees._

- **23.1.1 (G46)** `b4dd9fa`. New `tests/rate_limit_socket.rs`. 250 lines; 200 to 205 delivered; poll recovers. 5 of 5 runs pass.
- **23.1.2 (G50)** `0d985c7`. Hook test polls both flag files (spawn file must hold both ids). 5 of 5 runs pass.
- **23.1.3 (G57)** `fe98801`. `spawn_plan` pure fn in `src/cli.rs`, `SpawnPlan` struct. 4 unit tests. Behavior unchanged.
- **23.1.4 (G51)** `602ffa1`. `.github/workflows/ci.yml`: checkout, stable Rust, Node 20, `npm ci` for Pi, `make all`. YAML parses. Not pushed.
- **23.2.1 (G55)** `8460157`. Parent lookup is a closure. Depth test uses a 10-pid chain, agent at 105. Mutation check: test fails with the depth check removed.
- **23.2.2 (G47)** `f1b5a37`. Two bullets in README Socket Contract, linked to ADR 0016.
- **23.3.2 (G56 golden)** `1b1c3b0`. `tests/diff_golden.rs` pins the bytes at `f1b5a37`. Keys are alphabetical.
- **23.3.1 (G49)** `51c62ca`. `DaemonEvent::Sync(oneshot)`, replies in the loop. Focus test uses `sync_events` in place of two sleeps. 5 of 5 runs pass.
- **23.4.1 (G56)** `9843d6f`. `DiffFrame` in `src/types.rs`; `to_line` serializes through `Value`. Golden unchanged. Plan spec corrected (field order).
- **23.5.1 (G54)** `55489c7`. `on_window_created`, `on_window_destroyed`, `MonitorRequest`, `auto_monitor_wanted`. `register_agent` returns `Option<MonitorRequest>`. Clippy clean.
