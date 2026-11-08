# Orchestration 20 State: Public-Release Readiness: Diff Subscriptions (G9) and XDG Config Path (G10)

Plan: `planning/EXECUTION_PLAN_20.md`
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Orchestration 19 complete. HEAD `e8c1610` (G28 README fix). Working tree clean.
- Last serial `make all` at `e8c1610`: 35 unit / 10 integration / Pi 3/3 / hook test passed. No Rust code has changed since that run (the G28 change touched README only), so this stands as the baseline.
- Pre-wave 1 counts: unit 35 / integration 10 / result ok.

## Wave Board

### Wave 1: Measurement and Config Path
- [ ] **Task 20.1.1**: G9 broadcast-cost measurement (ignored test, `src/daemon.rs` tests)
- [ ] **Task 20.1.2**: G10 `config_path_from` and tests (`src/config.rs`)
- [ ] **Task 20.1.3**: G10 docs (README Configuration note, ADR 0010 amendment)

### Wave 1 end gate
- [x] `make all` passes serially. Expect 39 unit (35 + 4) and 10 integration. Actual: 39 unit (1 ignored) / 10 integration / Pi 3/3 / hook test passed.
- [ ] Orchestrator reviews 20.1.1 numbers and records the D7 decision here: D7 condition met (release N=50 K=10 = 27.3 us; debug N=50 K=10 = 214.9 us; both under 1 ms). Per D7, Wave 2 waits for the user's go-ahead. Decision: PROCEED (user, 2026-11-08: diff stream ships for public release).

### Wave 2: Diff Core (blocked until D7 recorded)
- [ ] **Task 20.2.1**: `SubscribeDiff` message and diff broadcast core (`src/types.rs`, `src/daemon.rs`)

### Wave 2 end gate
- [ ] `make all` passes serially. Expect 39 unit. Actual: ___

### Wave 3: Routing, Integration Test, Docs
- [ ] **Task 20.3.1**: `handle_client` routing and snapshot-after-drop unit tests
- [ ] **Task 20.3.2**: Integration test for diff stream (depends on 20.3.1)
- [ ] **Task 20.3.3**: G9 docs (README Socket Contract, ADR 0008 amendment)

### Wave 3 end gate
- [x] `make all` passes serially; `make test` runs Rust, Pi and hook suites. Expect 41 unit, 11 integration. Actual: 41 unit (1 ignored) / 11 integration / Pi 3/3 / hook test passed. Orchestrator note: the 20.3.2 worker's fixed 300 ms sleep was replaced by sending the Sway window events once a3 appears in the diff stream; the targeted test then passed 5 of 5 runs.

### Post-wave
- [ ] adr-distillation: ADR 0013 (diff subscription protocol), index updated.
- [ ] GOALS.md: G10 and G9 moved to Completed.

### Deferred (manual, not gated)
- None.

## Known limits (recorded at plan time)
- The external HUD (outside this repo) is not updated. The diff stream is documented for it, but no in-repo client consumes it.
- `status --watch` stays on the full stream (D1).

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

## 20.1.1
Status: done.
Files: src/daemon.rs 931-964 (new ignored test `measure_broadcast_cost`, end of mod tests); planning/tasks/20.1.1.todo.md (checked off).
Commands: `cargo test --lib measure_broadcast_cost -- --ignored --nocapture` (1 passed); `cargo test --release --lib measure_broadcast_cost -- --ignored --nocapture` (1 passed); `cargo test --lib` (35 passed, 1 ignored).
Debug:
N=5 K=1 mean_us=23.2 payload_bytes=641
N=5 K=10 mean_us=24.0 payload_bytes=641
N=20 K=1 mean_us=87.2 payload_bytes=2591
N=20 K=10 mean_us=89.0 payload_bytes=2591
N=50 K=1 mean_us=186.0 payload_bytes=6521
N=50 K=10 mean_us=214.9 payload_bytes=6521
N=200 K=1 mean_us=754.8 payload_bytes=26471
N=200 K=10 mean_us=871.0 payload_bytes=26471
Release:
N=5 K=1 mean_us=1.8 payload_bytes=641
N=5 K=10 mean_us=6.5 payload_bytes=641
N=20 K=1 mean_us=1.6 payload_bytes=2591
N=20 K=10 mean_us=14.8 payload_bytes=2591
N=50 K=1 mean_us=3.6 payload_bytes=6521
N=50 K=10 mean_us=27.3 payload_bytes=6521
N=200 K=1 mean_us=21.0 payload_bytes=26471
N=200 K=10 mean_us=140.2 payload_bytes=26471
Deviations: none (also ran planned `cargo test --lib`). Gotchas: debug is serialization-dominated; release cost scales with K (string clone per subscriber).

## 20.1.3
- Status: done. Both acceptance greps pass (README `XDG_CONFIG_HOME` count 1; ADR `Amended 2026-11-08` count 1).
- Files changed:
  - `README.md` lines 153-155: added the XDG paragraph under `## ⚙️ Configuration` (heading unchanged per Q2).
  - `docs/decisions/0010-toml-config-in-home-config-dir.md` line 34: added bullet `* Amended 2026-11-08 (Orchestration 20): ...` directly after the "XDG_CONFIG_HOME is ignored" bullet (per Q1).
  - `planning/tasks/20.1.3.todo.md`: checkboxes flipped.
