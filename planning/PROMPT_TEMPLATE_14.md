# Wave 14 Worker Prompt Template (Orchestration 14)

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your plan before you may write code.

## Task {{id}}: {{title}}
{{task spec verbatim from planning/EXECUTION_PLAN_14.md}}

## Ground rules (non-negotiable)
- NEVER write code or run state-changing commands before your plan is approved.
- NEVER touch files outside "Files in scope"; if required, STOP and report.
- NEVER go off-plan; discovered work goes in "## Discovered" and waits.
- NEVER assume: resolve unknowns by reading code in Analysis; anything still ambiguous becomes numbered questions (max 5) in your plan report.
- If a check fails twice for the same reason, stop and report. Do not thrash.
- Run only the acceptance checks named in your spec.

## Project conventions
- ADRs live in `docs/decisions/`, follow the frontmatter and section layout of `0007-agent-hierarchy-and-sibling-focus.md`, and are indexed in `docs/decisions/README.md`.
- Scratch files go in `.development/` only.
- Commit discipline: see [git-workflow-standards](../../git-workflow-standards/SKILL.md). Conventional commit format, one sentence, no orchestration or wave numbers in the message.

## [Worker] Analysis
Read: your task section in `planning/EXECUTION_PLAN_14.md`; `planning/STATE_14.md`; the anchors named in the spec.

## [Worker] Micro-planning
Write `planning/tasks/{{id}}.todo.md`:
  # {{id}} -- {{title}}
  ## Goal        <one sentence>
  ## TODO items  - [ ] small, verifiable steps, including the acceptance checks
  ## Notes       <facts discovered in Analysis, constraints honored, risks>
  ## Questions   <numbered, max 5, omit if none>
Output the file contents as your report and END YOUR TURN.

## [Worker] Execution
Work the TODO strictly in order, flipping `- [ ]` to `- [x]`. Match surrounding style. Run only the acceptance checks named in the spec.

## [Worker] Reporting
Append to `planning/STATE_14.md` under `## {{id}}` (<=15 lines): status; files changed with line ranges; exact check commands and results; deviations from the approved plan (or "none"); gotchas.
