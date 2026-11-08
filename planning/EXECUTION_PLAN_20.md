# Execution Plan 20: Public-Release Readiness: Diff Subscriptions (G9) and XDG Config Path (G10)

## Goal & Scope

**Goal**: From `planning/GOALS.md` (2026-11-08), both orthogonal circled goals:
- **G9**: Full-state broadcast cost is reduced to diffs for subscribers that opt in. The full-list stream is unchanged.
- **G10**: `XDG_CONFIG_HOME` is honored for the config path, with `$HOME/.config` as fallback.

**In scope:**
- Baseline measurement of broadcast cost at 5, 20, 50 and 200 agents (G9 gate).
- New `daemon.subscribe_diff` subscription with upserts, removals, and full snapshot on drop.
- `config_path_from` pure function and its tests (G10).
- README Configuration and Socket subscription docs; ADR 0008 and ADR 0010 notes.

**Out of scope:**
- Any change to `daemon.subscribe` output. Its bare-array lines must stay byte-identical.
- Changing `status --watch` (`src/cli.rs` `run_status_watch`) to use diffs. It stays on the full stream.
- Sequence numbers or client-side gap detection. Snapshot-after-drop replaces them (decision D2).
- Making a new ADR for G10. ADR 0010 is amended instead (decision D6).
- Integrations outside this repo (the external HUD). Its diff reader is not built here.

**Verbatim constraints** (from GOALS.md): "`daemon.subscribe` output is byte-identical to before." "`make all` passes."

## Decision Log

- **D1: G9 opt-in, not a format change** (2026-11-08, default). `daemon.subscribe` keeps the bare `Vec<AgentWindow>` line because `status --watch` (`src/cli.rs:501`, `run_status_watch`) parses exactly that, and external clients may too. Public release makes a silent format change a breaking change. New clients send `{"method":"daemon.subscribe_diff"}`.
- **D2: Recovery by snapshot, not sequence numbers** (2026-11-08, default). When a diff subscriber's channel is full, the daemon drops that message and sets `needs_snapshot` on that subscriber. Its next send is a full snapshot. A snapshot replaces the client's whole view, so no client gap detection is needed. This keeps ADR 0008's rule that the next full broadcast restores the view, and does it for diff subscribers too.
- **D3: Diff frame shape** (2026-11-08, default). Each diff line is `{"snapshot":bool,"upsert":[AgentWindow...],"remove":[AgentId...]}`. A snapshot sends every agent in `upsert` with `snapshot: true` and an empty `remove`. A subscriber's first line after subscribing is a snapshot.
- **D4: G10 rule for XDG** (2026-11-08, default). `XDG_CONFIG_HOME` counts only if it is non-empty and absolute, as the XDG Base Directory spec requires. Otherwise the path falls back to `$HOME/.config`. Relative values are ignored, not an error.
- **D5: Worker tier** (2026-11-08, carried from D5 of Plan 19). Mid-tier for `src/` and `tests/`, small for docs.
- **D6: ADR handling** (2026-11-08, default). G10 amends ADR 0010 (its negative bullet). G9 records one new ADR, 0013, for the diff protocol. The post-wave `adr-distillation` run creates it.
- **D7: Measurement gate** (2026-11-08, default). Wave 2 starts only after `20.1.1` numbers are in STATE_20 and the orchestrator has reviewed them. If the numbers show broadcast cost under 1 ms at 50 agents with 10 subscribers, the orchestrator records that and asks the user before Wave 2. Otherwise Wave 2 proceeds.
- **D8: No env mutation in tests** (2026-11-08, default). Edition 2024 makes `std::env::set_var` `unsafe`, and tests run in parallel. G10 tests call `config_path_from` directly.

## Hazard Notes

