# Execution Plan 23: Open Backlog Bundle (G46, G47, G49, G50, G51, G54, G55, G56, G57)

**Status: drafted 2026-11-08, awaiting approval. Not executed.**

## Goal & Scope

**Goal**: close the open goals in `planning/GOALS.md` that are internal to this repo and ready to plan. Bundled as one plan so the waves can be ordered by file overlap.

**In scope**:
- **G46** socket-level rate-limit test (circled).
- **G47** README Socket Contract documents the rate limit.
- **G49** focus-ordering sleeps (`tests/integration_tests.rs` 633, 638) replaced by an internal barrier event.
- **G50** hook-test sleep (`tests/integration_tests.rs` 948) replaced by polling the flag files.
- **G51** CI workflow file that runs `make all` on push (file only, not pushed).
- **G54** window lifecycle rules owned by `DaemonState`.
- **G55** process-tree depth-limit test that can fail.
- **G56** shared `DiffFrame` type, with a `subscribe_diff` golden test first.
- **G57** `spawn` command plan extracted as a pure function.

**Out of scope**:
- **G52** daemon delivery ack (Deferred, trigger not met).
- **G53** external HUD, **G25 / G15** Antigravity sidecar (external; prompts in `planning/external/`).
- **G58** focus-cycle struct (Deferred, trigger not met).
- Any change to the `daemon.subscribe` wire format. `tests/subscribe_golden.rs` must pass unchanged.
- Any change to the `daemon.subscribe_diff` wire bytes. G56 pins them first.
- Any new IPC message, ack, or reply.
- Pushing to any remote. Commits stay local.

**Verbatim constraints**:
- "that is fine, lets bundle a series of logical goals from goals.md/magic-wand into one plan" (user, Orchestration 23 request).
- "`make all` passes." Zero compiler warnings.
- "`daemon.subscribe` output is byte-identical to before." (G9, G29)

## Decision Log

- **D1: Scope** (2026-11-08, Magic Wand, user). The nine goals above. G52, G53, G58 excluded.
- **D2: No wire ack** (carried from Orchestration 22 D2). G49 uses an internal `DaemonEvent::Sync` with a oneshot reply. It is not sent over the socket. The enum is internal (`src/daemon.rs`).
- **D3: G54 keeps both gates** (default, not asked; **needs your approval**). The register path gates on "agent is a subagent" (`is_subagent`). The window path gates on "process-tree audit found a parent". Both are the same predicate, `subagent_evidence && config.daemon.auto_monitor`, with different evidence. G54 makes this one function and passes the evidence in. Behavior does not change.
- **D4: Execution mode** (default). Inline by the orchestrator, one commit per task, as Orchestration 22 did. No worker agents, no worktrees. Say so if you want workers.
- **D5: G51 writes the workflow file only** (default). It is never pushed. A push is needed for it to run, and that needs your go-ahead.
- **D6: G56 golden is taken from the current daemon before any change** (default). Golden first, then the refactor, so the bytes are pinned against the old code.
- **D7: Wave gates run serially** (carried from Plan 19 D6). Full `make all` at each wave end.
- **D8: Worktree base check** (carried). Not applicable with D4.

## Hazard Notes

