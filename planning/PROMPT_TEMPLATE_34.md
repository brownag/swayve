# Orchestration 34 Worker Prompt Template

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your plan before you may write code.

## Task {{id}}: {{title}}
Read your task section `## Task {{id}}` in `planning/EXECUTION_PLAN_34.md` (the spec, files in scope, anchors, acceptance). That section is the spec. Do not work from memory of this template.

## Ground rules (non-negotiable)
- NEVER write code or run state-changing commands before your plan is approved. Phase 1 is read-only except for creating your micro-TODO file.
- NEVER touch files outside "Files in scope". If one is required, STOP and report.
- NEVER go off-plan. Discovered work goes in a "## Discovered" section of your report and waits.
- NEVER assume. Resolve unknowns by reading code. Anything still ambiguous becomes numbered questions (max 5).
- If a test fails twice for the same reason, stop and report. Do not thrash.
- Run only the targeted commands named in your spec. The orchestrator runs `make test` and the acceptance gate.
- FIRST CHECK: `git log --oneline -1` must show a commit containing `773f529`. Otherwise stop and report "stale base".
- Do NOT create any `.js` file. Use `.mjs` (`.gitignore` ignores `*.js`).
- Do NOT push, and do NOT run any remote-write git command (the user's global rule forbids it). Do NOT reply to or resolve GitHub review threads.
- Do NOT edit `planning/STATE_34.md`. The orchestrator records reports.

## Project conventions
- Node 22, `node:test`, no dependencies. Files: `integrations/antigravity/sidecar.mjs`, `integrations/antigravity/sidecar.test.mjs`.
- Scratch files go in `.development/` only (gitignored).
- Commit discipline: see [git-workflow-standards](../../git-workflow-standards/SKILL.md). Do not commit: the orchestrator commits.
- Repo conventions: `AGENTS.md`.

## [Worker] Analysis
Read: your task section in `planning/EXECUTION_PLAN_34.md`; the Decision Log (D1-D5), the Recon log (R1-R7), and the Hazards in Task 34.1.1; the anchors; the existing style of `integrations/antigravity/sidecar.test.mjs` (the first 40 lines and the focus test only).

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
- exact commands and pass/fail results (include the pass count from `node --test`)
- deviations from the approved plan, or "none"
- gotchas for dependent tasks