- `src/daemon.rs:32` `broadcast_status`: serializes the whole list once, then `json.clone()` per subscriber. `subscribers: Vec<mpsc::Sender<String>>` at line 18. Changing that type breaks the existing tests at `src/daemon.rs` ~873-928 that push bare senders. Keep a full-mode push helper, or update those tests in the same task.
- `src/daemon.rs` `DaemonEvent::Subscribe(mpsc::Sender<String>)` (line ~196) and its handler (line ~455). `handle_client` at ~254 routes `AgentIPCMessage::SubscribeStatus` to it. The new variant needs its own routing.
- `src/types.rs:65` `SubscribeStatus` is `#[serde(rename = "daemon.subscribe")]`. Add `SubscribeDiff` with `#[serde(rename = "daemon.subscribe_diff")]`. Do not change the existing rename.
- `src/types.rs:31` `AgentWindow` derives `Serialize`, `Deserialize`. `AgentId` must be `Hash + Eq + Clone` for the diff map. Check before writing (`20.2.1` anchor).
- `src/config.rs:108` `load_config` builds the path from `HOME` with `format!`. Replace it with `config_path_from`. Keep the `Err` branch's `eprintln!` behavior.
- `tests/integration_tests.rs:963` `test_dashboard_subscription_stream` is the reference for the subscription test shape. Do not change it.
- `README.md:151` Configuration section; `README.md:50` Socket Contract section (added 19.1.4). Put the subscription note under Socket Contract.
- `docs/decisions/0008-push-status-subscription-for-hud.md` Consequences: the "slow subscriber" bullet describes the drop-newest rule. Add a sentence for diff subscribers only. Do not rewrite the existing bullet.
- `docs/decisions/0010-toml-config-in-home-config-dir.md` Consequences: the `XDG_CONFIG_HOME is ignored` bullet becomes historical. Amend it with a dated note, do not delete it.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **20.1.1** | G9 broadcast-cost measurement (ignored timing test) | `src/daemon.rs` (tests module only) | None |
| **Wave 1** | **20.1.2** | G10 `config_path_from` and tests | `src/config.rs` | None |
| **Wave 1** | **20.1.3** | G10 docs: README Configuration note, ADR 0010 amendment | `README.md` (Configuration section only), `docs/decisions/0010-toml-config-in-home-config-dir.md` | None |
| **Wave 2** | **20.2.1** | G9 `SubscribeDiff` message and diff broadcast core | `src/types.rs`, `src/daemon.rs` | Wave 1 gate + D7 review |
| **Wave 3** | **20.3.1** | G9 `handle_client` routing and unit tests for snapshot-after-drop | `src/daemon.rs` (routing and tests) | 20.2.1 |
| **Wave 3** | **20.3.2** | G9 integration test: diff stream matches daemon state | `tests/integration_tests.rs` (new test at end only) | 20.3.1 |
| **Wave 3** | **20.3.3** | G9 docs: README Socket Contract subscription note, ADR 0008 amendment | `README.md` (Socket Contract section only), `docs/decisions/0008-push-status-subscription-for-hud.md` | 20.2.1 |

Wave 1 file sets are disjoint: `src/daemon.rs` (tests), `src/config.rs`, `README.md` + ADR 0010. Wave 3 file sets are disjoint: `src/daemon.rs` (routing) and `tests/` and `README.md` + ADR 0008. Note: `20.3.1` and `20.3.3` both touch README-adjacent areas but different sections; run `20.3.3` after `20.3.1` if the orchestrator sees a conflict.

Wave 2 is one task on purpose. `types.rs` and `daemon.rs` change together, and the orchestrator reviews both.

---

## Task Specifications

### Task 20.1.1: G9 Broadcast-Cost Measurement

**ID**: `20.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` inside `mod tests` only
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `#[test] #[ignore]` named `measure_broadcast_cost` in `mod tests`. It must not run under `make all`.
2. For each agent count N in `[5, 20, 50, 200]` and subscriber count K in `[1, 10]`:
   - Build a `DaemonState` (see its constructor, `src/daemon.rs` ~line 22) with N agents registered through the same path the existing tests use.
   - Add K subscribers with `mpsc::channel(10_000)` and keep the receivers alive. Do not drain them, so no channel fills during timing.
   - Time 200 calls to `broadcast_status()` with `std::time::Instant`. Record the mean per call in microseconds.
   - Record the serialized payload size in bytes (the length of the JSON string for one broadcast).
3. Print one line per (N, K): `N=.. K=.. mean_us=.. payload_bytes=..`.
4. Do not change any non-test code.

**Constraints (do not touch)**: non-test code in `src/daemon.rs`; `src/types.rs`; `tests/`; `README.md`; `docs/`.

**Grep anchors**:
- `pub struct DaemonState` / its `new` (`src/daemon.rs` ~line 22)
- `pub fn broadcast_status` (`src/daemon.rs:32`)
- `mod tests` (bottom of `src/daemon.rs`)
- `fn test_daemon_broadcast_drops_newest_when_subscriber_full` (`src/daemon.rs` ~line 900): the reference for building subscribers.

