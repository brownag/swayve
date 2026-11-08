# Execution Plan 30: lifecycle hook tests (G74)

Source: `planning/GOALS.md` Sequenced Goal G74. Depends on G80 (closed in Orchestration 29; its argv-capture pattern is reused).
Recon base: HEAD `4df69ca`. Status: approved by the user 2026-11-08; executed (closing gate green).

## Goal & Scope

**Goal.** Lifecycle hooks (`on_agent_spawn`, `on_agent_status_change`) have tests that check the env contract (`SWAYVE_AGENT_ID`, `SWAYVE_AGENT_STATUS`), the status string for every `AgentStatus` variant, and the order of status updates the hook sees. `execute_lifecycle_hook` (`src/daemon.rs`) gets a test seam.

**In scope.**
- Split `execute_lifecycle_hook` into a pure command builder and a spawn call, so the env and argv can be asserted without running a process.
- Unit tests for the builder and for `agent_status_to_str` (all six variants).
- An integration test: a real daemon, a status-change hook that appends `id status` per update, two `UpdateStatus` messages, and an exact assertion on the file contents in order.

**Out of scope.**
- Positional arguments to hooks. Hooks take no positional args today; the contract is env vars (ADR 0006). Changing it is a new decision.
- Reaping hook child processes. `execute_lifecycle_hook` drops the `Child` without `wait`, so exited hooks stay zombies until the daemon exits. Recorded as a new goal candidate, not fixed here.
- Suppressing the status hook when the status is unchanged. Current behavior (fires on every update) is pinned, not changed.
- Config format (`HooksConfig`, ADR 0010) and the spawn hook's hardcoded `"idle"`.

**Verbatim constraints.**
- G74 exit criterion: "`on_agent_spawn` and `on_agent_status_change` have a test that checks argument order and status string."
- G74 depends on G80: "reuses its argv-capture pattern."
- ADR 0006 (env contract) and ADR 0010 (serde defaults) stand.

## Decision Log

- **D1 (2026-11-08): "argument order" means the hook inputs, not positional args.** Hooks run as `sh -c <cmd>` with no positional args. The order checked is the order of `UpdateStatus` messages the hook observes, and the env pair per call. No new args (see Out of scope).
- **D2 (2026-11-08): test seam by builder, not by mock process.** `lifecycle_hook_command(hook, agent_id, status) -> std::process::Command` is pure. Tests read `get_args()` and `get_envs()`. The integration test runs a real shell.
- **D3 (2026-11-08): status string is the existing `agent_status_to_str`.** No rename. `Failed(_)` maps to `"failed"` (the reason is not passed).
- **D4 (2026-11-08): one wave, two disjoint tasks.** 30.1.1 edits `src/daemon.rs` only. 30.2.1 edits `tests/integration_tests.rs` only. Neither depends on the other. `make all` runs serially after the wave.
- **D5 (2026-11-08): the integration test copies `test_hooks_and_monitoring`.** Same socket/daemon harness, same poll loop, `unique_path` for every file.

## Hazard Notes (from recon)

- **H1.** `src/daemon.rs` is 1455 lines; `tests/integration_tests.rs` is 1400. Use grep anchors and offset/limit reads, never whole-file reads.
- **H2.** `execute_lifecycle_hook` at `src/daemon.rs:323`. Its two call sites: spawn hook at `src/daemon.rs:171` (status `"idle"`), status hook at `src/daemon.rs:292` in `update_status`. `agent_status_to_str` at `src/daemon.rs:312`.
- **H3.** The `UpdateStatus` IPC arm is at `src/daemon.rs:518`. It calls `state.update_status` and then sends a border-colour command to sway. The integration test does not need sway: `sway_cmd_tx` is a receiver the test drops.
- **H4.** `daemon.rs` unit test module starts at `src/daemon.rs:784`. Add the new tests at its bottom.
- **H5.** Test paths use `common::unique_path` (G45). No `/tmp` literal.
- **H6.** `make all` is serial. Never background it.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 30.1.1 | G74 | `src/daemon.rs` | — |
| 1 | 30.2.1 | G74 | `tests/integration_tests.rs` | — |

Wave gate: serial `make all` exit 0, zero warnings. Targeted checks after each task.

---

## Task 30.1.1: pure hook command builder and status-string tests (G74)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `src/daemon.rs`
- can create: none (scratch in `.development/` only)

