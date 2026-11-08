# Orchestration 27 State: make all runs clippy with warnings denied

Plan: `planning/EXECUTION_PLAN_27.md` (approved 2026-11-08)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `00a4675` (main, pushed to `origin/main`).
- `make all` serial, exit 0: `.development/wave26-gate-make-all.log`. Unit 74 passed / 2 ignored; integration 11; Pi `npm test` 6/6; zero warnings.
- `cargo clippy --all-targets`: exit 0, zero warnings (`.development/wave26-clippy.log`).

## Wave Board

### Wave 1: clippy in the check gate (G72)
- [x] **Task 27.1.1**: `make check` runs `cargo clippy --all-targets -- -D warnings`; help text updated (`Makefile`) — `1f79de0`

### Wave 1 end gate
- [x] `make all` passes serially, zero warnings.
- [x] `make -n check` shows the clippy line.
- [x] `planning/GOALS.md`: G72 moved to Completed with its commit hash.

### Orchestrator steps
- [x] `git status` clean at wave end (planning files excepted). Nothing pushed.

## Deviations
- Task 27.1.1 ran inline by the orchestrator, not a spawned worker. One-line Makefile change in one file; no plan gate or worker spawn needed.

## Task Reports

## 27.1.1
- status: done
- files changed: `Makefile` L11 (help text), L31 (`cargo clippy --all-targets -- -D warnings` under `check:`). Commit `1f79de0`.
- `make -n check`: prints `cargo check` and `cargo clippy --all-targets -- -D warnings`.
- `make check`: exit 0, clippy `Checking swayve` with zero warnings.
- `make all` serial: exit 0, log `.development/wave27-gate-make-all.log`. Zero `warning`/`error` lines. Unit 74 passed / 2 ignored; integration 11; hook test assertions passed; Pi `npm test` in test target.
- deviations: none.
- gotchas: clippy now runs on every `make check` and `make all`; any new lint breaks the gate.
