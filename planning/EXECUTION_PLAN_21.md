# Execution Plan 21: Diff-Path Hardening and Robustness Follow-ups (G29-G37)

## Goal & Scope

**Goal**: From `planning/GOALS.md` (2026-11-08), close the gaps left by Orchestration 20:
- **G29**: `daemon.subscribe` bytes are pinned by a golden test.
- **G30**: A stalled diff subscriber recovers with a snapshot once it resumes reading (socket level).
- **G31**: The diff path is measured at the G9 N and K grid.
- **G32**: `AgentWindow` comparison is decided from the G31 result (conditional, see D3).
- **G33**: `swayve status --watch --diff` opts into the diff stream.
- **G34**: Foreign-owner checks in `ensure_private_dir` and `start_daemon_ipc` are tested by injecting the owner uid.
- **G35**: A non-UTF-8 IPC line is skipped, and the client stays connected.
- **G36**: A relative or empty `XDG_CONFIG_HOME` logs a one-line warning before falling back.
- **G37**: Startup `sleep` calls in integration tests are replaced with readiness checks.

**In scope:** all nine goals above, each with its exit criteria in GOALS.md.

**Out of scope:**
- Any change to the `daemon.subscribe` wire format. G29 pins it; it does not change it.
- The external HUD. The diff stream is documented; no client is built here.
- G15 and G25 (external sidecar).
- Pushing to any remote. All commits stay local.
- Per-subscriber memory reduction beyond what G32 decides (no change to the `sent` map design).

**Verbatim constraints** (from GOALS.md): "`daemon.subscribe` output is byte-identical to before." (G9, G29). "`make all` passes." (every goal). "An automated test asserts that the socket directory is created with mode 0700 and owned by the current uid, and that stale-socket removal refuses a non-socket and a foreign-owned path." (G22, now G34).

## Decision Log

- **D1: G35 policy, skip and keep client** (2026-11-08, user). A non-UTF-8 line is logged and skipped. The client stays connected. This matches how malformed JSON is already handled (G26).
- **D2: G34 inject the expected uid through inner functions** (2026-11-08, user). `ensure_private_dir` and `start_daemon_ipc` keep their public signatures and call `ensure_private_dir_for(p, uid)` and `prepare_socket_path(path, uid)` with `current_uid()`. Tests call the inner functions with a different uid. ADR 0012's "code review only" statement for foreign owners is amended after this orchestration (post-wave).
- **D3: G32 is decided by the G31 measurement, not now** (2026-11-08, user). The user first answered "derive PartialEq now", then replaced that answer with "do 31 measurement before 32, based on result we will include or not". The later answer governs. Threshold (user): if the diff broadcast at 50 agents with 10 subscribers, release build, costs more than 1 ms, derive `PartialEq` on `AgentWindow` and compare with `==`. Otherwise keep `serde_json::to_value` and record the result in ADR 0013.
- **D4: G33 adds a `--diff` flag; default stays full-list** (2026-11-08, user). `status --watch` is unchanged without the flag.
- **D5: Worker tiers** (carried from Plan 19 D5). Mid-tier for `src/` and `tests/`. Small for docs and pure planning.
- **D6: Wave gates run serially** (carried from Plan 19 D6). Full `make all` at each wave end.
- **D7: G29 and G30 live in new test files** (2026-11-08, default). `tests/subscribe_golden.rs` and `tests/slow_consumer.rs`. This keeps them disjoint from G37's edits to `tests/integration_tests.rs`.
- **D8: Ground rule for G37** (2026-11-08, default). Only startup sleeps (the sleep right after spawning the listener or daemon) are replaced. Sleeps that order daemon events are not removed, because a readiness check cannot replace them. Each remaining sleep is counted in the report.

## Hazard Notes

