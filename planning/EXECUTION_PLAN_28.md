# Execution Plan 28: agent ownership, subscription lifetime, and launch argv

Source: `planning/GOALS.md` (G79 circled, G81 sequenced, G80 backlog), from `planning/ARCHITECTURE_REVIEW_2026-11-08_FOLLOWUP.md` (§1, §3, §2).
Recon base: `planning/STATE_27.md`, HEAD `17c72ce`. Makefile at repo root.
Status: approved 2026-11-08, ready for wave-orchestration.

## Goal & Scope

**Goal.** Agent identity and window ownership live in one coherent module (G79). Subscription delivery releases closed subscribers and shares one writer lifetime (G81). Terminal launches build argv once, with whitespace paths and quoted prompts preserved (G80).

**In scope.**
- G79: registration keeps an existing container binding; rebinding releases the old container; CLI and Pi derive the same `app_id`; binding and creation publish to subscribers.
- G81: an unchanged diff subscriber whose channel is closed is removed on the next broadcast; full and diff writers share one lifetime owner.
- G80: the monitor launch and the `status --watch` panel launch build argv through one helper; the monitor log path with a space stays one argument; the argv-capture test replaces the `echo "$@"` substring assertions.

**Out of scope.**
- A generic registry-backend seam (review §1: no second adapter exists).
- A new resync protocol or any wire-format change (ADR 0008, 0013).
- Re-opening G65 (monitor env parity, pruned).
- G73, G74, G75, G66, G78 (backlog and deferred items).
- Pi lint or type-check gaps (G73).
- Any change to the `subscribe` bare-array format.

**Verbatim constraints.**
- G79: "Registration reconciliation, current window ownership, coherent PID and container indexes, consistent change publication" (review §1, After diagram).
- G79 exit criteria: "tests through the registry interface pass for (a) registration → binding → re-registration, (b) rebinding → old-window close, and (c) binding → subscriber observation. Each asserts that the registry and the status stream agree. No caller coordinates the indexes directly. `make all` is green with zero warnings."
- G81 exit criteria: "a unit test shows a diff subscriber whose channel closed while its state is unchanged is removed on the next broadcast. Repeated-subscription and writer-cleanup tests pass. The existing overflow and slow-consumer tests still pass. Both wire formats are unchanged. `subscribe_golden` passes."
- G80 exit criteria: "an argv-capture test covers all three launches [agent, monitor, panel], including a whitespace path, a quoted prompt and an explicit shell template. The current `echo \"$@\"` substring assertions are replaced. `make all` is green."
- Constraint (G79): "preserve ADR 0003, 0005, 0006 and 0009. Do not add a generic registry-backend seam."
- Constraint (G81): "preserve ADR 0008 and 0013 (full-list compatibility, per-subscriber baselines, snapshot recovery). No new resync protocol."
- Constraint (G80): "preserve ADR 0006 and 0010. Do not reopen G65. Keep the existing spawn-plan extraction (G60, G64)."

## Decision Log

- **D1 (2026-11-08): serial waves, one task per wave.** The user chose serial waves over parallel worktrees. The three goals share `src/daemon.rs` and `src/cli.rs`, so every wave holds one task.
- **D2 (2026-11-08): G81's unchanged-state disconnect fix is a separate task that does not depend on G79.** Its failing test only needs a closed receiver and no state change. Only the publication-consistency part waits for G79. The G81 writer-lifetime task runs after the disconnect fix.
- **D3 (2026-11-08): G79 is split into four tasks by ownership rule.** Re-registration, rebinding, `app_id` alignment, and publication each have their own failing test. Each is small enough for a worker and touches a known set of lines.
- **D4 (2026-11-08): G79's `app_id` alignment touches the Pi client.** The review (§1 Evidence) names `integrations/pi/src/swayve-client.ts:61–71` re-registering with a differently derived `app_id`. The fix is to make the Pi client and the CLI derive the same `app_id` from the agent id. The daemon does not change its binding rule.
- **D5 (2026-11-08): the review's "destruction trusts the association" finding is resolved by the rebinding task.** The fix releases the old container when a window is rebound. Destruction then removes only the agent that still owns that window.
- **D6 (2026-11-08): G80 keeps the existing `terminal.args` template format.** No config-format change (ADR 0010). The shared helper expands `{app_id}`, `{title}` and `{command}` for all three launches.

## Hazard Notes (from recon)

