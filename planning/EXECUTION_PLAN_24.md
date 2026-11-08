# Execution Plan 24: Spawn Registration Wait (G52, re-scoped) and FocusCycle (G58)

**Status: drafted 2026-11-08, awaiting approval. Not executed.**

## Goal & Scope

**Goal**: close the two deferred goals in `planning/GOALS.md` that the user chose to build:
- **G52, re-scoped**: `swayve spawn` waits until the daemon has applied its registration before it launches the terminal. The wire is not changed.
- **G58**: the focus cycle cursor is owned by a small `FocusCycle` type, and its update rule is unit-tested.

**In scope:**
- A client-side wait in `swayve spawn` that polls `daemon.subscribe` until the new agent id is listed (task 24.1.1).
- `FocusCycle { last }` with `observe()` and `next()`, replacing the `last_focused` local in `run_daemon_loops` (task 24.1.2).

**Out of scope:**
- A daemon delivery ack or any reply on the socket. The original G52 wire ack is pruned (D1).
- Any change to `daemon.subscribe`, `daemon.subscribe_diff`, or any IPC message format. `tests/subscribe_golden.rs` and `tests/diff_golden.rs` must pass unchanged.
- The CLI flat fallback (`next_flat_agent_index`, `focus_cycle_standalone`), which ADR 0003 keeps.
- `next_focus_target`, which stays a pure function with its tests.
- Pushing. Commits stay local.

**Verbatim constraints** (from the user):
- "build it" (G58, Orchestration 24 request).
- "`daemon.subscribe` output is byte-identical to before." (G9, G29)
- "`make all` passes." Zero compiler warnings.

## Decision Log

- **D1: G52 is re-scoped to a client-side wait** (2026-11-08, user). Options were a daemon wire ack, a client-side poll, or leaving the race. The user chose the poll. The wire ack is pruned, so ADR 0009 and ADR 0014 stay as they are. The Claude hooks, the sidecar (G25/G15) and the HUD (G53) do not change.
- **D2: spawn timeout is 2 s, and a timeout launches anyway** (default, not asked). `spawn` polls every 10 ms for up to 2 s. If the id is still not listed, it prints one `[spawn] Warning` line and launches the terminal as before. A slow daemon must not block a spawn forever.
- **D3: G58 builds the small version** (2026-11-08, user). `FocusCycle` holds only the cursor. The cycle rule stays in `next_focus_target`.
- **D4: `next()` moves the cursor only when a target is found and it has an agent** (default). This mirrors the current `Focus` arm, which sets `last_focused` only inside `if let Some(agent)`.
- **D5: Single wave** (default). The two tasks touch disjoint files (`src/cli.rs`, `src/daemon.rs`).
- **D6: Workers and worktrees** (carried from Orchestration 23 D4). Inline by the orchestrator, one commit per task.

## Hazard Notes

