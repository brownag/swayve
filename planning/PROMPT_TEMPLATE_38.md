# Worker Prompt Template — Orchestration 38

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your micro-plan before implementation or live OpenCode actions.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules
- Read `planning/EXECUTION_PLAN_38.md`, `planning/STATE_38.md`, root `AGENTS.md`, relevant Orchestration 36/37 records, named files, and ADRs 0004/0006/0009/0012.
- Do not edit implementation files or run OpenCode before your micro-plan is approved.
- Stay within the task's explicit files-in-scope list. Stop and report if out-of-scope changes are required.
- One harmless read-only provider-backed prompt is authorized and may incur usage. Do not write files or change provider/auth configuration.
- Preserve user work. Check Git status before and after the live run.
- Diagnostics are temporary: remove instrumentation/output after the run unless the confirmed focused fix requires a durable change.
- Keep the integration a thin socket client. Do not add dependencies, change daemon/Rust code, edit the root Makefile, or implement parentID mapping.
- Record observations only; never commit raw logs, transcripts, or secrets.
- Follow `git-workflow-standards` for Git safety. Commit verified implementation changes separately from tracked planning updates using Conventional Commits. No remote operations.
- If a test fails twice for the same reason, stop and report.

## Micro-planning
Create `planning/tasks/{{id}}.todo.md` with Goal, at most 12 TODO items, Notes, and numbered Questions (max 5; omit if none). Include the exact smoke procedure, focused test command, observation method, instrumentation cleanup, and verification. Then stop for orchestrator review.

## Execution
After explicit orchestrator approval, perform the trace and only the focused repairs needed to resolve an evidenced plugin defect. Run the targeted OpenCode test if implementation changes. The orchestrator owns any full `make all` gate. Keep implementation and planning commits separate; never commit temporary output.

## Reporting
Append a concise report under Task 38.1.1 in `planning/STATE_38.md`: status, runtime/version, observed events and identity fields, IPC/daemon evidence, files changed, commit IDs, exact commands/tests, deviations/blockers, and G90 gotchas.
