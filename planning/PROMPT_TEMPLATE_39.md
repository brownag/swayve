# Worker Prompt Template — Orchestration 39

You are implementing exactly one scoped task, supervised by an orchestrator that reviews your micro-plan before implementation or live runtime action.

## Task {{id}}: {{title}}
{{task spec verbatim}}

## Ground rules
- Read `planning/EXECUTION_PLAN_39.md`, `planning/STATE_39.md`, root `AGENTS.md`, relevant Orchestration 36/38 records, task grep anchors, and ADRs 0004/0006/0009/0012.
- Do not edit implementation files or run OpenCode before your micro-plan is approved.
- Stay within task file scope. For Task 39.1.1, `.opencode/plugins/swayve.ts` edits are temporary only and must be reverted; never delete unrelated `.development/` files.
- Task 39.1.1 is authorized to run exactly one read-only model turn with `--model openai/gpt-6-luna`. Do not use another model or run a second turn.
- Do not write raw OpenCode session IDs to trace files or reports; use a fixed alias such as `probe-session` and correlate raw IDs only transiently in memory.
- Proceed to Task 39.2.1 only if the Wave 1 live proof is positive and recorded; otherwise stop and report the missing boundary.
- Preserve user work and inspect Git status before/after. The existing research note change is out of scope; do not stage or alter it.
- No daemon/Rust changes, dependencies, root Makefile changes, provider/auth changes, parentID mapping, or remote Git operations.
- Keep diagnostics sanitized; never store raw payloads, secrets, full logs, or prompt output.
- Follow [[git-workflow-standards]](../../git-workflow-standards/SKILL.md) for safe Git operations and separate planning-only `docs(planning):` commits from implementation.
- If a test fails twice for the same reason, stop and report.

## Micro-planning
Create `planning/tasks/{{id}}.todo.md` with Goal, at most 12 TODO steps, Notes, and up to 5 numbered Questions. Task 39.1.1 must include the trace-file strategy, exact command/model, initial and updated `swayve status --watch --diff` observation, sanitized state report, and cleanup. Task 39.2.1 must cite the positive Wave 1 evidence and exact focused test. Stop for orchestrator review.

## Execution
After explicit approval, execute only the task spec. Task 39.1.1 must remove its temporary probe and output before reporting; its concise sanitized report in `planning/STATE_39.md` is the durable proof record. Task 39.2.1 may run only after the positive proof gate. Run the focused OpenCode test for permanent code changes; do not run `make all` unless scope expands. Commit implementation separately from planning artifacts; never commit temporary trace data.

## Reporting
Append a concise report under the task heading in `planning/STATE_39.md`: status, live proof evidence or failed boundary, files changed, commit IDs, exact commands/test results, deviations, and any blockers.
