# Execution Plan 22: Backlog Close-Out (G39-G43, circled as G44)

## Goal & Scope

**Goal**: Close the backlog left by Orchestration 21, as set out in `planning/GOALS.md` (circled goal G44):
- **G39**: the `status --watch --diff` fold has a behavioural test, not only parse tests.
- **G40**: the 7 "wait for the daemon to process" sleeps in `tests/integration_tests.rs` are replaced with polls of observable daemon state.
- **G41**: the three merged local branches are deleted.
- **G42**: the N=1000 diff-broadcast check. Measured 2026-11-08: release N=1000 K=10 is 11.5 ms with `to_value`, which misses the 10 ms bar. The user's rule was to test `PartialEq`. It takes N=1000 K=10 to about 2.2 ms (two runs: 2.29 and 2.17 ms). This plan lands `PartialEq`.
- **G43**: per-client IPC rate-limit scaffold, wired on. Over-limit lines are dropped and logged.

**In scope:** G39, G40, G41, G42, G43, and the ADR and GOALS updates they require.

**Out of scope:**
- Any change to the `daemon.subscribe` or `daemon.subscribe_diff` wire format. `tests/subscribe_golden.rs` must pass unchanged.
- An ack message from the daemon (rejected for G40, see D2).
- Per-client rate limits in config (constants only, see D1).
- The external HUD and the Antigravity sidecar (G15, G25).
- Pushing to any remote. Commits stay local unless the user says otherwise.
- The three sleeps at `tests/integration_tests.rs` 616, 621 and 931 (see D2).