**Reference implementation**: `test_daemon_broadcast_drops_newest_when_subscriber_full` in the same module.

**Acceptance**:
- `cargo test --lib measure_broadcast_cost -- --ignored --nocapture` prints 8 result lines and passes.
- `cargo test --lib` still passes (the ignored test is not run).

**Targeted test**: `cargo test --lib measure_broadcast_cost -- --ignored --nocapture`

---

### Task 20.1.2: G10 `config_path_from`

**ID**: `20.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/config.rs` (the `load_config` function, and `mod tests` at the bottom)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `pub fn config_path_from(xdg: Option<&str>, home: Option<&str>) -> Option<std::path::PathBuf>`.
   - If `xdg` is `Some(x)` with `x` non-empty and starting with `/`, return `x/swayve/config.toml`.
   - Otherwise, if `home` is `Some(h)`, return `h/.config/swayve/config.toml`.
   - Otherwise return `None`.
2. Change `load_config` to read `XDG_CONFIG_HOME` and `HOME` with `std::env::var(..).ok()`, call `config_path_from`, and return `Config::default()` when it returns `None`. The `parse_config_file` call and its `Err` branch stay as they are.
3. Add 4 tests in `mod tests`, named `test_config_path_*`:
   - absolute XDG wins
   - relative XDG falls back to HOME
   - empty XDG falls back to HOME
   - HOME only
   Do not mutate env vars.
4. Do not change `parse_config_file` or any other config behavior.

**Constraints (do not touch)**: `parse_config_file`; the `Config` struct; `src/daemon.rs`; `README.md`; `docs/`.

**Grep anchors**:
- `pub fn load_config` (`src/config.rs:108`)
- `fn parse_config_file` (above `load_config`, ~line 95)
- `mod tests` at the bottom of `src/config.rs`; add the new tests after `test_daemon_auto_monitor_config_parsing`.

**Reference implementation**: `runtime_dir_from` in `src/cli.rs` (`fn runtime_dir_from`) for the pure-function-plus-env-wrapper pattern.

**Acceptance**:
- `cargo test --lib test_config_path` runs 4 tests, all pass.
- `cargo test --lib` passes.

**Targeted test**: `cargo test --lib test_config_path`

---

### Task 20.1.3: G10 Docs

**ID**: `20.1.3`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `README.md` (the Configuration section only, starting at `## ⚙️ Configuration` line ~151)
- must edit: `docs/decisions/0010-toml-config-in-home-config-dir.md` (the `Consequences` section only)

**Depends-on**: None

**Spec prose**:
1. In the README Configuration section, add one short paragraph: the config file is `$XDG_CONFIG_HOME/swayve/config.toml` when `XDG_CONFIG_HOME` is set to an absolute path, otherwise `~/.config/swayve/config.toml`. A relative or empty `XDG_CONFIG_HOME` is ignored.
2. In ADR 0010, do not delete the bullet that says `XDG_CONFIG_HOME` is ignored. Add an amendment line after it: `Amended 2026-11-08 (Orchestration 20): XDG_CONFIG_HOME is honored when absolute; see the Configuration section of README.md.`
3. Change nothing else in either file.

**Constraints (do not touch)**: any other README section; any other ADR; `src/`.

**Grep anchors**:
- `## ⚙️ Configuration` (`README.md:151`)
- `XDG_CONFIG_HOME is ignored` (`docs/decisions/0010-toml-config-in-home-config-dir.md`)

**Acceptance**:
- `grep -c 'XDG_CONFIG_HOME' README.md` returns at least 1.
- `grep -c 'Amended 2026-11-08' docs/decisions/0010-toml-config-in-home-config-dir.md` returns 1.

**Targeted test**: the two greps above.

---

### Task 20.2.1: G9 `SubscribeDiff` and Diff Broadcast Core

**ID**: `20.2.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/types.rs` (the `AgentIPCMessage` enum only)
- must edit: `src/daemon.rs` (the `subscribers` field, `broadcast_status`, and a new `broadcast_diff` path; the `DaemonEvent::Subscribe` handler)
- scratch: `.development/` only

**Depends-on**: Wave 1 gate passed; the orchestrator reviewed the `20.1.1` numbers (D7).

