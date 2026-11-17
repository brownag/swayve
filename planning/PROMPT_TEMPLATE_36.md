# Worker Prompt Template — Orchestration 36

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your plan before you may write code.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules
- Do not make implementation changes or run implementation state-changing commands before the orchestrator approves your micro-plan.
- After approval, work only within the task's explicit files-in-scope list. If a required file is out of scope, stop and report.
- Do not assume OpenCode event names or payload shapes; use the first-party docs/types cited in the plan.
- Follow ADR 0009: this is a thin socket client; do not add unrelated OpenCode SDK behavior.
- Keep Swayve socket failures best-effort and non-fatal to OpenCode.
- Do not modify daemon/Rust code, existing integrations, or the root Makefile.
- For Git branch, status, and focused local commits, follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md). No remote operations are authorized.
- If a test fails twice for the same reason, stop and report.

## Analysis
Read root `AGENTS.md`, the named plan task, relevant state reports, the named reference implementation, relevant ADRs, and the official OpenCode documentation. Inspect only required code anchors.

## Micro-planning
Create `planning/tasks/{{id}}.todo.md` with Goal, TODO items (maximum 12 steps), Notes, and numbered Questions (maximum 5; omit if none). Include exact targeted test commands. Output the file and stop for approval.

## Execution
After explicit orchestrator approval, execute the TODO in order, check off completed steps, and run only the targeted test commands in the task spec. Commit verified implementation work locally in a focused Conventional Commit. Keep planning artifact updates out of implementation commits.

## Reporting
Append a concise report under the task ID in `planning/STATE_36.md`: status, files changed, commit ID, exact tests and results, deviations, and gotchas for dependent tasks.