**Verbatim constraints** (from GOALS.md and the user's Orchestration 22 request):
- "we can revisit G42 and check if N=1000 runs within 10ms, if not we test partialeq and see if it improves."
- "G43 I think a reasonable scaffold for rate-limiting should be included as a matter of good practice."
- "`daemon.subscribe` output is byte-identical to before." (G9, G29)
- "`make all` passes." (every task); zero compiler warnings.

## Decision Log

- **D1: G43 is wired on; drop and log; constants** (2026-11-08, user). Each client connection gets a token bucket: sustained 100 lines/s, burst 200. An over-limit line is dropped, logged as `[daemon] Rate limit exceeded; dropping IPC line`, and the client stays connected. This matches the ADR 0014 skip policy. The values are constants in `src/ratelimit.rs`, not config.
- **D2: G40 polls daemon state; no `src/` change** (2026-11-08, user). Each replaced sleep becomes a poll of observable state, through a second `daemon.subscribe` connection, as in `tests/slow_consumer.rs`. Sleeps that order Sway events (616, 621) or wait for spawned hooks and process execution (931) are not observable through the daemon. They stay and are counted.
- **D3: G42 lands `PartialEq`** (2026-11-08, from the user's rule, applied here). Measured N=1000 K=10 release: 11.5 ms with `to_value`, 2.2 ms with `PartialEq`. The 10 ms bar is missed, and the change improves the number by about 5x. `AgentWindow` contains only `AgentId`, `ProjectId`, `Option<AgentId>`, `Option<i64>`, `String` and `AgentStatus`, all of which already derive `PartialEq`, so the swap cannot change which diffs are sent. ADR 0013 is amended, not superseded.
- **D4: G39 uses a pure helper** (default, not asked). The fold moves into `fn fold_diff_line(map, line) -> bool` in `src/cli.rs`, with unit tests. Testing it over a real socket would need a daemon and a terminal, which is more set-up than the fold needs.
- **D5: G41 runs at wave end, by the orchestrator** (default, not asked). Branch deletion is a local git operation and is not a worker task. `git branch -d` refuses any unmerged branch, so the check is built in.
- **D6: Worker tiers** (carried from Orchestration 21 D5). Mid-tier for `src/` and `tests/`. Small for docs.
- **D7: Wave gates run serially** (carried from Plan 19 D6). Full `make all` at each wave end.
- **D8: Worktree base is verified before approval** (process lesson from Orchestration 21). Every worktree's HEAD must equal `main` before a worker plans. Stale worktrees were the largest source of rework last time.

## Hazard Notes

- **H1 (G43 vs G30 test)**: `tests/slow_consumer.rs:86` sends 300 `UpdateStatus` lines on one connection. A burst of 200 drops about 100 of them. The test's `s299` ordering check and its final-state comparison would then fail. Fix in task 22.2.1: split the 300 sends across connections so that no connection sends more than 150 lines. Assertions are unchanged. The channel still overflows, so the test still proves recovery.
- **H2 (G43 and other tests)**: `tests/integration_tests.rs` sends at most about 20 lines per connection (the `for _ in 0..20` at 1154 is the largest). The probe loops in `wait_for_socket` send no lines. `tests/subscribe_golden.rs` sends 3 lines.
- **H3 (G42 measurement grid)**: the grid lives in `measure_broadcast_cost_diff`, `src/daemon.rs` `mod tests`, currently `for n in [5usize, 20, 50, 200]`. Adding 1000 costs about 15 s in release. The test is `#[ignore]`, so `make all` is not affected.
- **H4 (G39 and the no-JSON branch)**: the current code skips a line that is not JSON with `let Ok(v) ... else { continue }`. `fold_diff_line` must keep that behaviour: a non-JSON line changes nothing and triggers no render.
- **H5 (G43 and the rate limiter's clock)**: `RateLimiter::allow` takes an `Instant`, so unit tests inject times and never sleep. `handle_client` passes `Instant::now()`.
- **H6 (G41 and the current branch)**: the three branches are merged into `main` (verified 2026-11-08). The check is `git branch --merged main`, and each one must appear in it before deletion.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **22.1.1** | G42 `PartialEq` on `AgentWindow`; N=1000 added to measurement grid | `src/types.rs` (derive line only), `src/daemon.rs` (`broadcast_diff` comparison, `measure_broadcast_cost_diff` grid) | None |
| **Wave 1** | **22.1.2** | G39 `fold_diff_line` helper and tests | `src/cli.rs` (`run_status_watch` fold, new fn, `mod tests`) | None |
| **Wave 1** | **22.1.3** | G40 poll-based replacement of 7 sleeps | `tests/integration_tests.rs` | None |
| **Wave 2** | **22.2.1** | G43 `RateLimiter` scaffold, wired into `handle_client`; H1 split in `slow_consumer` | `src/ratelimit.rs` (new), `src/lib.rs` (one line), `src/daemon.rs` (`handle_client` only), `tests/slow_consumer.rs` (send loop only) | Wave 1 gate |
| **Wave 3** | **22.3.1** | ADR 0013 amendment (PartialEq), ADR 0015 (rate-limit policy), catalog update | `docs/decisions/0013-opt-in-diff-status-subscription.md` (Negative bullet only), `docs/decisions/0015-ipc-rate-limit-drop-and-log.md` (new), `docs/decisions/README.md` | Wave 2 gate |

**Disjointness check.**
- Wave 1: `src/types.rs`, `src/daemon.rs` (broadcast_diff and mod tests), `src/cli.rs`, `tests/integration_tests.rs`. Pairwise disjoint.
- Wave 2: one task. Its files do not overlap with anything else in the wave.
- Wave 3: docs only. Disjoint from code.

**Orchestrator steps, not worker tasks:**
- After Wave 3: G41, delete the three merged branches with `git branch -d`.
- After each wave: update `planning/GOALS.md` and `planning/STATE_22.md`.

---

## Task Specifications

### Task 22.1.1: G42 `PartialEq` and the N=1000 Measurement

**ID**: `22.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/types.rs` (the `AgentWindow` derive line only), `src/daemon.rs` (the comparison in `broadcast_diff`, and the `n` grid in `measure_broadcast_cost_diff` inside `mod tests`)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. In `src/types.rs`, add `PartialEq` to `AgentWindow`'s derive. Current line: `#[derive(Debug, Clone, Serialize, Deserialize)]` above `pub struct AgentWindow`. Do not touch any other item.
2. In `src/daemon.rs` `broadcast_diff`, replace `serde_json::to_value(old).ok() != serde_json::to_value(a).ok()` with `old != *a`. `a` is `&&AgentWindow` inside the filter, so the dereference is required. Do not change the `sent` map or the snapshot logic.
3. In `measure_broadcast_cost_diff` (in `mod tests`), change `for n in [5usize, 20, 50, 200]` to `for n in [5usize, 20, 50, 200, 1000]`. Do not change anything else in the test.
4. Add no new tests. The existing diff tests and `subscribe_golden` cover behaviour.
5. Run the release measurement and copy its N=1000 K=10 line into `planning/STATE_22.md`.

**Constraints (do not touch)**: the `sent` map design; `broadcast_status`; `handle_client`; any other item in `types.rs`; `tests/`.

**Grep anchors**:
- `pub struct AgentWindow` (`src/types.rs` ~31)
- `fn broadcast_diff` and `serde_json::to_value(old)` (`src/daemon.rs` ~88 and ~101)
- `fn measure_broadcast_cost_diff` and `for n in [5usize, 20, 50, 200]` (`src/daemon.rs` ~1162)

**Acceptance**:
- `cargo test --lib` passes (50 passed, 2 ignored).
- `cargo test --test subscribe_golden` passes unchanged.
- `cargo test --release --lib measure_broadcast_cost_diff -- --ignored --nocapture` prints a line starting `N=1000 K=10`. The expected value is about 2.2 ms (2165 to 2290 µs), against 11.5 ms before the change.

**Targeted test**: `cargo test --lib diff` and `cargo test --test subscribe_golden`.

---

### Task 22.1.2: G39 `fold_diff_line` Helper

**ID**: `22.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs` (`run_status_watch` diff branch, a new `fn fold_diff_line`, and `mod tests`)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `fn fold_diff_line(map: &mut std::collections::HashMap<String, serde_json::Value>, line: &str) -> bool` in `src/cli.rs`, next to `run_status_watch`. It returns `false` when `line` is not JSON, and changes nothing in that case. Otherwise it returns `true` after folding:
   - `snapshot == true` clears `map`;
   - each object in `upsert` is inserted, keyed by its `id` string;
   - each string in `remove` is removed.
2. In `run_status_watch`, the diff branch calls `fold_diff_line(&mut map, &line)`. When it returns `false`, the loop `continue`s, as the current code does. Leave the rendering code and the non-diff branch unchanged.
3. Add these tests in `mod tests`:
   - `test_fold_diff_snapshot_clears_then_upserts`: start with one entry `a`; fold `{"snapshot":true,"upsert":[{"id":"b", ...}],"remove":[]}`; assert `a` is gone and `b` is present.
   - `test_fold_diff_upsert_then_remove`: fold an upsert of `c`, then a remove of `c`; assert the map is empty.
   - `test_fold_diff_skips_non_json`: fold `not json`; assert it returns `false` and the map is unchanged.
4. Use minimal `AgentWindow`-shaped JSON in the tests (`id`, `project`, `parent_id`, `container_id`, `app_id`, `mark`, `status`).

**Constraints (do not touch)**: the full-list branch; the JSON output path; `render_agent_tree`; `Commands::Status`; the `--diff` flag parsing (`test_status_watch_cmd_parsing`).

**Grep anchors**:
- `async fn run_status_watch` (`src/cli.rs` ~508)
- `let agents = if diff {` and `map.insert(id.to_string(), a.clone());` (`src/cli.rs` ~537 to ~550)
- `fn test_status_watch_cmd_parsing` (`src/cli.rs` ~710), the reference for `mod tests` style

**Acceptance**:
- `cargo test --lib test_fold_diff` runs 3 tests, all pass.
- `cargo test --lib test_status_watch` still passes (1 test).
- `cargo check` shows no warnings.

**Targeted test**: `cargo test --lib test_fold_diff`.

---

### Task 22.1.3: G40 Poll-Based Sleep Replacement

**ID**: `22.1.3`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `tests/integration_tests.rs` (a helper near `wait_for_socket` at the top, and the 7 sleeps listed below)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add a helper `async fn wait_for_status(socket_path: &str, agent_id: &str, want: &str)`. It polls, every 10 ms with a 2 s cap, on a new connection: send `{"method":"daemon.subscribe"}`, read the first line, and return when that line contains `want`. Panic with the agent id on timeout.
2. Replace each of these 7 sleeps with a poll on the state the test asserts next. Read each test before editing to pick the observable state:
   - lines 170, 226, 316: "allow time for daemon socket task to process registration messages"; poll for the registered agent (`want` = its id).
   - lines 409, 511, 597, 693: after `stream.flush()`; poll for the status the test sends (`want` = the status string).
3. Leave these 3 sleeps and do not change them: 616 and 621 (inside a spawned task, between Sway events) and 931 (300 ms for spawned hooks and process execution).
4. If a site asserts something the subscribe line does not show, leave that sleep and count it in the report.
5. Do not change any assertion.
6. Report: the number of sleeps replaced, the number left, and their line numbers.

**Constraints (do not touch)**: assertions; `src/`; `tests/slow_consumer.rs`; `tests/subscribe_golden.rs`; the `wait_for_socket` helper.

**Grep anchors**:
- `tokio::time::sleep` (`tests/integration_tests.rs`, 11 occurrences; the first, at line 22, is inside `wait_for_socket` and stays)
- `fn wait_for_socket` (top of the file), the helper pattern to copy
- `async fn test_diff_stream_matches_daemon_state` (the end-of-file replay pattern, for the second-connection poll)

**Acceptance**:
- `cargo test --test integration_tests` passes (11 tests), run 5 times in a row.
- The report lists exactly 7 sleeps replaced and 3 left (616, 621, 931), or it explains any difference.

**Targeted test**: `cargo test --test integration_tests` (5 runs).

---

### Task 22.2.1: G43 Rate-Limit Scaffold, Wired On

**ID**: `22.2.1`
**Repo**: `swayve`
**Files-in-scope**:
- must create: `src/ratelimit.rs`
- must edit: `src/lib.rs` (one line: `pub mod ratelimit;`), `src/daemon.rs` (`handle_client` only), `tests/slow_consumer.rs` (the send loop only, H1)
- scratch: `.development/` only

**Depends-on**: Wave 1 gate

**Spec prose**:
1. Create `src/ratelimit.rs` with:
   - `pub const IPC_LINES_PER_SEC: f64 = 100.0;` and `pub const IPC_BURST: f64 = 200.0;`
   - `pub struct RateLimiter` holding `capacity: f64`, `refill_per_sec: f64`, `tokens: f64`, `last: std::time::Instant`.
   - `pub fn new(refill_per_sec: f64, capacity: f64, now: Instant) -> Self`, which starts full.
   - `pub fn allow(&mut self, now: Instant) -> bool`: add `elapsed * refill_per_sec` tokens, capped at `capacity`; set `last = now`; if at least 1 token is available, take one and return `true`, otherwise return `false`.
2. In `src/lib.rs`, add `pub mod ratelimit;` next to the other module lines.
3. In `handle_client` (`src/daemon.rs`), create `let mut limiter = crate::ratelimit::RateLimiter::new(IPC_LINES_PER_SEC, IPC_BURST, Instant::now());` next to `let mut buf`. Immediately after `read_until` returns more than 0 bytes, call `if !limiter.allow(Instant::now()) { eprintln!("[daemon] Rate limit exceeded; dropping IPC line"); continue; }`. The check comes before the UTF-8 check, so bad lines count too.
4. Tests in `src/ratelimit.rs` `mod tests`, with injected `Instant` values and no sleeps:
   - `test_rate_limiter_allows_burst_then_denies`: with capacity 3 and a refill of 1 per second, `allow` returns true 3 times and false on the 4th, at one instant.
   - `test_rate_limiter_refills_over_time`: after the denial, `allow` at `now + 1 s` returns true once.
   - `test_rate_limiter_caps_at_capacity`: after 10 s idle, the bucket holds 3 tokens, not 30. `allow` returns true 3 times and false on the 4th.
5. In `tests/slow_consumer.rs`, split the 300 `UpdateStatus` sends (line 86, one connection) across connections so that no connection sends more than 150 lines. Keep the same total, the same payloads and the same order. Change no assertion.

**Constraints (do not touch)**: the malformed-JSON branch; the `DaemonEvent` routing; `prepare_socket_path`; `ensure_private_dir_for`; the `daemon.subscribe` wire format; any assertion in any test.

**Grep anchors**:
- `async fn handle_client` and `read_until(b'\n', &mut buf)` (`src/daemon.rs` ~325 and ~333)
- `pub mod` lines (`src/lib.rs`, 8 lines total)
- `for i in 0..300` (`tests/slow_consumer.rs` ~86), and its `UnixStream::connect` at ~68

**Acceptance**:
- `cargo test --lib ratelimit` runs 3 tests, all pass.
- `cargo test --test slow_consumer` passes, run 5 times in a row (H1 applied).
- `make all` passes, zero warnings.

**Targeted test**: `cargo test --lib ratelimit`, then `cargo test --test slow_consumer` (5 runs).

---

### Task 22.3.1: ADR Amendment and Rate-Limit ADR

**ID**: `22.3.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `docs/decisions/0013-opt-in-diff-status-subscription.md` (the Negative bullet on `to_value` only), `docs/decisions/README.md` (one new row)
- must create: `docs/decisions/0015-ipc-rate-limit-drop-and-log.md`
- scratch: `.development/` only

**Depends-on**: Wave 2 gate

**Spec prose**:
1. Amend the `to_value` Negative bullet in ADR 0013. Replace the "kept" outcome with: `PartialEq` was derived on `AgentWindow` under G42 (Orchestration 22). Release N=50 K=10 was 621 µs with `to_value`, and N=1000 K=10 was 11.5 ms, which missed the 10 ms bar. With `PartialEq` it is about 2.2 ms at N=1000 K=10. Keep the superseded numbers as history.
2. Create ADR 0015 "Per-Client IPC Rate Limit: Drop and Log" using the same front-matter and section layout as ADR 0014. Its context: a client can flood the socket, and the daemon has no per-client limit. Decision: a token bucket per connection, sustained 100 lines/s, burst 200, over-limit lines dropped and logged, client kept connected (D1). Consequences: a legitimate burst over 200 loses lines; the limit is a constant, not config; a client that exceeds it has no error reply. Validation: `src/ratelimit.rs` tests and `tests/slow_consumer.rs`.
3. Add the 0015 row to `docs/decisions/README.md`, after 0014. Use the same column layout.

**Constraints (do not touch)**: any other ADR; any `src/` or `tests/` file; the README's other rows.

**Grep anchors**:
- `Negative**: the daemon serializes agents` (ADR 0013, ~line 37)
- `0014-skip-undecodable-ipc-lines-keep-client.md` (the index row and the front-matter pattern to copy)

**Acceptance**:
- ADR 0015 exists with valid front matter (`id: "0015"`, `status: "Accepted"`).
- The README table has a 0015 row.
- Every link in the new and changed rows resolves to an existing file.

**Targeted test**: `ls docs/decisions/0015-ipc-rate-limit-drop-and-log.md` and a grep for the `0015` row.

---

## Wave Gates

- **Baseline (before Wave 1)**: `make all` green. Unit 50 passed / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1; Pi 3/3; hook test passed. Verified 2026-11-08 at `0fd3c63`. Record in STATE_22.
- **Wave 1 end gate**: `make all` serially. Expect unit 53 passed (50 + 3 from 22.1.2) / 2 ignored; integration 11; `slow_consumer` 1; `subscribe_golden` 1. Record the N=1000 K=10 line from 22.1.1.
- **Wave 2 end gate**: `make all` serially. Expect unit 56 passed (53 + 3 from 22.2.1) / 2 ignored. `slow_consumer` passes 5 runs.
- **Wave 3 end gate**: `make all` and `make test` serially. Expect the Wave 2 counts. `subscribe_golden` unchanged.
- **Orchestrator, after Wave 3**: G41 (`git branch --merged main`, then `git branch -d` each of `docs/adr-distillation`, `docs/agents-md-structure`, `docs/task-4.1-checkoff`). Then update GOALS.md: move G39–G43 to Completed with SHAs, and move G44 to Completed.
- **Post-wave**: confirm `git status` is clean and `git worktree list` shows only `main`.