**Spec prose**:
1. In `src/types.rs`, add `SubscribeDiff` to `AgentIPCMessage` with `#[serde(rename = "daemon.subscribe_diff")]`. Do not change `SubscribeStatus`.
2. In `src/daemon.rs`, change `subscribers` to a list of entries, each holding:
   - the `mpsc::Sender<String>`
   - a mode: `Full` or `Diff`
   - for `Diff`: a `needs_snapshot: bool` and a `sent: HashMap<AgentId, AgentWindow>` holding what the client last received
3. Keep `broadcast_status` behavior for `Full` entries exactly as before. Its drop-newest and retain rules do not change.
4. Add a diff broadcast for `Diff` entries. For each entry:
   - If `needs_snapshot` or this is the first send, send a snapshot line: `{"snapshot":true,"upsert":[all current agents],"remove":[]}`.
   - Otherwise compute `upsert` as agents whose value differs from `sent` (or is new), and `remove` as ids in `sent` but not in the current set. Send `{"snapshot":false,"upsert":[..],"remove":[..]}`. Skip the send if both lists are empty.
   - On `Ok`, update `sent` to the current state and clear `needs_snapshot`.
   - On `Full` (channel full): drop this message, set `needs_snapshot = true`, log a line, keep the entry.
   - On `Closed`: remove the entry.
5. `DaemonEvent::Subscribe` keeps its current behavior. Add a separate `DaemonEvent::SubscribeDiff(mpsc::Sender<String>)`, which pushes a `Diff` entry with `needs_snapshot = true`.
6. Existing tests that push bare senders must still compile and pass. Keep a helper such as `push_full_subscriber` for them, or update them in this task.
7. Do not change `handle_client`. That is task `20.3.1`.

**Constraints (do not touch)**: the output of `daemon.subscribe` (byte-identical); `src/cli.rs`; `tests/`; `README.md`; `docs/`.

**Grep anchors**:
- `pub struct DaemonState` field `subscribers` (`src/daemon.rs:18`)
- `pub fn broadcast_status` (`src/daemon.rs:32`)
- `pub enum DaemonEvent` (`src/daemon.rs` ~192)
- `DaemonEvent::Subscribe(sub_tx)` handler (`src/daemon.rs` ~455)
- `pub struct AgentWindow` (`src/types.rs:31`); check that `AgentId` derives `Hash, Eq, Clone` before using it as a map key.
- `SubscribeStatus` (`src/types.rs:65`)

**Reference implementation**: `broadcast_status` (`src/daemon.rs:32`) for the send and retain pattern.

**Acceptance**:
- `cargo check` passes.
- `cargo test --lib` passes (existing tests, including `test_daemon_broadcast_drops_newest_when_subscriber_full`, still pass).
- `grep -n 'SubscribeDiff' src/types.rs` returns at least 1.

**Targeted test**: `cargo test --lib broadcast`

---

### Task 20.3.1: G9 Routing and Snapshot-After-Drop Tests

**ID**: `20.3.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` (`handle_client` routing, and `mod tests`)
- scratch: `.development/` only

**Depends-on**: `20.2.1`

**Spec prose**:
1. In `handle_client`, route `AgentIPCMessage::SubscribeDiff` to `DaemonEvent::SubscribeDiff`. Use the same writer-task pattern as `SubscribeStatus` (`src/daemon.rs` ~255-265).
2. Add a unit test `test_diff_subscriber_gets_snapshot_then_diffs`: register agents, subscribe a diff entry, check the first line is a snapshot, then change one agent's status and check the next line is a diff with that agent in `upsert` and an empty `remove`.
3. Add a unit test `test_diff_subscriber_recovers_with_snapshot_after_drop`: use a channel of capacity 1, fill it, trigger a broadcast so the diff drops, drain, trigger another broadcast, and check the next line is a snapshot with `snapshot: true`.
4. Do not edit existing tests.

**Constraints (do not touch)**: `daemon.subscribe` routing; existing tests; `src/types.rs` (already done in `20.2.1`).

**Grep anchors**:
- `AgentIPCMessage::SubscribeStatus =>` in `handle_client` (`src/daemon.rs` ~258)
- `test_daemon_broadcast_drops_newest_when_subscriber_full` (`src/daemon.rs` ~900): the reference for the capacity-1 setup.