- **H1.** `src/daemon.rs` is 1365 lines. Workers must use grep anchors, never a whole-file read. Test module starts near line 785.
- **H2.** `register_agent` (`src/daemon.rs:140`) replaces the record with `container_id: None` and `status: Idle`. It also writes `pid_to_agent` but does not clear `con_to_agent`, so a stale reverse map can survive a re-registration.
- **H3.** `on_window_destroyed` (`src/daemon.rs:244`) removes the agent if its con_id maps to it, whether or not the agent was rebound since.
- **H4.** `broadcast_diff` (`src/daemon.rs:88`) returns early at the `if !snapshot && upsert.is_empty() && remove.is_empty()` check, before `try_send`. A closed receiver is found only on the next changed broadcast (review §3).
- **H5.** `spawn_monitor_terminal` (`src/daemon.rs:257`) interpolates `logs_dir` into a string and shlex-splits it. A runtime dir with a space splits into two arguments.
- **H6.** The `echo "$@"` mock terminal is at `tests/integration_tests.rs:898–903`. Its assertions at lines 985–1000 use substrings.
- **H7.** `app_id` derivation: CLI at `src/cli.rs:545–547` (dotted), Pi at `integrations/pi/src/swayve-client.ts:61–71` (`agent.${agentId}`). They must match.
- **H8.** `src/daemon.rs` tests run as unit tests in `#[cfg(test)] mod tests` at the bottom of the file (AGENTS.md).
- **H9.** `make all` is serial. The gate must run serially, never backgrounded.
- **H10.** Test sockets and flags use `tests/common::unique_path` (G45). Do not add `/tmp` literals.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 28.1.1 | G79 | `src/daemon.rs` | — |
| 2 | 28.1.2 | G79 | `src/daemon.rs` | 28.1.1 |
| 3 | 28.1.3 | G79 | `src/cli.rs`, `integrations/pi/src/swayve-client.ts`, `integrations/pi/tests/swayve-client.test.ts` | 28.1.2 |
| 4 | 28.1.4 | G79 | `src/daemon.rs`, `tests/integration_tests.rs` | 28.1.3 |
| 5 | 28.2.1 | G81 | `src/daemon.rs` | — (independent of G79; runs after G79 by D1) |
| 6 | 28.2.2 | G81 | `src/daemon.rs`, `src/cli.rs` | 28.2.1 |
| 7 | 28.3.1 | G80 | `src/daemon.rs`, `src/cli.rs` (shared argv helper) | 28.2.2 |
| 8 | 28.3.2 | G80 | `tests/integration_tests.rs` | 28.3.1 |
| 9 | 28.3.3 | G80 | `src/cli.rs` (status --watch panel argv) | 28.3.2 |

Each wave holds one task. Within a wave the file scope is never shared across tasks, because each wave has one task.

## Wave Gate (every wave)

- The task's targeted tests pass (named in its spec).
- Serial `make all` exits 0 with zero warnings after waves 1, 4, 6 and 9 (the gates in STATE_28.md).
- `planning/GOALS.md`: a goal moves to Completed with its commit hash only when all its tasks are done.

---

## Task 28.1.1: re-registration keeps an existing container binding (G79)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `src/daemon.rs`
- can create: `.development/` scratch only

**Depends on:** none.

**Spec.**
1. Write a failing unit test in the `src/daemon.rs` test module. Register an agent, bind it with `register_agent_window`, re-register the same agent id, and assert `container_id` still equals the bound con_id and `con_to_agent` still maps that con_id to the agent.
2. In `register_agent` (`src/daemon.rs:140`), when an agent with the same id already exists and has a `container_id`, keep `container_id`, keep the `con_to_agent` entry, and keep the current `status`. Do not reset them. The new `project`, `parent_id`, `app_id` and `pid` still apply.
3. Keep the `pid_to_agent` update. Do not change `auto_monitor_wanted` or the lifecycle hook call.

**Anchors.** `register_agent` at `src/daemon.rs:140`; the reset block at lines 152–163. `register_agent_window` at `src/daemon.rs:642`. `on_window_created` at `src/daemon.rs:182`. Tests live at the bottom of `src/daemon.rs` (after line ~785), beside `test_window_destroyed_promotes_children` (~1158).

**Reference implementation.** `test_window_destroyed_promotes_children` (`src/daemon.rs:1158`) for building a `DaemonState` and registering agents.

**Constraints.** Do not touch rebinding or destruction (28.1.2). Do not touch the Pi client (28.1.3). No new public API. Preserve ADR 0006 identity (app_id) semantics.

**Acceptance.**
- `cargo test --lib test_reregister` passes (new test named `test_reregister_keeps_container_binding`).
- `cargo test --lib test_register_agent` passes.
- `make check` exits 0 with zero warnings.

---