- **H1 (G46 and the burst)**: the test sends 250 lines on one connection. The bucket starts with 200 tokens and refills at 100/s while the flood is sent, so the number delivered is 200 to about 205, not exactly 200. The test asserts that range. It then polls, not sleeps, for a token to return: it sends `daemon.focus` every 20 ms until one is delivered (within 2 s). That shows the client stayed connected and the limit recovered.
- **H2 (G47 and the config docs)**: the README already documents the keys (G48). G47 only adds the socket-contract bullets.
- **H3 (G49 and the event enum)**: `DaemonEvent` is `pub`. Adding `Sync` is internal, but the match in `run_daemon_loops` must reply to it. A missed arm is a silent hang, so the match is exhaustive.
- **H4 (G50 and the flag files)**: the hook test writes flag files from spawned processes. A poll needs a timeout. Keep the existing 2 s cap and 10 ms interval.
- **H5 (G54 and ADR 0005)**: `LayoutAction` shape stays. The new methods return `Vec<LayoutAction>` and `Vec<MonitorRequest>`. No layout change.
- **H6 (G54 and the orphan path)**: `promote_orphans` runs inside `register_agent_window`. Moving it into `DaemonState` must keep the order: promote orphans, then auditing, then the monitor decision.
- **H7 (G55 and the walk)**: `find_registered_parent_ancestor` takes `parent_of: impl Fn(u32) -> Option<u32>`. The production closure reads `/proc/<pid>/stat` through `get_parent_pid`. Behavior is unchanged.
- **H8 (G56 golden and HashMap order)**: `subscribe_diff` output for more than one agent depends on `HashMap` order. The golden test uses one agent per snapshot, or accepts both orders as G29 does.
- **H9 (G57 and the spawn arm)**: the arm was only partly read in Magic Wand. Task 23.1.3 reads the whole arm before planning.
- **H10 (G51 and the runner)**: the workflow runs `make all`, which needs `cargo` and `node`. It does not need Sway. It must not reference secrets.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **23.1.1** | G46 socket-level over-limit test | `tests/rate_limit_socket.rs` (new) | Baseline |
| **Wave 1** | **23.1.2** | G50 poll flag files instead of 948 sleep | `tests/integration_tests.rs` (hooks test only) | Baseline |
| **Wave 1** | **23.1.3** | G57 spawn plan as a pure function | `src/cli.rs` (`Commands::Spawn` arm, new fn, `mod tests`) | Baseline |
| **Wave 1** | **23.1.4** | G51 CI workflow file | `.github/workflows/ci.yml` (new) | Baseline |
| **Wave 2** | **23.2.1** | G55 depth-limit test can fail | `src/daemon.rs` (`find_registered_parent_ancestor`, its test) | Wave 1 gate |
| **Wave 2** | **23.2.2** | G47 README Socket Contract bullets | `README.md` (Socket Contract section only) | Wave 1 gate (uses G46) |
| **Wave 3** | **23.3.1** | G49 barrier replaces 633 and 638 sleeps | `src/daemon.rs` (`DaemonEvent`, `run_daemon_loops` match), `tests/integration_tests.rs` (focus test only) | Wave 2 gate |
| **Wave 3** | **23.3.2** | G56 golden for `subscribe_diff` bytes, before any change | `tests/diff_golden.rs` (new) | Wave 2 gate |
| **Wave 4** | **23.4.1** | G56 `DiffFrame` shared type | `src/daemon.rs` (`broadcast_diff`, `broadcast_status` diff path), `src/cli.rs` (`fold_diff_line`) | Wave 3 gate |
| **Wave 5** | **23.5.1** | G54 window lifecycle in `DaemonState` | `src/daemon.rs` (`register_agent`, `register_agent_window`, WindowCreated arm, `promote_orphans`, `apply_process_tree_auditing`), `src/layout.rs` (no shape change) | Wave 4 gate |

**Disjointness check.**
- Wave 1: `tests/rate_limit_socket.rs`, `tests/integration_tests.rs`, `src/cli.rs`, `.github/workflows/ci.yml`. Pairwise disjoint.
- Wave 2: `src/daemon.rs`, `README.md`. Disjoint.
- Wave 3: `src/daemon.rs` and `tests/integration_tests.rs` (23.3.1), `tests/diff_golden.rs` (23.3.2). Disjoint.
- Wave 4 and 5: one `src/daemon.rs` task each. Sequenced by wave.

**Orchestrator steps, not worker tasks:**
- After each wave: update `planning/GOALS.md` and `planning/STATE_23.md`.
- After Wave 5: confirm `git status` clean; leave the push to the user.

---

## Task Specifications

### Task 23.1.1: G46 Socket-Level Over-Limit Test

**Files-in-scope**: must create `tests/rate_limit_socket.rs`. Use `mod common;` and `common::unique_path`.

**Spec prose**:
1. Start the listener with `swayve::daemon::start_daemon_ipc_with_limits(path, tx, IpcLimits::default())`, and drain the event channel, as `tests/slow_consumer.rs` does.
2. One connection sends 250 `{"method":"daemon.focus"}` lines.
3. Count `DaemonEvent::Focus` within 2 s. Assert the count is between 200 and 205 inclusive (H1).
4. Poll: send one `daemon.focus` every 20 ms, up to 2 s, until one `Focus` is delivered. Assert that happens. This shows the connection stayed open and the limit recovered.
5. Do not add a sleep to order events.

**Anchors**: `tests/slow_consumer.rs` connection setup (top of `test_stalled_diff_subscriber_recovers_with_snapshot`); `src/ratelimit.rs` (`IPC_BURST`).

**Acceptance**: `cargo test --test rate_limit_socket` passes 5 runs. Zero warnings.

---

### Task 23.1.2: G50 Poll the Hook Flag Files

**Files-in-scope**: must edit `tests/integration_tests.rs` (the `test_hooks_and_monitoring` function only).

**Spec prose**:
1. Replace `tokio::time::sleep(Duration::from_millis(300)).await;` before the flag reads with a poll: every 10 ms, up to 2 s, until both `spawn_flag_path` and `monitor_flag_path` exist and are non-empty.
2. Keep every assertion unchanged.
3. Do not change the hook scripts or config.

