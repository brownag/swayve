# Orchestration 29 State: one argv path for all three launches (G80 residual)

Plan: `planning/EXECUTION_PLAN_29.md` (draft, pending approval)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `2b6898f` (main).
- Pre-wave-1 step: serial `make all`, record exit code and warning count here. Last recorded green: Orchestration 28 (`b37b0e2`): unit 83 passed / 2 ignored; integration 12; Pi `npm test` 8/8; zero warnings.
- Working tree before wave 1: clean (planning files committed).

## Wave Board

### Wave 1: spawn_plan through expand_terminal_args (G80)
- [ ] **Task 29.1.1**: `spawn_plan` builds argv through `expand_terminal_args`; D2 fallback kept

### Wave 2: agent-launch argv-capture test (G80)
- [ ] **Task 29.2.1**: `test_spawn_argv_capture` runs the real binary and asserts exact argv

### Wave gates
- [ ] After wave 2: serial `make all` exit 0, zero warnings. G80 exit criteria checked.

### Orchestrator steps
- [ ] `git status` clean at each wave end (planning files excepted). Nothing pushed.

## Deviations
- None yet.

## Task Reports