## Task 28.1.2: rebinding releases the old container; old-window close keeps the rebound agent (G79)

**Repo:** swayve. **Wave:** 2.

**Files in scope.**
- must edit: `src/daemon.rs`
- can create: `.development/` scratch only

**Depends on:** 28.1.1 (the re-registration path is now fixed).

**Spec.**
1. Write a failing test: bind agent A to con 10, bind A again to con 20 (same app_id), then destroy con 10. Assert agent A still exists and `container_id == Some(20)`. Assert `con_to_agent` has no entry for 10.
2. In `register_agent_window` (`src/daemon.rs:642`), when the agent already has a different `container_id`, remove the old `con_to_agent` entry before inserting the new one.
3. In `on_window_destroyed` (`src/daemon.rs:244`), only remove the agent when its current `container_id` equals the destroyed con_id. Otherwise keep the agent and only drop the stale mapping.
4. Keep `promote_orphans` and the `LayoutAction::WindowDestroyed` return unchanged in the non-stale case.

**Anchors.** `register_agent_window` at `src/daemon.rs:642`. `on_window_destroyed` at `src/daemon.rs:244`, body lines 244–256. `promote_orphans` at `src/daemon.rs:631`. The test module as in 28.1.1.

**Reference implementation.** `test_window_destroyed_promotes_children` (`src/daemon.rs:1158`).

**Constraints.** Do not change the registration path (done in 28.1.1). Do not change publication (28.1.4). No new public API.

**Acceptance.**
- `cargo test --lib test_rebind` passes (new test named `test_rebind_releases_old_container`).
- `cargo test --lib test_window_destroyed` passes (existing `test_window_destroyed_promotes_children` still passes).
- `make check` exits 0 with zero warnings.

---

## Task 28.1.3: CLI and Pi derive the same `app_id` (G79)

**Repo:** swayve. **Wave:** 3.

**Files in scope.**
- must edit: `src/cli.rs`, `integrations/pi/src/swayve-client.ts`, `integrations/pi/tests/swayve-client.test.ts`
- can create: `.development/` scratch only

**Depends on:** 28.1.2.

**Spec.**
1. Read the CLI `app_id` derivation at `src/cli.rs:545–547` and the Pi derivation at `integrations/pi/src/swayve-client.ts:61–71`. Record both formats in the micro-plan.
2. Write a failing Pi test asserting that, for agent id `X`, the Pi client's registered `app_id` equals the CLI's derived `app_id`. Derive the expected value from the CLI format; do not hard-code a guess.
3. Make the Pi client use the same format as the CLI. Change the minimum: one derivation function in TypeScript, matching the CLI format exactly.
4. Add a Rust test that pins the CLI format for a sample id, so the two cannot drift silently.

**Anchors.** `src/cli.rs:545–547` (CLI derivation). `integrations/pi/src/swayve-client.ts:61–71` (`register()` `app_id: \`agent.${this.agentId}\``). Pi tests live in `integrations/pi/tests/swayve-client.test.ts`. Pi has its own `package.json` and runs with `tsx --test`.

**Reference implementation.** The existing Pi `register` tests in `integrations/pi/tests/swayve-client.test.ts`.

**Constraints.** Do not change the daemon's binding rule. Do not change the wire format of `agent.register`. Do not touch `Makefile`. Do not run the full `npm run build` target.

**Acceptance.**
- `cd integrations/pi && npm test` exits 0 (the new test passes; prior count 6/6 plus one).
- `cargo test --lib` for the new Rust format pin passes.
- `make check` exits 0 with zero warnings.

---

## Task 28.1.4: binding and creation publish to subscribers (G79)

**Repo:** swayve. **Wave:** 4.

**Files in scope.**
- must edit: `src/daemon.rs`, `tests/integration_tests.rs`
- can create: `.development/` scratch only

**Depends on:** 28.1.3.

**Spec.**
1. Read how `run_daemon_loops` (`src/daemon.rs:511`) publishes status today. The review says destruction publishes, and creation or binding does not (`src/daemon.rs:574–589`).
2. Write a failing integration test: subscribe, register an agent, bind a window, and assert the subscriber's next status line shows the agent with its `container_id`. Use the existing socket test helpers in `tests/integration_tests.rs`.
3. In `run_daemon_loops`, call `broadcast_status` (and `broadcast_diff`, as the existing code does) after a window bind, so the subscriber sees the bound container.
4. Do not add a new event type or wire field.

**Anchors.** `run_daemon_loops` at `src/daemon.rs:511`; the destruction publication at `src/daemon.rs:574–589`. `broadcast_status` at `src/daemon.rs:56`. `broadcast_diff` at `src/daemon.rs:88`. Integration test helpers: `wait_for_socket` and the subscription patterns in `tests/integration_tests.rs` (grep `subscribe`).