- `src/cli.rs:84` `current_uid` and `src/cli.rs:100` the owner check in `ensure_private_dir`. The inner function `ensure_private_dir_for` takes `expected_uid: u32`.
- `src/daemon.rs:287` the stale-socket check in `start_daemon_ipc` (`md.uid() == crate::cli::current_uid()?`). Extract it to `prepare_socket_path(path, expected_uid)`. Do not change its refusal behavior.
- `src/daemon.rs:326` `lines.next_line()` in `handle_client`. A non-UTF-8 line makes it return `Err`, which closes the client. Replace it with a `read_until(b'\n')` loop on bytes and `std::str::from_utf8`.
- `tests/integration_tests.rs` has 21 `tokio::time::sleep` calls (lines 41-1104). Only startup sleeps are replaced (D8).
- The `status --watch` parser test at `src/cli.rs:664` destructures `Commands::Status { watch, json }` without `..`. Adding a `diff` field requires updating that test in the same task.
- `G30` needs the subscriber's writer task to block on a full socket buffer. A diff stream with small changes will not fill it. The test must send large `app_id` values (about 8 KiB each) across many updates.
- Edition 2024: `std::env::set_var` is `unsafe`. Do not mutate env vars in tests. G36 tests call a pure function.
- `src/types.rs:31` `AgentWindow` derives `Debug, Clone, Serialize, Deserialize` only. G32 adds `PartialEq` only if D3 says so.
- ADR 0013 Negative bullet on `to_value` cost and ADR 0012 foreign-owner statement are both amended post-wave (see Wave Gates).
- Existing test helper pattern: `start_daemon_ipc` + `run_daemon_loops` as in `tests/integration_tests.rs` `test_dashboard_subscription_stream` (~line 1063 after Orchestration 20's test).

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **21.1.1** | G31 diff-path measurement (ignored test, diff subscribers) | `src/daemon.rs` (tests module only) | None |
| **Wave 1** | **21.1.2** | G34a `ensure_private_dir_for` and tests | `src/cli.rs` (`ensure_private_dir` and tests) | None |
| **Wave 1** | **21.1.3** | G36 XDG warning | `src/config.rs` (`config_path_from`/`load_config` and tests) | None |
| **Wave 1** | **21.1.4** | G37 startup sleeps to readiness checks | `tests/integration_tests.rs` | None |
| **Wave 1** | **21.1.5** | G29 golden test for `daemon.subscribe` bytes | `tests/subscribe_golden.rs` (new) | None |
| **Wave 1** | **21.1.6** | G30 socket-level slow-consumer recovery test | `tests/slow_consumer.rs` (new) | None |
| **Wave 2** | **21.2.1** | G35 non-UTF-8 skip policy and G34b `prepare_socket_path` | `src/daemon.rs` (`handle_client`, `start_daemon_ipc`, tests module) | Wave 1 gate |
| **Wave 3** | **21.3.1** | G32 conditional: derive `PartialEq`, use `==` in `broadcast_diff` | `src/types.rs` (`AgentWindow` derive), `src/daemon.rs` (`broadcast_diff` only) | Wave 2 gate; **runs only if D3 threshold is met** |
| **Wave 3** | **21.3.2** | G33 `status --watch --diff` | `src/cli.rs` (`Status` variant, `run_status_watch`, parser test) | 21.1.6 |

**Disjointness check.**
- Wave 1: `src/daemon.rs` (21.1.1), `src/cli.rs` (21.1.2), `src/config.rs` (21.1.3), `tests/integration_tests.rs` (21.1.4), `tests/subscribe_golden.rs` (21.1.5), `tests/slow_consumer.rs` (21.1.6). Pairwise disjoint.
- Wave 2: one task, so no within-wave conflict. `src/daemon.rs` is touched by 21.1.1 in Wave 1, which completes first.
- Wave 3: `src/types.rs` plus `src/daemon.rs` (21.3.1) versus `src/cli.rs` (21.3.2). Disjoint.

---

## Task Specifications

### Task 21.1.1: G31 Diff-Path Measurement

**ID**: `21.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` inside `mod tests` only (append after `measure_broadcast_cost`)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `#[test] #[ignore]` named `measure_broadcast_cost_diff` in `mod tests`.
2. Same grid as `measure_broadcast_cost`: N in `[5, 20, 50, 200]`, K in `[1, 10]`. Build a `DaemonState` with N agents. Add K subscribers with `push_diff_subscriber` and `mpsc::channel(10_000)`, keeping receivers alive.
3. Call `broadcast_status()` once before timing, so every subscriber has its snapshot and `sent` is populated.
4. Then time 200 calls where each call first changes one agent's status (`update_status`), then calls `broadcast_status()`. Record mean microseconds for the timed section.
5. Print `N=.. K=.. mean_us=.. mode=diff`.
6. Also record the line count and average line size of one diff broadcast.
7. Do not change non-test code.

**Constraints (do not touch)**: non-test code in `src/`; `tests/`; `docs/`.

**Grep anchors**:
- `fn measure_broadcast_cost` (`src/daemon.rs`, the existing ignored test, the template)
- `pub fn push_diff_subscriber` (`src/daemon.rs`)
- `pub fn update_status` (`src/daemon.rs`)

**Acceptance**:
- `cargo test --lib measure_broadcast_cost_diff -- --ignored --nocapture` prints 8 lines and passes, in debug and in `--release`.
- `cargo test --lib` passes (the ignored test is skipped).

**Targeted test**: `cargo test --release --lib measure_broadcast_cost_diff -- --ignored --nocapture` and the debug run.

---

### Task 21.1.2: G34a `ensure_private_dir_for`

**ID**: `21.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs` (`ensure_private_dir` and `mod tests` only)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `pub fn ensure_private_dir_for(p: &Path, expected_uid: u32) -> Result<()>` with the body of `ensure_private_dir` but comparing `md.uid()` with `expected_uid`.
2. Make `ensure_private_dir(p)` call `ensure_private_dir_for(p, current_uid()?)`. Its behavior and signature do not change.
3. Add tests in `mod tests`:
   - `test_ensure_private_dir_for_rejects_foreign_uid`: create a 0700 dir in a unique temp path, call `ensure_private_dir_for(&p, current_uid()? + 1)` (use `wrapping_add` to avoid overflow), assert `Err`, remove the dir.
   - `test_ensure_private_dir_for_accepts_matching_uid`: same dir, `current_uid()`, assert `Ok`.
4. Use unique temp names (`swayve_epdf_<pid>`) so tests do not collide with the existing `swayve_epd_*` tests.
5. Remove directories before asserting, so a failed assert does not leak them.

**Constraints (do not touch)**: the mode check (`mode & 0o077`) behavior; `current_uid`; any function outside `ensure_private_dir` and `mod tests`.

**Grep anchors**:
- `pub fn ensure_private_dir` and `pub fn current_uid` (`src/cli.rs` ~84, ~100)
- `test_ensure_private_dir_creates_0700` (existing reference test, Orchestration 19)

**Acceptance**:
- `cargo test --lib test_ensure_private_dir_for` runs 2 tests, both pass.
- `cargo test --lib test_ensure_private_dir` runs the 4 total tests, all pass.

**Targeted test**: `cargo test --lib test_ensure_private_dir`

---

### Task 21.1.3: G36 XDG Warning

**ID**: `21.1.3`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/config.rs` (`load_config` and `mod tests` only)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `pub fn xdg_config_warning(xdg: Option<&str>) -> Option<&'static str>` returning `Some("XDG_CONFIG_HOME is relative or empty; ignoring and using $HOME/.config")` when `xdg` is `Some(x)` and `x` is empty or does not start with `/`. Otherwise `None`. Pure.
2. In `load_config`, when the warning is `Some`, print it with `eprintln!("[config] {}", msg)` once, then continue as today.
3. Add 4 tests named `test_xdg_warning_*`, one per case: absolute gives `None`; relative gives `Some`; empty gives `Some`; unset gives `None`.
4. Do not change `config_path_from` behavior.

**Constraints (do not touch)**: `config_path_from`; `parse_config_file`; any env mutation in tests.

**Grep anchors**:
- `pub fn config_path_from` and `pub fn load_config` (`src/config.rs`)
- `test_config_path_relative_xdg_falls_back_to_home` (existing reference test)

**Acceptance**:
- `cargo test --lib test_xdg_warning` runs 4 tests, all pass.
- `cargo test --lib test_config_path` still passes (4 tests).

**Targeted test**: `cargo test --lib test_xdg_warning`

---

### Task 21.1.4: G37 Startup Sleeps to Readiness Checks

**ID**: `21.1.4`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `tests/integration_tests.rs` (helper addition at top, and startup sleeps only)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add a helper `async fn wait_for_socket(path: &str)` that polls `UnixStream::connect` every 10 ms, up to 2 s, and panics with the path on timeout.
2. For each test that spawns `start_daemon_ipc` and `run_daemon_loops`, replace the startup `tokio::time::sleep(...)` that follows the spawns with `wait_for_socket(socket_path).await`.
3. Leave any sleep that orders daemon events (for example, after a send and before a read). Count them.
4. Do not change assertions.
5. Report: number of startup sleeps replaced, number of event-ordering sleeps left, and the sleep line numbers left.

**Constraints (do not touch)**: test assertions; `src/`; other test files.

**Grep anchors**:
- `tokio::time::sleep` (21 occurrences in `tests/integration_tests.rs`)
- `async fn test_dashboard_subscription_stream` (the subscription pattern)
- `fn recv_command` (existing timeout helper)

**Acceptance**:
- `cargo test --test integration_tests` passes (11 tests), run 5 times in a row with no failure.
- Startup sleeps replaced: the number reported is the count of spawn-then-sleep sites.

**Targeted test**: `cargo test --test integration_tests` (5 consecutive runs).

---

### Task 21.1.5: G29 Golden Test for `daemon.subscribe`

**ID**: `21.1.5`
**Repo**: `swayve`
**Files-in-scope**:
- must create: `tests/subscribe_golden.rs`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Create `tests/subscribe_golden.rs` with `#[tokio::test]` `test_subscribe_full_list_golden_bytes`. Use the same daemon startup pattern as `tests/integration_tests.rs` (`start_daemon_ipc` + `run_daemon_loops` on a unique socket `/tmp/swayve_test_golden.sock`, plus a `wait_for_socket` readiness helper defined locally).
2. Register exactly 2 agents with fixed ids, project, pid, app_id, and parent (no parent), and set one agent to `AwaitingHumanInput`.
3. Connect a `daemon.subscribe` client. Read its first line.
4. Assert the line is byte-identical to a golden string written in the test. Construct the golden string from the fixed state: a JSON array of two `AgentWindow` objects in the order the daemon serializes them. First run: capture the actual bytes, check them by hand against the `AgentWindow` field order and `AgentStatus` serde format, then paste as the golden. Do not paste blindly.
5. Also assert the line does not begin with `{` (it is a bare array).
6. Remove the socket at the end.

**Constraints (do not touch)**: any `src/` file; `tests/integration_tests.rs`; `tests/slow_consumer.rs`.

**Grep anchors**:
- `pub struct AgentWindow` (`src/types.rs:31`) for field order
- `pub enum AgentStatus` (`src/types.rs`) for status serde format
- `fn broadcast_status` (`src/daemon.rs:~50`) for the serialization call

**Acceptance**:
- `cargo test --test subscribe_golden` passes.
- The golden string was checked by hand against field order and status format, and the report says so.

**Targeted test**: `cargo test --test subscribe_golden`

---

### Task 21.1.6: G30 Socket-Level Slow-Consumer Recovery

**ID**: `21.1.6`
**Repo**: `swayve`
**Files-in-scope**:
- must create: `tests/slow_consumer.rs`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Create `tests/slow_consumer.rs` with `#[tokio::test]` `test_stalled_diff_subscriber_recovers_with_snapshot`. Use the same daemon startup pattern and a local `wait_for_socket` helper, on socket `/tmp/swayve_test_slow.sock`.
2. Connect a `daemon.subscribe_diff` client and send the subscribe line. Do not read from it yet.
3. Register one agent. Then send many status updates (for example 300) with a large `app_id` on the agent (about 8 KiB), so the socket buffer and the 32-slot channel both fill. Each update is a new status value so each produces a diff.
4. Wait briefly (up to 2 s, polling) for the daemon to finish processing the events, using a status query through a second `daemon.subscribe` connection (read its first line) as the ordering check.
5. Now read the diff stream line by line with a timeout on every read. Apply each line to a `HashMap` (snapshot clears, upsert and remove apply). Count snapshot lines.
6. Assert at least 2 snapshot lines were seen (the first, and at least one recovery after a drop).
7. Assert the final map equals the daemon's final agent list from a fresh `daemon.subscribe` read, sorted by id.
8. Remove the socket at the end. The test is bounded: at most 1000 reads, each with a timeout.

**Constraints (do not touch)**: any `src/` file; other test files.

**Grep anchors**:
- `fn broadcast_diff` (`src/daemon.rs` ~85): the drop path (`TrySendError::Full`) sets `needs_snapshot`.
- `test_diff_stream_matches_daemon_state` (`tests/integration_tests.rs`, end of file): the replay pattern.

**Acceptance**:
- `cargo test --test slow_consumer` passes, run 5 times in a row.
- The test fails if the drop path is removed: check this by reasoning, not by editing src (record in report).

**Targeted test**: `cargo test --test slow_consumer` (5 runs).

---

### Task 21.2.1: G35 Skip Policy and G34b `prepare_socket_path`

**ID**: `21.2.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` (`handle_client` read loop, `start_daemon_ipc` stale-check extraction, and `mod tests`)
- scratch: `.development/` only

**Depends-on**: Wave 1 gate

**Spec prose**:
1. **G35:** in `handle_client`, replace `lines.next_line()` with a loop that reads bytes with `read_until(b'\n', &mut buf)`. On 0 bytes, end the client. Convert each line with `std::str::from_utf8`. On error, log `[daemon] Skipping non-UTF-8 IPC line` and continue. Otherwise continue with the existing trim-and-parse logic unchanged.
2. **G35 test:** `test_non_utf8_ipc_line_is_skipped`. Use `UnixStream::pair()` and `handle_client` as in `test_malformed_ipc_line_keeps_client`. Write `b"\xff\xfe\n"`, then `{"method":"daemon.focus"}\n`. Assert the next event is `DaemonEvent::Focus`, with a 2 s timeout.
3. **G34b:** extract the stale-socket check in `start_daemon_ipc` into `pub(crate) fn prepare_socket_path(path: &Path, expected_uid: u32) -> Result<()>`. It keeps the same refusal and removal behavior. `start_daemon_ipc` calls it with `current_uid()?`.
4. **G34b tests:**
   - `test_prepare_socket_path_refuses_foreign_uid`: create a socket with `UnixListener::bind` at a unique temp path, drop it, call `prepare_socket_path(&p, uid + 1)` with `wrapping_add`, assert `Err`, and assert the socket file still exists.
   - `test_prepare_socket_path_removes_matching_socket`: same socket, `current_uid()`, assert `Ok` and the file is gone.
5. Do not change `handle_client` routing, `start_daemon_ipc` binding, or any existing test.

**Constraints (do not touch)**: the malformed-JSON branch; the `DaemonEvent` routing; `test_stale_socket_removal_refuses_regular_file`; any file other than `src/daemon.rs`.

**Grep anchors**:
- `async fn handle_client` (`src/daemon.rs` ~320) and `lines.next_line()` (~326)
- `async fn start_daemon_ipc` (`src/daemon.rs` ~199), the stale check near `md.uid() == crate::cli::current_uid()?` (~287)
- `test_malformed_ipc_line_keeps_client` (reference for the pair setup)

**Acceptance**:
- `cargo test --lib test_non_utf8_ipc_line test_prepare_socket_path` runs 3 tests, all pass. Use separate commands, since cargo takes one filter before `--`.
- `cargo test --lib` passes.

**Targeted test**: `cargo test --lib test_non_utf8_ipc_line_is_skipped`, then `cargo test --lib test_prepare_socket_path`.

---

### Task 21.3.1: G32 Conditional `PartialEq`

**ID**: `21.3.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/types.rs` (`AgentWindow` derive line only), `src/daemon.rs` (`broadcast_diff` comparison only)
- scratch: `.development/` only

**Depends-on**: Wave 2 gate; **gate condition**: the 21.1.1 release measurement at N=50, K=10 is above 1 ms (D3). If not, this task is skipped and the orchestrator records "G32 not included" in STATE_21 and ADR 0013 is amended with the measured numbers.

**Spec prose**:
1. Add `PartialEq` to `AgentWindow`'s derive list in `src/types.rs`. Check that every field type implements `PartialEq` (`AgentStatus`, `AgentId`, `ProjectId`); if not, stop and report.
2. In `broadcast_diff` (`src/daemon.rs`), replace `serde_json::to_value(old).ok() != serde_json::to_value(a).ok()` with `old != a`.
3. Do not change the `sent` map shape or the snapshot logic.
4. Add no tests in this task. The existing diff tests and the golden test cover behavior.

**Constraints (do not touch)**: `sent` map design; `broadcast_status` full path; any other `types.rs` item.

**Grep anchors**:
- `pub struct AgentWindow` (`src/types.rs:31`)
- `fn broadcast_diff` and `serde_json::to_value(old)` (`src/daemon.rs`)

**Acceptance**:
- `cargo test --lib` passes.
- `cargo test --test subscribe_golden` passes unchanged (proves the full-list bytes did not move).
- `cargo test --release --lib measure_broadcast_cost_diff -- --ignored --nocapture` records the post-change numbers in STATE_21.

**Targeted test**: `cargo test --lib diff` and `cargo test --test subscribe_golden`.

---

### Task 21.3.2: G33 `status --watch --diff`

**ID**: `21.3.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs` (the `Status` variant, `run_status_watch`, `test_status_watch_cmd_parsing`, and any `Commands::Status` match arms)
- scratch: `.development/` only

**Depends-on**: 21.1.6 (G30 proved diff recovery at socket level)

**Spec prose**:
1. Add `#[arg(long, requires = "watch")] diff: bool` to the `Status` variant, next to `watch`. Use the same attribute style as the existing `watch` field.
2. In `run_status_watch`, take `diff: bool`. When false, behavior is unchanged. When true, send `{"method":"daemon.subscribe_diff"}` and maintain a `HashMap<String, serde_json::Value>`: a snapshot line clears it, then upserts and removes apply.
3. On each line when `diff` is true, render from the map: deserialize the values into `Vec<AgentWindow>`, sort by id, and call `render_agent_tree`. Keep the same clear-screen escape as the full path.
4. Update `test_status_watch_cmd_parsing` (`src/cli.rs` ~664) to destructure `diff` too, and add a case: `["swayve", "status", "--watch", "--diff"]` parses with both set. Add a case: `["swayve", "status", "--diff"]` is rejected (`requires = "watch"`).
5. Update every `Commands::Status { .. }` match arm to include `diff`.
6. Do not change the daemon, types, or the full path.

**Constraints (do not touch)**: `src/daemon.rs`; `src/types.rs`; the `json` output path.

**Grep anchors**:
- `Status {` (`src/cli.rs:53`) and `watch: bool` (`src/cli.rs:57`)
- `Commands::Status { json, watch }` (`src/cli.rs:372`)
- `async fn run_status_watch` (`src/cli.rs:501`) and `fn render_agent_tree`
- `fn test_status_watch_cmd_parsing` (`src/cli.rs:664`)

**Acceptance**:
- `cargo test --lib test_status_watch` passes, including the two new parse cases.
- `cargo check` passes with no unused-variable warnings.

**Targeted test**: `cargo test --lib test_status_watch`

---

## Wave Gates

- **Pre-wave 1 (baseline)**: the last serial `make all` (after the Orchestration 20 wave 3 gate and the docs commits) gave 41 unit (1 ignored), 11 integration, Pi 3/3, hook test passed. No code has changed since, so this is the baseline. Record in STATE_21.
- **Wave 1 end gate**: `make all` serially. Expect unit 41 + 2 (21.1.2) + 4 (21.1.3) = 47 passed, and 2 ignored (21.1.1 adds one ignored measurement). `integration_tests` 11 (21.1.4 adds no tests). New binaries: `subscribe_golden` 1 and `slow_consumer` 1. Record actual counts.
  - Then run `cargo test --release --lib measure_broadcast_cost_diff -- --ignored --nocapture` and record the N=50 K=10 number in STATE_21.
  - **Decide G32 (D3)**: if the release number at N=50 K=10 is above 1 ms, 21.3.1 runs. Otherwise it is skipped and the decision is recorded.
- **Wave 2 end gate**: `make all` serially. Expect 50 passed (21.2.1 adds 1 + 2). Record actual.
- **Wave 3 end gate**: `make all` serially, then `make test` (Rust, Pi, hook). Expect 50 passed (21.3.2 extends an existing test and adds no new function; 21.3.1 adds none). `subscribe_golden` must pass unchanged. If 21.3.1 ran, record the post-change diff measurement.
- **Post-wave (ADR amendments)**:
  - ADR 0012: amend the "foreign-owner branches are covered by code review only" consequence. Foreign-owner paths are now tested through injected uid (`ensure_private_dir_for`, `prepare_socket_path`).
  - ADR 0013: amend the `to_value` Negative bullet with the outcome of D3 (either "PartialEq derived, cost removed" or "kept, measured at N ms").
  - Run `adr-distillation` for any new enduring decision (G35 skip policy is a candidate: it sets a client-facing rule for bad input).
- **Post-wave (GOALS)**: move G29–G37 to Completed in GOALS.md, with commit SHAs; G32 recorded as "not included" if D3 skipped it.