- Commands: `grep -c 'XDG_CONFIG_HOME' README.md` -> 1; `grep -c 'Amended 2026-11-08' docs/decisions/0010-toml-config-in-home-config-dir.md` -> 1.
- Deviations: TODO item "git diff --stat / git diff" not run, because the ground rules say to run only the two greps. Diff was checked by reading the edited ranges instead.
- Gotchas: the README paragraph sits before the "Customize agent states..." line, so the TOML example stays attached to its intro sentence. Nothing committed or pushed.

## 20.1.2
- Status: done
- Files: src/config.rs (new `config_path_from` + rewritten `load_config` head ~L108-127; 4 tests appended at end of `mod tests`); planning/tasks/20.1.2.todo.md
- Test: `cargo test --lib test_config_path` -> 4 passed, 0 failed
- Deviations: none (full `cargo test --lib` not run, per orchestrator; left for the wave gate)
- Gotchas: HOME unset with absolute XDG set now loads from XDG. `parse_config_file` untouched. The home_only test also asserts (None, None) -> None.

## 20.2.1
- Status: done
- Files: src/types.rs L65-68 (`SubscribeDiff`); src/daemon.rs: `SubMode`/`Subscriber` + `subscribers` field (~L13-28), `push_full/diff_subscriber` + `broadcast_status` + new `broadcast_diff` (~L45-130), `DaemonEvent::SubscribeDiff` variant + handler, 4 test sites -> `push_full_subscriber`; planning/tasks/20.2.1.todo.md
- Commands: `cargo check` ok; `cargo test --lib broadcast` 2 passed 1 ignored; `cargo test --lib` 39 passed 0 failed 1 ignored; `grep -n SubscribeDiff src/types.rs` -> L68
- Deviations: added a no-op `AgentIPCMessage::SubscribeDiff => {}` arm in the daemon event-loop match (~L421) because the match is exhaustive; `handle_client` untouched. No new tests (per orchestrator; belong to 20.3.1).
- Gotchas: `broadcast_status` calls `broadcast_diff` at its end, so all 4 call sites cover diff subscribers. Diff compare uses `serde_json::to_value` (no PartialEq on AgentWindow).

## 20.3.3
- Status: done. Both acceptance greps pass (`daemon.subscribe_diff` in README count 1; `Amended 2026-11-08` in ADR 0008 count 1).
- Files changed:
  - `README.md` line 56: added the `daemon.subscribe` / `daemon.subscribe_diff` bullet as the last bullet of `### Socket Contract` (after the `SWAYVE_SOCKET` bullet, before `---`).
  - `docs/decisions/0008-push-status-subscription-for-hud.md` line 39: added the `* Amended 2026-11-08 (Orchestration 20): ...` bullet after the "Slow subscribers" bullet (line 38).
  - `planning/tasks/20.3.3.todo.md`: checkboxes flipped.
- Commands: `grep -c 'daemon.subscribe_diff' README.md` -> 1; `grep -c 'Amended 2026-11-08' docs/decisions/0008-push-status-subscription-for-hud.md` -> 1.
- Deviations: none.
- Gotchas: the ADR 0013 reference dangles until the post-wave adr-distillation step (expected). The "Slow subscribers" bullet still describes the full-list behavior, which now applies only to `daemon.subscribe`; left unedited per spec. Nothing committed or pushed.

## 20.3.1
- Status: done
- Files: src/daemon.rs: `handle_client` new `SubscribeDiff` arm (L354-371), event-loop no-op comment updated (L438), 2 new tests (L1035-1073); planning/tasks/20.3.1.todo.md
- Commands: `cargo test --lib test_diff_subscriber` -> 2 passed, 0 failed. Full `cargo test --lib` not run (orchestrator runs at gate).
- Deviations: none.
- Gotchas: `push_diff_subscriber` sends nothing; tests call `broadcast_status()` for the first snapshot. Drop test: second broadcast's diff is dropped (channel full), the first snapshot is still queued; recovery snapshot is sent on the next broadcast. No bug seen in `broadcast_diff`. Nothing committed.

## 20.3.2
- Status: done
- Files: tests/integration_tests.rs L1086-1200 (new test appended at end); planning/tasks/20.3.2.todo.md
- Command: `cargo test --test integration_tests test_diff_stream_matches_daemon_state` -> 1 passed, 0 failed
- Removal exercised: yes, via DaemonEvent::Sway WindowCreated (agent.pi.diff-a3) then WindowDestroyed (no deregister IPC exists)
- Final state read: a fresh `daemon.subscribe` (full list) stream's first line, compared to the client map as id-sorted JSON; not the diff stream's own snapshot
- Deviations: none
- Gotchas: first run failed (diff read timeout) because the Sway WindowCreated event raced the socket Register; fixed with a 300ms sleep after registering. Every read uses timeout(2s); diff replay loop bounded to 20 lines. Nothing committed.
