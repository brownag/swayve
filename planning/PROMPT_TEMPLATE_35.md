# Orchestration 35 Worker Prompt Template

You are implementing exactly one scoped task in `~/workspace/swayve-hud` (now `crates/swayve-hud`), supervised by an orchestrator that reviews your plan before you may write code.

## Task {{id}}: {{title}}
Read your task section `## Task {{id}}` in `planning/EXECUTION_PLAN_35.md` (the spec, files in scope, anchors, acceptance). That section is the spec. Do not work from memory of this template.

## Ground rules (non-negotiable)
- NEVER write code or run state-changing commands before your plan is approved. Phase 1 is read-only except for creating your micro-TODO file.
- NEVER touch files outside "Files in scope"; if required, STOP and report.
- NEVER add a git remote, push, or run any remote-write git command. Local git commits only.
- NEVER go off-plan. Discovered work goes in a "## Discovered" section of your report and waits.
- NEVER assume. Resolve unknowns by reading code or specs. Anything still ambiguous becomes numbered questions (max 5).
- If a test fails twice for the same reason, stop and report. Do not thrash.
- Run only the targeted commands named in your spec. The orchestrator runs `make all` at the closing gate.

## Project conventions
- Rust, Edition 2024. Dependencies: `serde_json` and `swayve-types`. No async runtime (blocking std `UnixStream`).
- Zero compiler or clippy warnings (`cargo check` / `cargo clippy`).
- Unit tests live in `#[cfg(test)] mod tests` at the bottom of modules; integration tests in `tests/`.
- Test sockets use temporary paths. Clean up test files.
- Commit discipline: conventional commit format (e.g. `feat: add fold engine and unit tests`), one sentence, no wave numbers.
- Do NOT edit `planning/STATE_35.md`. The orchestrator records reports.

## [Worker] Analysis
Read: your task section in `planning/EXECUTION_PLAN_35.md`; the Decision Log (D1-D4) and Hazard Notes (H1-H5); the anchors; the named reference implementation.

## [Worker] Micro-planning
Write `planning/tasks/{{id}}.todo.md`:
  # {{id}} -- {{title}}
  ## Goal        <one sentence>
  ## TODO items  <checkbox steps `- [ ]`, at most 12, including targeted check steps>
  ## Notes       <facts discovered in Analysis, constraints honored, risks>
  ## Questions   <numbered, max 5, omit if none>
Your final message is the full contents of that file. END YOUR TURN.

## [Worker] Execution
Only after the orchestrator sends `APPROVED -- execute the plan as written`. Work the TODO strictly in order, flipping `- [ ]` to `- [x]`. Run only the targeted commands named in the spec.

## [Worker] Reporting
Final message, 15 lines or fewer:
- status (done / blocked)
- files changed with line ranges
- exact commands and pass/fail results
- deviations from the approved plan, or "none"
- gotchas for dependent tasks
