# Orchestration 14 State: Record Focus-State Decision and Correct Focus-Order ADR (G2)

Plan: `planning/EXECUTION_PLAN_14.md`
Last Updated: 2026-11-02

## Baseline (before Wave 1)
- Orchestration 13 complete, committed as `436b190`.
- `cargo test`: 22 unit passed, 10 integration passed (Orchestration 13 gate).
- No code changes in this orchestration.

## Wave Board

### Wave 1: Focus-State ADR and Focus-Order Correction
- [x] **Task 14.1.1**: Amend ADR 0007; create ADR 0011; add README row

### Wave 1 end gate
- [x] Acceptance checks in the spec pass
- [x] `cargo check` passes (sanity)

### 14.1.1
- Status: done. Micro-plan approved (with two clarifications), then executed.
- Files: `docs/decisions/0007-agent-hierarchy-and-sibling-focus.md` (Rules section); `docs/decisions/0011-focus-state-from-sway-window-events.md` (new, 44 lines); `docs/decisions/README.md` (0011 row).
- Checks: all six acceptance checks pass; `cargo check` clean. Re-run by orchestrator.
- Deviations: 0011 has a "See also" link to 0007 in Validation (not in spec; harmless). Daemon-initiated focus listed as Neutral.

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._
