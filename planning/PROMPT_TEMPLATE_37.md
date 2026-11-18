# Worker Prompt Template — Orchestration 37

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your micro-plan before any implementation or live OpenCode action.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules
- Read `planning/EXECUTION_PLAN_37.md`, `planning/STATE_37.md`, root `AGENTS.md`, relevant Orchestration 36 reports, the named files, and ADRs 0004/0006/0009/0012.
- Do not run OpenCode or make implementation/state-changing changes until the orchestrator approves your micro-plan.
- Stay within the task's explicit files-in-scope list. If the smoke test requires an out-of-scope change, stop and report it.
- The user authorized one harmless read-only prompt, which may incur provider usage. Do not write files, change provider/auth settings, or expose secrets/repository-sensitive content.
- Preserve user work. Check Git status before and after the live run; do not overwrite or clean unrelated changes.
- Keep the integration a thin socket client per ADR 0009. Do not add dependencies, change daemon/Rust code, edit the root Makefile, or implement `parentID` mapping (reserved for G90).
- Record observations, not raw logs or transcripts.
- Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md) for Git safety. If implementation files change and verification passes, make a focused local implementation commit. Keep tracked planning updates in a separate `docs(planning):` commit; do not commit temporary smoke output. No remote operations are authorized.
- If a test fails twice for the same reason, stop and report.

## Micro-planning
Create `planning/tasks/{{id}}.todo.md` with Goal, no more than 12 TODO items, Notes, and numbered Questions (max 5; omit if none). Include the exact live smoke procedure, focused test command, observation method, and cleanup. Then stop for orchestrator review.

## Execution
After explicit approval, perform the live check and only the focused repairs needed to meet acceptance. Run the targeted OpenCode test if implementation changes. The orchestrator owns the final full `make all` gate. Commit implementation changes only if files changed and verification passes; commit tracked planning updates separately with `docs(planning):`. Never commit temporary smoke output.

## Reporting
Append a concise report under Task 37.1.1 in `planning/STATE_37.md`: status, observed runtime/version, events and identity fields verified, files changed, commit ID, exact tests/commands, deviations/blockers, and G90 gotchas.
