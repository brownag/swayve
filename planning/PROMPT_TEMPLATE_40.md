# Worker Prompt Template — Orchestration 40

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your micro-plan before implementation or live runtime action.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules
- Read `planning/EXECUTION_PLAN_40.md`, `planning/STATE_40.md`, root `AGENTS.md`, and `crates/swayve-hud/AGENTS.md` for HUD work.
- Preserve the existing modification to `planning/RESEARCH_OPENCODE_INTEGRATION.md`. Inspect Git status before and after; do not stage unrelated changes.
- Do not write implementation code, run live OpenCode/Sway interactions, or trigger provider work before the task micro-plan is approved.
- Stay within the task's explicit files-in-scope list. Do not edit other files or add dependencies.
- Never infer a session/window match from a project directory, title, session ID, plugin PID, or server PID unless Wave 1 provides exact evidence for that identity contract.
- Do not read message history/content to discover session identity. Do not record raw session IDs, event payloads, prompts, secrets, or full logs in durable files.
- User-assisted live validation is only Task 40.4.1. The user identifies sessions and triggers any turn/tool action. Do not initiate model prompts, write workspace files, or operate on unselected sessions/windows.
- If Wave 1 cannot prove exact association or authoritative retirement evidence, stop before implementation and report the boundary for a user decision.
- Keep OpenCode parent-session `parentID` separate from Swayve's explicit hosting-window association. Do not change `parent_agent_id` semantics.
- If work appears to require superseding an accepted ADR, stop and ask for explicit approval naming the ADR.
- Planning artifacts are tracked; make planning-only `docs(planning):` commits separate from implementation. Never stage pre-existing unrelated changes.
- Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md) for local commits; no push, pull request, comments, remote changes, or other remote actions.
- If a test fails twice for the same reason, stop and report.

## Micro-planning
Create `planning/tasks/{{id}}.todo.md` with:
- Goal (one sentence)
- TODO items (at most 12, with exact targeted test commands)
- Notes (facts, constraints, and risks)
- Questions (numbered, at most 5; omit if none)

Output the micro-TODO and stop for orchestrator review. Do not execute until the explicit message `APPROVED -- execute the plan as written`.

## Execution
After approval, execute the TODO strictly in order and touch only the task's scoped files. Run only the targeted tests in the task spec. Make a focused local Conventional Commit for verified implementation work, separate from planning artifacts. Do not perform remote Git operations.

## Reporting
Append a concise task report (no more than 15 lines) to `planning/STATE_40.md`: status, files changed, local commit ID(s), exact tests and pass/fail counts, deviations (or “none”), and gotchas for dependent tasks.
