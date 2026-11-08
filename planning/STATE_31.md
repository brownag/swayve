# Orchestration 31 State: reap daemon-spawned children (G82)

Plan: `planning/EXECUTION_PLAN_31.md` (approved by the user, 2026-11-08)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `2a4dd0a` (main; GOALS re-rank on top of `8d78ec0`).
- Pre-wave-1 step: serial `make all`, record exit code and warning count here. Last recorded green: Orchestration 30 (`8d78ec0`): unit 87 passed / 2 ignored; integration 14; zero warnings.
- Working tree before wave 1: clean.

## Wave Board

### Wave 1: reap daemon-spawned children (G82)
- [x] **Task 31.1.1** (`907588d`): `spawn_reaped` helper; hook and monitor launches route through it; `/proc` reap test

### Wave gates
- [x] After wave 1: serial `make all` exit 0, zero warnings (unit 88 passed / 2 ignored; integration 14). G82 exit criteria checked (monitor path by routing, see D3).

### Orchestrator steps
- [x] `git status` clean at each wave end (planning files excepted). Nothing pushed.
- [x] Move G82 to Completed in `planning/GOALS.md` with its commit hash.

## Deviations
- Task ran inline, not in a worker agent (as in orchestrations 29 and 30). No `planning/tasks/31.*.todo.md` file was written.
- The reap test was red twice: first on the missing helper, then with the helper but no wait (the child stayed as a zombie). Both reds are recorded below.
- Acceptance grep: expected two call sites. Three matches: the two launch sites plus the test's own call to the helper.

## Task Reports

### 31.1.1 (done, `907588d`)
- `src/daemon.rs` `spawn_reaped` (~335) spawns, returns the pid, and moves the `Child` into a detached `wait()` thread. `spawn_monitor_terminal` (~276) and `execute_lifecycle_hook` (~345) route through it. Ponytail note on the thread-per-child ceiling.
- Red: compile failure on the missing helper; then `/proc/<pid>` still present with no wait.
- Green: `test_spawned_child_is_reaped` passes; `lifecycle` 2 passed; `make check` exit 0, zero warnings.
- Gate: serial `make all` exit 0, zero warnings; unit 88 passed / 2 ignored; integration 14.
