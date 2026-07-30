# Wave 3 Worker Prompt Template

You are a specialized coding agent executing a task from a pre-approved `EXECUTION_PLAN.md` for the **swayve** project.

## Context

- **Wave**: Wave 3 — Architectural Polish & Agent Coordination
- **Goal**: Break daemon ↔ layout circular dependency; add process-tree config; verify parent agent support
- **Project**: Rust daemon + CLI for Sway window manager coordination with AI agents
- **Test Command**: `cargo test` (full suite; per-task gates: `cargo check` for 3.1/3.2, `cargo test` for 3.3)
- **Build**: `cargo build --release` (binary) or `cargo check` (fast)

## Operating Instructions

1. **Understand**: Read the files in `files-in-scope` and locate the `grep-anchors` for this task.
2. **Implement**: Perform the work described in the `spec` prose.
   - Be surgical: edit only what's specified.
   - Follow project conventions (Rust idioms, module structure, error handling).
   - Reference existing patterns (e.g., how config.rs parses TOML, how daemon.rs uses Config).
3. **Verify**:
   - Run the `acceptance` command (`cargo check` or `cargo test` as specified).
   - If task has a `targeted-test`, run it explicitly (e.g., `cargo test --lib layout`).
   - Ensure all prior waves' tests still pass.
4. **Report**:
   - If successful: "Task completed successfully. [Brief summary of changes]"
   - If failed: "Task failed: [error message]. [Brief explanation]"

## Hazard Notes & Constraints

- **Circular import during Task 3.1**: You'll update both `daemon.rs` (impl trait) and `layout.rs` (use trait) in one commit. Ensure both changes land atomically.
- **Config defaults**: `max_ancestry_depth` default in code must match any hardcoded constant. If changed, update both.
- **Focus command scope** (Task 3.3): Wire CLI into daemon, but don't add hierarchical focus logic yet — that's Wave 4.
- **No behavior changes**: Tasks 3.1 and 3.2 are pure refactors. Task 3.3 is test + integration, no new user-facing features.

## Dependencies & Ordering

- Task 3.1 → Task 3.2 → Task 3.3 (strict sequence)
- All three tasks must complete before wave is considered done
- Each task's acceptance criteria must pass before proceeding to the next

## Project Conventions

- **Modules**: src/cli.rs (CLI parsing), src/daemon.rs (socket + event loop), src/layout.rs (orchestration), src/types.rs (shared types), src/config.rs (config loading)
- **Testing**: Unit tests inline with `#[cfg(test)]` modules; integration tests in tests/integration_tests.rs
- **Error handling**: Use `anyhow::Result<T>` and `?` operator; propagate errors rather than panicking
- **Commits**: Conventional format (feat, fix, refactor, test). One commit per task.

## Reference Implementations

- **Trait usage**: See `crate::types::AgentStatus` enum pattern (Serialize/Deserialize derive)
- **Config loading**: See `src/config.rs::load_config()` — TOML parsing with fallback defaults
- **Daemon event loop**: See `src/daemon.rs::run_daemon_loops()` — match on event types, dispatch to handlers
- **LayoutManager**: See `src/layout.rs::LayoutManager::compute_commands()` — pure function, message-passing pattern
- **Integration tests**: See `tests/integration_tests.rs::test_subagent_layout_routing()` — setup daemon, send events, assert on commands

## Success Criteria

- ✅ Task 3.1: `cargo check` passes; `cargo test --lib layout` passes; import cycle broken (no daemon.rs imports in layout.rs)
- ✅ Task 3.2: `cargo check` passes; config.rs parses `[daemon]` section; daemon.rs reads from config
- ✅ Task 3.3: `cargo test` (full suite) passes; new test `test_cli_parent_flag_to_layout` passes; focus command flows through daemon
- ✅ Overall: No regressions in Wave 1 or Wave 2 tests; working tree clean; all commits follow conventional format

## Execution

Begin the task now. Start with Task 3.1 if this is the first run, or identify which task to run based on the wave state.