**Anchors**: `async fn test_hooks_and_monitoring` (`tests/integration_tests.rs` ~843); the sleep is ~948 (after `reg_sub`).

**Acceptance**: `cargo test --test integration_tests` passes 5 runs. The hooks test has no fixed sleep.

---

### Task 23.1.3: G57 Spawn Plan as a Pure Function

**Files-in-scope**: must edit `src/cli.rs` (`Commands::Spawn` arm, a new `fn spawn_plan`, `mod tests`).

**Spec prose**:
1. Read the whole `Commands::Spawn` arm (starts ~228) before planning. Magic Wand read only its first 40 lines.
2. Move the pure rules into `fn spawn_plan(args, now_ms: u64) -> SpawnPlan`: prompt escaping, `bash -c` wrapping, and derivation of `agent_id`, `app_id`, and title from the clock.
3. The arm builds the plan, then does the socket register and terminal launch.
4. Tests: escaping of quotes and `$`; the ID shape is stable for a fixed `now_ms`; two different `now_ms` give different IDs.
5. Do not change the command-line flags or their parsing.

**Constraints**: no change to `Commands::Spawn` fields; no behavior change visible to a user.

**Acceptance**: `cargo test --lib spawn_plan` runs at least 3 tests and passes. `cargo check` shows no warnings.

---

### Task 23.1.4: G51 CI Workflow File

**Files-in-scope**: must create `.github/workflows/ci.yml`.

**Spec prose**:
1. One job on `ubuntu-latest`, on `push` and `pull_request`.
2. Steps: `actions/checkout`; install stable Rust (`dtolnay/rust-toolchain@stable`); `actions/setup-node` for the Pi client; run `make all`.
3. Install `sway` is not needed. The tests do not require it.
4. Pin action versions to major tags. No secrets.
5. Do not push. The file is committed locally only.

**Acceptance**: the YAML parses (`python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci.yml'))"`, or `actionlint` if installed). No secrets referenced.

---

### Task 23.2.1: G55 Depth-Limit Test Can Fail

**Files-in-scope**: must edit `src/daemon.rs` (`find_registered_parent_ancestor` and `test_early_termination_at_depth_limit` only).

**Spec prose**:
1. Change `find_registered_parent_ancestor(pid, pid_to_agent, max_depth)` to take `parent_of: impl Fn(u32) -> Option<u32>` in place of the `/proc` read. Production passes `|p| get_parent_pid(p)`.
2. Rewrite `test_early_termination_at_depth_limit` to use a map-backed chain longer than the limit: `100 -> 101 -> ... -> 110`, with the registered agent at PID 105, and `max_depth = 3`. Assert `None`. Then `max_depth = 10`, assert `Some`.
3. Keep the other callers working.

**Anchors**: `fn get_parent_pid` (~688); `fn find_registered_parent_ancestor` (~702); `test_early_termination_at_depth_limit` (~819).

**Acceptance**: `cargo test --lib depth` passes. Removing the depth check makes the test fail (verify once by hand, then revert). `make all` zero warnings.

---

### Task 23.2.2: G47 Document the Rate Limit in the Socket Contract

**Files-in-scope**: must edit `README.md`, the Socket Contract section only.

**Spec prose**:
1. Add three bullets under "Socket Contract": the per-client limit (`ipc_lines_per_sec`, default 100, sustained; `ipc_burst`, default 200), over-limit lines dropped and logged as `[daemon] Rate limit exceeded; dropping IPC line`, and the client stays connected with no error reply.
2. Link to `docs/decisions/0016-configurable-ipc-rate-limit.md`.
3. Do not repeat the config section (already done in G48).

**Acceptance**: the bullets match `src/ratelimit.rs` and `src/daemon.rs` `handle_client`. Every link resolves.

---

### Task 23.3.1: G49 Barrier Event Replaces Focus Sleeps

**Files-in-scope**: must edit `src/daemon.rs` (`DaemonEvent` enum, the `run_daemon_loops` match) and `tests/integration_tests.rs` (`test_focus_follows_sway_focus` only).

**Spec prose**:
1. Add `DaemonEvent::Sync(tokio::sync::oneshot::Sender<()>)`. The `run_daemon_loops` arm replies with `let _ = tx.send(());`.
2. In the focus test, replace the sleeps at ~633 and ~638 with: send the `WindowFocused` event, then send `Sync` and await the reply, then write the `daemon.focus` line.
3. Every other `match` on `DaemonEvent` must still compile. Add the arm to each.
4. Do not send `Sync` over the socket. Do not change any wire format.