**Acceptance**:
- `cargo test --lib test_diff_subscriber` runs 2 tests, both pass.
- `cargo test --lib` passes.

**Targeted test**: `cargo test --lib test_diff_subscriber`

---

### Task 20.3.2: G9 Integration Test

**ID**: `20.3.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `tests/integration_tests.rs` (add one new `#[tokio::test]` at the end of the file only)
- scratch: `.development/` only

**Depends-on**: `20.3.1`

**Spec prose**:
1. Add `test_diff_stream_matches_daemon_state`. Use the shape of `test_dashboard_subscription_stream` (`tests/integration_tests.rs:963`) for socket setup.
2. Connect a client that sends `{"method":"daemon.subscribe_diff"}`.
3. Make a series of changes: register 3 agents, update one status, remove one agent (use whatever deregistration path the daemon exposes; if none exists, skip removal and note it in the report).
4. The client keeps a `HashMap<id, value>`. For each received line, if `snapshot` is true, clear the map; then apply `upsert` and `remove`.
5. After the last change, assert the client map equals the daemon's agent list (compare as sorted JSON).
6. Use a timeout on every read, so a regression fails instead of hanging.

**Constraints (do not touch)**: existing integration tests; `src/`.

**Grep anchors**:
- `async fn test_dashboard_subscription_stream` (`tests/integration_tests.rs:963`)
- `fn recv_command` (`tests/integration_tests.rs:13`) for the timeout pattern.

**Acceptance**:
- `cargo test --test integration_tests test_diff_stream_matches_daemon_state` passes.
- `cargo test --test integration_tests` passes (all 11 tests).

**Targeted test**: `cargo test --test integration_tests test_diff_stream_matches_daemon_state`

---

### Task 20.3.3: G9 Docs

**ID**: `20.3.3`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `README.md` (the Socket Contract section only, starting at line ~50)
- must edit: `docs/decisions/0008-push-status-subscription-for-hud.md` (the Consequences section only)

**Depends-on**: `20.2.1`

**Spec prose**:
1. In README Socket Contract, add one bullet: `daemon.subscribe` streams the full agent list as one JSON array per line. `daemon.subscribe_diff` streams `{"snapshot","upsert","remove"}` objects; the first line is a snapshot. A subscriber whose channel fills receives a snapshot on its next line.
2. In ADR 0008 Consequences, after the "Slow subscribers" bullet, add one bullet: `Amended 2026-11-08 (Orchestration 20): diff subscribers (daemon.subscribe_diff) recover from a dropped line with a full snapshot instead of waiting for the next change. See ADR 0013.`
3. Change nothing else in either file.

**Constraints (do not touch)**: other README sections; other ADRs; `src/`.

**Grep anchors**:
- `### Socket Contract` (`README.md:50`)
- `Slow subscribers` (`docs/decisions/0008-push-status-subscription-for-hud.md`)

**Acceptance**:
- `grep -c 'daemon.subscribe_diff' README.md` returns at least 1.
- `grep -c 'Amended 2026-11-08' docs/decisions/0008-push-status-subscription-for-hud.md` returns 1.

**Targeted test**: the two greps above.

---

## Wave Gates

- **Pre-wave 1 (baseline)**: the last serial `make all` passed at `e8c1610` (35 unit, 10 integration, Pi 3/3, hook test). That run was after G28 and no Rust code has changed since, so it stands as the baseline. Record it in STATE_20 with a note.
- **Wave 1 end gate**: `make all` serially. Expect 35 unit + 4 config path tests = 39 unit, 10 integration. The ignored measurement test does not run. Record counts in STATE_20. Then the orchestrator reviews the `20.1.1` numbers (D7) and records the decision.
- **Wave 2 end gate**: `make all` serially. Expect 39 unit + 0 new tests from `20.2.1` (existing tests still pass).
- **Wave 3 end gate**: `make all` serially, then `make test` runs Rust, Pi and hook suites. Expect 39 + 2 (`20.3.1`) = 41 unit, 11 integration (10 + `20.3.2`).
- **Post-wave (adr-distillation)**: create ADR 0013 for the diff subscription protocol (opt-in method, bare-array retained, snapshot-after-drop, D1-D3). Update the index in `docs/decisions/README.md`.
- **Post-wave (G10 close)**: GOALS.md moves G10 to Completed. Post-wave (G9 close): GOALS.md moves G9 to Completed.
