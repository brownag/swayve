# Orchestration 26 State: Pi test sockets out of cwd, spawn_plan request struct

Plan: `planning/EXECUTION_PLAN_26.md` (draft, awaiting approval)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `9a91b55` (main). Last full gate in STATE_25: `make all` green, exit 0, `# fail 0`. Unit 72 passed / 2 ignored; integration 11; single-test suites 1 each; Pi `npm test` 7/7 at `af5176e`; zero warnings; clippy clean.
- Pre-wave-1 `make all` re-run serially at `9a91b55`: exit 0. Unit 74 passed / 2 ignored (STATE_25 said 72; recorded as measured now); integration 11; single-test suites 1 each; Pi `npm test` 7/7; log at `.development/wave26-baseline-make-all.log`.

## Wave Board

### Wave 1: Pi test sockets (G63), spawn_plan request struct (G64)
- [x] **Task 26.1.1**: G63, Pi test writes no socket files into `process.cwd()` (`integrations/pi/tests/swayve-client.test.ts`)
- [x] **Task 26.1.2**: G64, `spawn_plan` takes `SpawnRequest<'_>`, allow attribute removed (`src/cli.rs`)

### Wave 1 end gate
- [x] `make all` passes serially, zero warnings, clippy clean (exit 0; Pi 6/6; unit 74 + 2 ignored; log `.development/wave26-gate-make-all.log`).
- [x] `ls integrations/pi/*.sock` empty; `too_many_arguments` allow gone from `src/cli.rs`.
- [x] `planning/GOALS.md`: G63 and G64 moved to Completed with commit hashes; G65 moved to Pruned.

### Orchestrator steps
- [x] Commits are local only: `9b31e73` (G64), `bf5feaf` (G63). Nothing pushed.

## Deviations
- D4 corrected: Pi count is 6 after the `cleanup` test is removed, not 5 (the file had 7 tests).
- Worktree isolation skipped: disjoint files, and the Pi worktree would lack `node_modules`.
- Plan status: approved by the orchestrator under the `run plan 26` request.

## Task Reports

## 26.1.1
- done. `integrations/pi/tests/swayve-client.test.ts`: removed `test.sock` path and `cleanup` test; `dummy.sock` in `mkdtemp`. `npm test` 6/6, build exit 0, no `*.sock` left. Commit `bf5feaf`.

## 26.1.2
- done. `src/cli.rs`: `SpawnRequest<'a>`, `spawn_plan(req)`, allow removed, 1 prod + 8 test call sites. `cargo test spawn_plan` 7/7, clippy 0 warnings. Commit `9b31e73`.

_Workers append their reports here, under `## <task id>`._