**Reference implementation.** `test_daemon_broadcast_status` (`src/daemon.rs:1186`) for the in-process form; `tests/integration_tests.rs` for the socket form.

**Constraints.** No new wire format (ADR 0008, 0013). Keep `subscribe` as a bare array. Keep `DiffFrame` (G70).

**Acceptance.**
- `cargo test --test integration_tests subscriber_sees_bound_container` passes (new test name in `tests/integration_tests.rs`).
- `cargo test --test subscribe_golden` passes unchanged.
- `make check` exits 0 with zero warnings.

---

## Task 28.2.1: an unchanged diff subscriber with a closed channel is removed (G81)

**Repo:** swayve. **Wave:** 5.

**Files in scope.**
- must edit: `src/daemon.rs`
- can create: `.development/` scratch only

**Depends on:** 28.1.4 (run after G79 by D1; this task has no code dependency on G79).

**Spec.**
1. Write a failing unit test in the `src/daemon.rs` test module. Push a diff subscriber whose receiver is dropped. Call `broadcast_status` once with no state change. Assert the subscriber is removed from `subscribers`.
2. In `broadcast_diff` (`src/daemon.rs:88`), move the closed-channel check before the unchanged-state early return. The simplest correct form: when `!snapshot && upsert.is_empty() && remove.is_empty()`, still call `try_send` only to detect `Closed`, or check `tx.is_closed()` before returning.
3. Keep `try_send` behaviour for the changed case unchanged. Keep the `Full` branch that sets `needs_snapshot`.

**Anchors.** `broadcast_diff` at `src/daemon.rs:88`; early return at lines 107–108; `try_send` match at lines 122–134. `push_diff_subscriber` at `src/daemon.rs:49`. Existing diff tests: `test_diff_subscriber_gets_snapshot_then_diffs` (`src/daemon.rs:1250`) and `test_diff_subscriber_recovers_with_snapshot_after_drop` (`src/daemon.rs:1270`).

**Reference implementation.** `test_diff_subscriber_recovers_with_snapshot_after_drop`.

**Constraints.** Do not change the `Full` or `Closed` return values. Do not change overflow policy (ADR 0008). Do not change `subscribe_golden`.

**Acceptance.**
- `cargo test --lib test_diff_subscriber_removed_when_unchanged` passes (new test).
- `cargo test --lib test_diff_subscriber` passes (all three diff tests).
- `cargo test --test slow_consumer` passes unchanged.
- `make check` exits 0 with zero warnings.

---

## Task 28.2.2: full and diff writers share one lifetime owner (G81)

**Repo:** swayve. **Wave:** 6.

**Files in scope.**
- must edit: `src/daemon.rs`, `src/cli.rs`
- can create: `.development/` scratch only

**Depends on:** 28.2.1.

**Spec.**
1. Read the detached writer tasks at `src/daemon.rs:463–497` (the full and diff writer loops in `handle_client`) and `src/cli.rs:593–614` (the fresh-subscription wait in `register_and_wait`). Record in the micro-plan which code duplicates a writer loop.
2. Write a failing test for writer cleanup: a subscription whose socket closes is removed from the subscriber list and its writer task ends, for both modes.
3. Extract one writer function that owns the receiver and the socket write for both modes. The mode only selects which channel and frame format it uses. Keep `handle_client` as the caller.
4. Do not change the wire format or the `subscribe` / `subscribe_diff` method names.

**Anchors.** `handle_client` at `src/daemon.rs:426`; writer loops at `src/daemon.rs:463–497`. `register_and_wait` at `src/cli.rs:577`; fresh subscription at `src/cli.rs:593–614`. Existing golden: `tests/subscribe_golden.rs`. Existing overflow: `tests/slow_consumer.rs`.

**Reference implementation.** The current full writer in `handle_client` (`src/daemon.rs:463`), as the shape to keep.

**Constraints.** Keep the `daemon.subscribe` bare-array format and the `subscribe_diff` frame format. No new message kind. Preserve the two overflow policies.

**Acceptance.**
- `cargo test --test subscribe_golden` passes unchanged.
- `cargo test --test slow_consumer` passes unchanged.
- `cargo test --lib test_writer` passes (new writer-cleanup test).
- `make check` exits 0 with zero warnings.

---

## Task 28.3.1: the monitor launch builds argv through one helper; a whitespace log path stays one argument (G80)

**Repo:** swayve. **Wave:** 7.