**Acceptance**: `cargo test --test integration_tests` passes 5 runs. `grep -n "sleep" tests/integration_tests.rs` shows the focus test has no sleep (the 616/621 sleeps are gone). `subscribe_golden` passes unchanged. Zero warnings.

---

### Task 23.3.2: G56 Golden for `subscribe_diff` Bytes

**Files-in-scope**: must create `tests/diff_golden.rs`. Use `mod common;`.

**Spec prose**:
1. Start the daemon listener, as `tests/subscribe_golden.rs` does.
2. Register one agent. Connect to `daemon.subscribe_diff` and read the first line. Assert it is byte-exact to a literal captured from the current code, with `serde_json` key order as produced today.
3. Send one `UpdateStatus`. Read the next line and assert its bytes.
4. Use one agent only, so `HashMap` order cannot change the output (H8).
5. Capture the literals from a run of the current code before any change. Record them in the test with a comment saying they were captured at `<SHA>`.

**Acceptance**: `cargo test --test diff_golden` passes. This test is the guard for 23.4.1.

---

### Task 23.4.1: G56 Shared `DiffFrame` Type

**Files-in-scope**: must edit `src/daemon.rs` (`broadcast_diff`, and the diff path of `broadcast_status` if it builds the JSON), `src/cli.rs` (`fold_diff_line`). New type goes in `src/types.rs` (append only).

**Spec prose**:
1. Add `#[derive(Debug, Serialize, Deserialize, PartialEq)] pub struct DiffFrame { pub snapshot: bool, pub upsert: Vec<AgentWindow>, pub remove: Vec<AgentId> }`. Field order must match the current JSON: `snapshot`, `upsert`, `remove`.
2. Daemon builds `DiffFrame` and serializes it. `remove` serializes as the id string (use `AgentId`'s existing serde form, which is a string; confirm).
3. `fold_diff_line` parses `DiffFrame`. A non-JSON line still returns `false` (H4 of Orchestration 22).
4. `tests/diff_golden.rs` (23.3.2) must pass unchanged. If it fails, stop and report: the bytes changed.

**Acceptance**: `cargo test --test diff_golden` passes. `cargo test --lib fold` passes. `cargo test --test subscribe_golden` passes. Zero warnings.

---

### Task 23.5.1: G54 Window Lifecycle Owned by `DaemonState`

**Files-in-scope**: must edit `src/daemon.rs` (`register_agent`, `register_agent_window`, the WindowCreated and WindowDestroyed arms, `promote_orphans`, `apply_process_tree_auditing`, a new `DaemonState::on_window_created` and `on_window_destroyed`). `src/layout.rs` may gain a `MonitorRequest` type only; no change to `LayoutAction`.

**Spec prose**:
1. Add `pub enum MonitorRequest { Spawn { agent_id: AgentId } }` in `src/daemon.rs`.
2. `DaemonState::on_window_created(con_id, app_id, pid) -> (Vec<LayoutAction>, Vec<MonitorRequest>)` runs, in order: register the window, promote orphans, process-tree audit, then the monitor decision.
3. `DaemonState::on_window_destroyed(con_id) -> Vec<LayoutAction>` keeps today's behavior.
4. One function, `fn auto_monitor_wanted(&self, subagent_evidence: bool) -> bool`, returns `subagent_evidence && self.config.daemon.auto_monitor`. Both `register_agent` and `on_window_created` call it (D3).
5. The loop executes each `MonitorRequest` by calling `spawn_monitor_terminal`. The loop no longer calls it directly.
6. `test_auto_monitor_subagent_registration` must be extended to assert the monitor request is returned, without a channel.

**Constraints**: `LayoutAction` shape unchanged (H5). Order from H6. No change to `subscribe` output.

**Acceptance**: `cargo test --lib auto_monitor` passes. `cargo test --test integration_tests` passes 5 runs. `subscribe_golden` and `diff_golden` pass unchanged. Zero warnings.

---

## Wave Gates

- **Baseline (before Wave 1)**: `make all` green at `48b8d61`. Unit 60 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1. Recorded in `STATE_23.md`.
- **Wave 1 gate**: `make all` serially. Expect unit 60 (no new unit tests from 23.1.1, 23.1.2 or 23.1.4; 23.1.3 adds at least 3); integration 11; new `rate_limit_socket` 1; `subscribe_golden` 1.
- **Wave 2 gate**: `make all` serially. `cargo test --lib depth` passes.
- **Wave 3 gate**: `make all` serially. `diff_golden` 1 passes; integration suite 5 runs.
- **Wave 4 gate**: `make all` serially. `diff_golden` and `subscribe_golden` unchanged.
- **Wave 5 gate**: `make all` serially, and clippy clean. Integration suite 5 runs.
- **Post-wave**: `git status` clean. Nothing pushed.
