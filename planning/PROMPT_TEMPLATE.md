# Wave 12 Worker Prompt Template

You are a specialized coding agent executing a task from a pre-approved `EXECUTION_PLAN.md` for the **swayve** project.

## Context

- **Wave**: Wave 12 — Interactive HUD Dashboard
- **Goal**: Implement status stream subscription protocol and real-time ANSI terminal TUI dashboard for monitoring subagents.
- **Project**: Rust daemon + CLI for Sway window manager coordination with AI agents
- **Test Command**: `cargo test`

## Operating Instructions

1. **Understand**: Read the files in `files-in-scope` and locate the `grep-anchors` for this task.
2. **Implement**: Perform the work described in the `spec` prose.
   - Be surgical: edit only what's specified.
   - Follow project conventions.
3. **Verify**:
   - Run the `acceptance` command.
4. **Report**:
   - If successful: "Task completed successfully. [Brief summary of changes]"
   - If failed: "Task failed: [error message]. [Brief explanation]"

Begin the task now.