**Files in scope.**
- must edit: `src/daemon.rs`, `src/cli.rs`
- can create: `.development/` scratch only

**Depends on:** 28.2.2.

**Spec.**
1. Write a failing unit test: build the monitor argv for agent `a1` with a `logs_dir` containing a space, and assert the path is one argument, not two. Run the test against the pure helper, not a spawned process.
2. Extract a pure helper that takes the template args, app_id, title, and a command given as a list of argv (not a string). It expands `{app_id}` and `{title}` and replaces `{command}` by splicing the argv in. No shlex split of paths.
3. Use the helper in `spawn_monitor_terminal` (`src/daemon.rs:257`). Build the command argv as `["tail", "-f", <path>]` directly.
4. Do not change the `terminal.args` config format (ADR 0010).

**Anchors.** `spawn_monitor_terminal` at `src/daemon.rs:257`; the log path at line 281 (`tail -f {}/{}.log`); the shlex split at lines 273–279. `TerminalConfig` in `src/config.rs:111–120`. The panel launch in `src/cli.rs:350–361` is task 28.3.3, not this task.

**Reference implementation.** `spawn_plan` (`src/cli.rs:531`) as the pure-function shape for argv.

**Constraints.** Do not reopen G65 (no env change for monitor). Keep `spawn_plan` unchanged (G60, G64). Do not change `execute_lifecycle_hook`.

**Acceptance.**
- `cargo test --lib test_monitor_argv` passes (new test with a spaced path).
- `cargo test --lib test_spawn_plan` passes (7 existing tests unchanged).
- `make check` exits 0 with zero warnings.

---

## Task 28.3.2: the argv-capture test replaces `echo "$@"` substring assertions (G80)

**Repo:** swayve. **Wave:** 8.

**Files in scope.**
- must edit: `tests/integration_tests.rs`
- can create: `.development/` scratch only

**Depends on:** 28.3.1.

**Spec.**
1. Read the mock terminal at `tests/integration_tests.rs:898–903` (`echo "$@" > flag`) and its assertions at `985–1000`.
2. Replace the mock with an argv-capture adapter: a script that writes each argument on its own line to a file under `unique_path`. Keep the script simple (`printf '%s\n' "$@"`).
3. Assert exact argv arrays for the monitor launch: the whitespace path is one entry, `-f` is one entry, and `tail` is the command.
4. Keep the existing subscribe and spawn-hook flags as they are.

**Anchors.** Mock writer at `tests/integration_tests.rs:898–903`. Assertions at `985–1000`. The hook at line 914. Test helper `common::unique_path`.

**Reference implementation.** The existing spawn-hook flag pattern at `tests/integration_tests.rs:914`.

**Constraints.** Do not change the assertion for the spawn hook. Do not add `/tmp` literals (G45). Do not change `tests/common` beyond what the test needs.

**Acceptance.**
- `cargo test --test integration_tests monitor` passes.
- `cargo test --test integration_tests` passes overall (count matches the prior integration count, 11, plus any new tests).
- `make check` exits 0 with zero warnings.

---

## Task 28.3.3: the `status --watch` panel launch passes its command as argv, not one string (G80)

**Repo:** swayve. **Wave:** 9.

**Files in scope.**
- must edit: `src/cli.rs`
- can create: `.development/` scratch only

**Depends on:** 28.3.2.

**Spec.**
1. Read the panel launch in `src/cli.rs:350–361`. It passes `swayve status --watch` as one argument. Record the current argv in the micro-plan.
2. Build the panel command with the shared argv helper from 28.3.1, so `status --watch` becomes separate argv entries (`swayve`, `status`, `--watch`) under the `{command}` placeholder.
3. Write a failing unit test that asserts the panel argv entries. Use the pure helper, not a spawned process.

**Anchors.** Panel launch at `src/cli.rs:350–361`. Shared helper from 28.3.1 (in `src/daemon.rs` or `src/cli.rs`, whichever 28.3.1 placed it).

**Reference implementation.** `spawn_plan` (`src/cli.rs:531`).

**Constraints.** Do not change `status --watch` behaviour. Do not change the `--diff` flag handling (G33). Do not change the config format.

**Acceptance.**
- `cargo test --lib test_panel_argv` passes (new test).
- `cargo test --lib test_status_watch_cmd_parsing` passes unchanged.
- `make check` exits 0 with zero warnings.

---

## Closing gate

After wave 9, run `make all` serially. Confirm exit 0 and zero warnings. Then:
- move G79, G81 and G80 to Completed in `planning/GOALS.md` with their commit hashes;
- promote the next Independent Backlog item only if the user asks.
