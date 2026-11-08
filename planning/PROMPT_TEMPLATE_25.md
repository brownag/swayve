# Orchestration 25 Worker Prompt Template

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your plan before you may write code.

## Task {{id}}: {{title}}
Read your task section `### Task {{id}}` in `planning/EXECUTION_PLAN_25.md` (the spec, files in scope, grep anchors, acceptance). That section is the spec. Do not work from memory of this template.

## Ground rules (non-negotiable)
- NEVER write code or run state-changing commands before your plan is approved. Phase 1 is read-only except for creating your micro-TODO file.
- NEVER touch files outside "Files in scope"; if required, STOP and report.
- NEVER go off-plan. Discovered work goes in a "## Discovered" section of your report and waits.
- NEVER assume. Resolve unknowns by reading code. Anything still ambiguous becomes numbered questions (max 5) in your plan.
- If a test fails twice for the same reason, stop and report. Do not thrash.
- Run only the targeted tests named in your spec. The orchestrator runs `make all` at wave end.
- Do not change non-test code unless your spec says so.
- FIRST CHECK: run `git log --oneline -1`. Your HEAD must be `main` (the commit the orchestrator names in your brief). If it is not, stop and report "stale base". Do not edit.

## Project conventions
- Rust, Edition 2024. Build: `cargo check`. Tests: `cargo test`. Full gate: `make all`.
- Pi client (`integrations/pi/`): `npm test`, `npm run build`. Its own `package.json`.
- Match surrounding style. No new dependencies.
- Zero compiler warnings. A new warning is a failure of your task.
- Scratch files go in `.development/` only.
- Commit discipline: see [git-workflow-standards](../../git-workflow-standards/SKILL.md). Conventional commit format, one sentence, no orchestration or wave numbers in the message. Do not commit: the orchestrator commits.
- Do not push or run any remote-write git command.
- Do NOT edit `planning/STATE_25.md`. The orchestrator records reports.

## [Worker] Analysis
Read: your task section in `planning/EXECUTION_PLAN_25.md`; the Decision Log and Hazard Notes; the grep anchors; the reference implementation the spec names.

## [Worker] Micro-planning
Write `planning/tasks/{{id}}.todo.md` in your worktree:
  # {{id}} -- {{title}}
  ## Goal        <one sentence>
  ## TODO items  <checkbox steps `- [ ]`, at most 12, including the explicit targeted-test steps>
  ## Notes       <facts discovered in Analysis, constraints honored, risks>
  ## Questions   <numbered, max 5, omit if none>
Your final message is the full contents of that file. END YOUR TURN.

## [Worker] Execution
Only after the orchestrator sends `APPROVED -- execute the plan as written`. Work the TODO strictly in order, flipping `- [ ]` to `- [x]`. Match surrounding style. Run only the targeted test named in the spec.

## [Worker] Reporting
Final message, 15 lines or fewer:
- status (done / blocked)
- files changed with line ranges
- exact test commands and pass/fail counts
- any numbers the spec asks you to copy (copy them verbatim)
- deviations from the approved plan, or "none"
- gotchas for dependent tasks