**Depends on:** none.

**Spec.**
1. Add `fn lifecycle_hook_command(hook_cmd: &str, agent_id: &AgentId, status: &str) -> std::process::Command` next to `execute_lifecycle_hook` (`src/daemon.rs:323`). It builds `Command::new("sh").arg("-c").arg(hook_cmd).env("SWAYVE_AGENT_ID", &agent_id.0).env("SWAYVE_AGENT_STATUS", status)` and returns it.
2. Rewrite `execute_lifecycle_hook` to call the builder and then `.spawn()`, keeping the same `eprintln!` error message and the same behavior (no wait, the `Child` is dropped as today).
3. Unit tests in the `daemon.rs` test module (`src/daemon.rs:784`):
   (a) `test_lifecycle_hook_command_args_and_env`: `get_program()` is `sh`; `get_args()` yields `["-c", hook]` in that order; `get_envs()` contains `("SWAYVE_AGENT_ID", id)` and `("SWAYVE_AGENT_STATUS", status)`.
   (b) `test_agent_status_to_str_all_variants`: the six variants map to `idle`, `thinking`, `executing_tool`, `awaiting_human_input`, `completed`, `failed`.
4. Do not change `update_status`, the spawn hook call, or the IPC arm.

**Anchors.** `execute_lifecycle_hook` at `src/daemon.rs:323`. `agent_status_to_str` at `src/daemon.rs:312`. Call sites `src/daemon.rs:171` and `src/daemon.rs:292`. Test module at `src/daemon.rs:784`.

**Reference implementation.** `panel_terminal_args` in `src/cli.rs:534` (pure builder, tested by value).

**Constraints.** Keep the env names exactly as ADR 0006 states. Do not add positional args (D1). Do not add a wait or a reaper (Out of scope).

**Acceptance.**
- `cargo test --lib test_lifecycle_hook_command` passes (1 test).
- `cargo test --lib test_agent_status_to_str` passes (1 test).
- `make check` exits 0 with zero warnings.

---

## Task 30.2.1: status-change hook observes UpdateStatus order (G74)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `tests/integration_tests.rs`
- can create: none (scratch in `.development/` only)

**Depends on:** none.

**Spec.**
1. Write `test_lifecycle_status_hook_order` in `tests/integration_tests.rs`, modeled on `test_hooks_and_monitoring` (`tests/integration_tests.rs:871`).
2. Config: `config.hooks.on_agent_status_change = Some(format!("printf '%s %s\\n' \"$SWAYVE_AGENT_ID\" \"$SWAYVE_AGENT_STATUS\" >> {flag}"))`. `auto_monitor` off, `on_agent_spawn` None, `terminal.default_emulator` left as default (it is not run).
3. Start the daemon as in the hooks test. Register one agent (`agent-status-1`, `pi`, pid unique). Then send `UpdateStatus` with `Thinking`, then `ExecutingTool`, on the same connection, one JSON line each (`AgentIPCMessage::UpdateStatus`).
4. Poll the flag file up to 2 s until it has two lines. Assert exactly:
   `["agent-status-1 thinking", "agent-status-1 executing_tool"]`.
5. Clean up the socket, the flag file and the daemon tasks, as the hooks test does.

**Anchors.** Hooks test harness `tests/integration_tests.rs:871-~960`. `UpdateStatus` message type: `AgentIPCMessage::UpdateStatus` in `src/types.rs` (fields `agent_id`, `status`; check the exact shape with grep before writing). `start_daemon_ipc` and `run_daemon_loops` imports already at the top of the file.

**Reference implementation.** `test_hooks_and_monitoring` (`tests/integration_tests.rs:871`).

**Constraints.** Do not add `/tmp` literals (G45). Do not change `tests/common`. Do not change the existing hooks test's assertions. No running Sway: the test drops the `sway_cmd_tx` receiver's end, as the hooks test does.

**Acceptance.**
- `cargo test --test integration_tests test_lifecycle_status_hook_order` passes.
- `cargo test --test integration_tests` passes with 14 tests (13 existing plus this one).
- `make check` exits 0 with zero warnings.

---

## Closing gate

After wave 1, run serial `make all`. Confirm exit 0 and zero warnings. Then move G74 to Completed in `planning/GOALS.md` with its commit hashes. Record the new candidate (hook child reaping) in `GOALS.md` Independent Backlog.