- **H1 (spawn and the daemon being down)**: if `connect` fails, the spawn keeps its existing standalone path. The wait only runs after a successful register write.
- **H2 (the wait's connection)**: each poll opens a fresh `daemon.subscribe` connection and reads one line, as `wait_for_listed` does. Each connection is dropped after its read.
- **H3 (flush before the wait)**: the register line must be flushed before the wait starts, so the daemon has the bytes. Use `flush()` after `write_all()`.
- **H4 (FocusCycle and the barrier)**: the `Sync` barrier and `DaemonEvent` are unchanged. `test_focus_follows_sway_focus` must pass 5 runs.
- **H5 (observe rule)**: only agent windows move the cursor. A `WindowFocused` for a con id with no agent is ignored, as today.
- **H6 (`last_focused` is read in one more place)**: the `Focus` arm reads and writes it through `next()`. Keep the `FocusWindow` command and its mark lookup in the loop.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **24.1.1** | G52 re-scoped: spawn waits for registration by polling `daemon.subscribe` | `src/cli.rs` (`Commands::Spawn` arm, new `register_and_wait` fn, `mod tests`), `tests/spawn_register.rs` (new) | Baseline |
| **Wave 1** | **24.1.2** | G58: `FocusCycle` owns the cursor | `src/daemon.rs` (`run_daemon_loops` cursor and its two arms, new `FocusCycle` type, `mod tests`) | Baseline |

**Disjointness check.** `src/cli.rs` + `tests/spawn_register.rs` against `src/daemon.rs`. Pairwise disjoint.

**Orchestrator steps (not worker tasks):** after the wave, update `planning/GOALS.md`: G52 moves to Completed (re-scoped, ack pruned), G58 moves to Completed.

---

## Task Specifications

### Task 24.1.1: Spawn Waits for Its Registration

**ID**: `24.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs` (`Commands::Spawn` arm around line 250-262; a new `pub async fn register_and_wait`; `mod tests`)
- must create: `tests/spawn_register.rs`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `pub async fn register_and_wait(socket_path: &str, reg_line: &str, agent_id: &str, timeout: Duration) -> bool` to `src/cli.rs`, near `run_status_watch`.
   - Write `reg_line` on one connection and flush it.
   - Then poll every 10 ms until `timeout`: open a fresh connection, send `{"method":"daemon.subscribe"}\n`, read the first line with a 100 ms cap, and return `true` when that line contains `"id":"<agent_id>"`.
   - Return `false` when the timeout passes, or when the first connect fails.
2. In the `Commands::Spawn` arm, replace `let _ = stream.write_all(payload.as_bytes()).await;` with a call to `register_and_wait(socket_path, &payload, &agent_id, Duration::from_secs(2))`. If it returns `false`, print one line: `[spawn] Warning: daemon did not list {agent_id} within 2s; launching anyway`. Then continue as before.
3. The `else` branch (connect failed, standalone mode) is unchanged.
4. Do not change the register message, the terminal launch, or any output line other than the new warning.
5. Test `tests/spawn_register.rs`:
   - Start `start_daemon_ipc` and `run_daemon_loops` on a `common::unique_path` socket, as `tests/diff_golden.rs` does.
   - Call `register_and_wait` with a valid register line for `spawn-1`. Assert `true`.
   - Call it with `timeout = 300 ms` for an agent id that is never registered. Assert `false`, and assert the call returned within 1 s.
6. Unit test in `src/cli.rs` `mod tests`: `register_and_wait` returns `false` quickly when no socket exists at the path (use `common::unique_path`-style path via `std::env::temp_dir()`).

**Constraints (do not touch)**: the register message shape; the `daemon.subscribe` wire format; the terminal-launch code; `wait_for_listed` in `tests/integration_tests.rs` (leave it as is).

**Grep anchors**:
- `let payload = serde_json::to_string(&reg_msg)? + "\n";` (`src/cli.rs` ~259)
- `async fn run_status_watch(` (`src/cli.rs` ~545), the reference for the subscribe read
- `async fn wait_for_listed(` (`tests/integration_tests.rs`), the reference poll loop

**Acceptance**:
- `cargo test --test spawn_register` passes 5 runs in a row.
- `cargo test --lib register_and_wait` passes.
- `make all` passes, zero warnings.

**Targeted test**: `cargo test --test spawn_register` (5 runs), then `cargo test --lib register_and_wait`.

---

### Task 24.1.2: G58 FocusCycle Owns the Cursor

**ID**: `24.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` (new `FocusCycle` type near `next_focus_target`; `run_daemon_loops` where `last_focused` is declared at ~520; the `SwayEvent::WindowFocused` arm at ~594; the `DaemonEvent::Focus` arm at ~600; `mod tests`)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add to `src/daemon.rs`:
   ```rust
   /// The focus cycle's cursor: the last agent focused by the cycle or by the user.
   #[derive(Debug, Default)]
   pub struct FocusCycle {
       last: Option<AgentId>,
   }
   ```
   with:
   - `pub fn observe(&mut self, agent: AgentId)`: sets `last`. Called on a manual focus of an agent window (H5).
   - `pub fn next(&mut self, agents: &HashMap<AgentId, AgentWindow>) -> Option<AgentId>`: calls `next_focus_target(agents, self.last.as_ref())`. If it returns `Some(target)` and `agents.contains_key(&target)`, sets `last = Some(target.clone())` and returns `Some(target)`. Otherwise returns `None` and leaves `last` as it was (D4).
2. In `run_daemon_loops`, replace `let mut last_focused: Option<AgentId> = None;` with `let mut focus = FocusCycle::default();`.
3. `SwayEvent::WindowFocused` arm: replace `last_focused = Some(agent_id.clone());` with `focus.observe(agent_id.clone());`. Keep the `con_to_agent.get` check.
4. `DaemonEvent::Focus` arm: replace the `if let Some(target) = next_focus_target(...) && let Some(agent) = state.agents.get(&target) { ... last_focused = Some(target); }` with `if let Some(target) = focus.next(&state.agents) && let Some(agent) = state.agents.get(&target) { ... }`. The `FocusWindow` send is unchanged, and it still reads `agent.mark`. Remove the `last_focused = Some(target);` line.
5. Unit tests in `mod tests`, using the existing `registry` and `mock_agent` helpers:
   - `test_focus_cycle_next_moves_cursor`: with two root agents, `observe` the first, `next` returns the second, and a second `next` returns the first.
   - `test_focus_cycle_next_none_keeps_cursor`: with an empty registry, `next` returns `None` and a later `observe` still applies.
   - `test_focus_cycle_next_requires_agent_in_map`: a target that is not in the map is not returned and does not move the cursor.

**Constraints (do not touch)**: `next_focus_target` and its tests; the `Sync` arm; the flat fallback in `src/cli.rs`; `DaemonEvent`; any wire format.

**Grep anchors**:
- `let mut last_focused: Option<AgentId> = None;` (`src/daemon.rs` ~520)
- `SwayEvent::WindowFocused { con_id } =>` (`src/daemon.rs` ~594)
- `DaemonEvent::Focus =>` (`src/daemon.rs` ~600)
- `fn next_focus_target(` (`src/daemon.rs` ~690) and `fn test_focus_cycles_between_roots` (`src/daemon.rs` ~995)

**Acceptance**:
- `cargo test --lib focus_cycle` runs 3 tests, all pass.
- `cargo test --test integration_tests` passes 5 runs in a row (H4).
- `cargo test --lib test_hierarchical_focus test_focus_cycles_between_roots` still pass.
- `make all` passes, zero warnings.

**Targeted test**: `cargo test --lib focus_cycle`, then `cargo test --test integration_tests` (5 runs).

---

## Wave Gates

- **Baseline (before Wave 1)**: `make all` green at `805fcea` (= `origin/main`). Unit 67 passed / 2 ignored; integration 11; `rate_limit_socket` 1; `slow_consumer` 1; `subscribe_golden` 1; `diff_golden` 1; Pi and hook tests passed; zero warnings.
- **Wave 1 gate**: `make all` serially. Expect unit 71 passed / 2 ignored (67 + 3 `focus_cycle` + 1 `register_and_wait`); integration 11; `spawn_register` 1; the others unchanged; `subscribe_golden` and `diff_golden` pass unchanged. Zero warnings.
- **Post-wave**: update GOALS: G52 to Completed (re-scoped; wire ack pruned) and G58 to Completed. `git status` clean. Nothing pushed.
