# Orchestration 32 State: open goals in series (G83, G73, G75)

Plan: `planning/EXECUTION_PLAN_32.md` (approved by the user, 2026-11-08)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `682085c` (main).
- Pre-wave-1 step: serial `make all`, record exit code and warning count here. Last recorded green: Orchestration 31 (`682085c`): unit 88 passed / 2 ignored; integration 14; zero warnings.
- Pre-wave-1 step: `cargo audit` records the tokio warning (RUSTSEC-2026-0386, tokio 1.53.1). Pi `npm test` 8/8 recorded.
- Working tree before wave 1: clean.

## Wave Board

### Wave 1: tokio past RUSTSEC-2026-0386 (G83)
- [x] **Task 32.1.1** (`cfab81d`): `cargo update -p tokio --precise 1.53.3`; `cargo audit` clean for tokio

### Wave 2: Pi tests in the build (G73)
- [x] **Task 32.2.1** (`e0ef35b`): `tsconfig.test.json` and `typecheck` script chained into `build`; red probe recorded and deleted

### Wave 3: design docs marked historical (G75)
- [x] **Task 32.3.1** (`200524b`): blockquote header in both design docs; diff is additions only

### Wave gates
- [x] After each wave: serial `make all` exit 0, zero warnings (wave 1: unit 88/2 ignored, integration 14; waves 2 and 3 exit 0).
- [x] Closing: G83, G73, G75 moved to Completed in `planning/GOALS.md`.

### Orchestrator steps
- [x] `git status` clean at each wave end (planning files excepted). Nothing pushed.

## Deviations
- Tasks ran inline, not in worker agents (as in orchestrations 29-31). No `planning/tasks/32.*.todo.md` files were written.
- Wave 2 red probe: the first run failed only because `tsconfig.test.json` did not exist yet (TS5058). It was re-run after the config was added and failed on a real type error (TS2322). Both results are recorded below.

## Task Reports

### 32.1.1 (done, `cfab81d`)
- `cargo update -p tokio --precise 1.53.3`. `Cargo.lock` diff is the tokio version only (2 lines).
- Red: `cargo audit` showed the RUSTSEC-2026-0386 warning at 1.53.1. Green: scan clean, no warning line.
- Gate: `make all` exit 0, zero warnings; unit 88 passed / 2 ignored; integration 14.

### 32.2.1 (done, `e0ef35b`)
- `integrations/pi/tsconfig.test.json` (extends base, `noEmit`, includes `tests/`). `package.json`: `build` is now `tsc && npm run typecheck`; `typecheck` is `tsc -p tsconfig.test.json`. `test` unchanged.
- Red: probe `const x: number = "not a number"` gave TS5058 before the config existed, then TS2322 once it did. Probe deleted.
- Green: `npm run build` 0, `npm run typecheck` 0, `npm test` 8/8 pass.
- Gate: `make all` exit 0, zero warnings.

### 32.3.1 (done, `200524b`)
- Blockquote header added to `planning/Sway Multi Agent Architecture Design.md` and `planning/Sway Multi-Agent Viewer Architecture.md`. `git diff --numstat` shows `2 0` for each.
- Link targets (ADRs 0003, 0004, 0009; README) confirmed on disk.
- Gate: `make all` exit 0, zero warnings.

### Closing
- G83, G73, G75 moved to Completed. Independent Backlog is empty.
