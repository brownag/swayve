# Execution Plan 31: reap daemon-spawned children (G82)

Source: `planning/GOALS.md` Circled Goal G82. Found by the magic-wand audit (2026-11-08); the hook half was first noted in Orchestration 30.
Recon base: HEAD `2a4dd0a`. Status: approved by the user 2026-11-08; executed (closing gate green).

## Goal & Scope

**Goal.** The daemon reaps every child process it launches. A child that exits is not left as a zombie for the life of the daemon. Covers lifecycle hook commands and monitor terminals.

**In scope.**
- One helper in `src/daemon.rs` that spawns a `std::process::Command` and hands the `Child` to a detached thread that calls `wait()`.
- Both daemon launch sites route through it: `execute_lifecycle_hook` and `spawn_monitor_terminal`.
- A Linux test that proves a reaped child is gone from `/proc`.

**Out of scope.**
- The `swayve spawn` CLI arm (`src/cli.rs`, ~296-310). It is a short-lived process; when it exits, init reaps its children. Not a leak.
- A shared reaper task, tokio `process` changes, or `SIGCHLD` handling. A thread per live child is enough at this scale (see the ponytail note in the code).
- Any change to hook commands, env, or argv (ADR 0006).

**Verbatim constraints.**
- G82 exit criteria (as written in GOALS.md): "a test runs a daemon-spawned hook (`exit 0`) and a monitor launch, waits for them, and asserts neither is left as a zombie child of the test process. Serial `make all` exit 0, zero warnings."
- Constraint from G74: `lifecycle_hook_command` stays the pure builder; its tests stay as they are.

## Decision Log

- **D1 (2026-11-08): reap with a detached thread, not an async task.** `spawn_reaped` calls `Command::spawn`, then `std::thread::spawn(move || child.wait())`. No new dependency. The daemon call sites are sync, so no runtime is needed. Ceiling: one thread per live child; upgrade path is a shared reaper if hook fan-out ever becomes large.
- **D2 (2026-11-08): the helper returns the pid.** Tests need the pid to check `/proc/<pid>`. Callers that do not need it ignore it.
- **D3 (2026-11-08): the monitor path is covered by routing, not by a process-level test.** `spawn_monitor_terminal` returns nothing and launches a real emulator. Testing it at process level would need a fake emulator and a pid it does not expose. The helper test covers the mechanism; a grep acceptance check covers that both sites call it. This narrows the G82 exit criterion for the monitor launch; the judgment call is flagged for approval.
- **D4 (2026-11-08): one task, one file.** The change is one helper, two call sites, and one test, all in `src/daemon.rs`. A single task avoids a same-file collision.

## Hazard Notes (from recon)

- **H1.** `src/daemon.rs` is 1479 lines. Use grep anchors and offset/limit reads, never whole-file reads.
- **H2.** `execute_lifecycle_hook` at `src/daemon.rs:333`, calling `lifecycle_hook_command` (`src/daemon.rs:324`). Call sites: spawn hook `src/daemon.rs:171`, status hook `src/daemon.rs:292`.
- **H3.** `spawn_monitor_terminal` at `src/daemon.rs:266`; its `Command::new(&self.config.terminal.default_emulator)...spawn()` is at `~276`.
- **H4.** Linux only. The test reads `/proc`. swayve runs only under Sway (Linux), so this is acceptable; no `#[cfg]` gate is needed beyond the test itself.
- **H5.** Test module is at the bottom of `src/daemon.rs` (`mod tests`, `src/daemon.rs:788`). Add the test there.
- **H6.** `make all` is serial. Never background it.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 31.1.1 | G82 | `src/daemon.rs` | — |

Wave gate: serial `make all` exit 0, zero warnings.

---

## Task 31.1.1: reap daemon-spawned children through one helper (G82)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `src/daemon.rs`
- can create: none (scratch in `.development/` only)

**Depends on:** none.

**Spec.**
1. Write the failing test first, in the `daemon.rs` test module: `test_spawned_child_is_reaped`. It calls `spawn_reaped(std::process::Command::new("sh").args(["-c", "exit 0"]))`, takes the returned pid, and polls up to 2 s until `/proc/<pid>` no longer exists. Before the fix, the test does not compile (no helper), so record that red state, then add the helper and rerun. Red for the right reason: after the helper exists but before the wait, `/proc/<pid>` stays as a zombie and the test fails.
2. Add `fn spawn_reaped(cmd: &mut std::process::Command) -> std::io::Result<u32>` next to `lifecycle_hook_command` (`src/daemon.rs:~324`). It spawns the child, takes its pid, and moves the `Child` into `std::thread::spawn(move || { let _ = child.wait(); })`. Add the ponytail comment: thread per live child; shared reaper if fan-out grows.
3. `execute_lifecycle_hook` (`src/daemon.rs:333`): replace `lifecycle_hook_command(...).spawn()` with `spawn_reaped(&mut lifecycle_hook_command(...))`. Keep the `eprintln!` error path unchanged.
4. `spawn_monitor_terminal` (`src/daemon.rs:~276`): replace its `Command::new(...)...spawn()` with `spawn_reaped(...)`. Keep the `eprintln!` error path unchanged. Do not change `monitor_argv` or the logs-dir logic.
5. Do not change `lifecycle_hook_command`, its tests, `agent_status_to_str`, or the CLI spawn arm.

**Anchors.** `lifecycle_hook_command` at `src/daemon.rs:324`. `execute_lifecycle_hook` at `src/daemon.rs:333`. `spawn_monitor_terminal` at `src/daemon.rs:266`, spawn at `~276`. Test module at `src/daemon.rs:788`; the G74 tests `test_lifecycle_hook_command_args_and_env` and `test_agent_status_to_str_all_variants` sit there.

**Reference implementation.** `lifecycle_hook_command` (`src/daemon.rs:324`): pure builder, tested by value. The new helper is the spawning counterpart.

**Constraints.** Linux `/proc` only in the test. Do not add a runtime, a dependency, or a `SIGCHLD` handler (D1). Keep the env and argv contract (ADR 0006) unchanged.

**Acceptance.**
- `cargo test --lib test_spawned_child_is_reaped` passes.
- `grep -n "spawn_reaped" src/daemon.rs` shows the helper definition and exactly two call sites (`execute_lifecycle_hook` and `spawn_monitor_terminal`).
- `cargo test --lib lifecycle` still passes (G74 tests).
- `make check` exits 0 with zero warnings.

---

## Closing gate

After wave 1, run serial `make all`. Confirm exit 0 and zero warnings. Then move G82 to Completed in `planning/GOALS.md` with its commit hash. Promote the next open goal: G83 (advisory) is the only backlog item with a first action that touches the build; G84 stays deferred until its trigger.
