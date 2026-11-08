# Orchestration 28 State: agent ownership, subscription lifetime, launch argv

Plan: `planning/EXECUTION_PLAN_28.md` (draft, pending approval)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `17c72ce` (main).
- Pre-wave-1 step: run serial `make all`, record exit code and warning count here. Last known green: Orchestration 27 (`1f79de0`): unit 74 passed / 2 ignored; integration 11; Pi `npm test` 6/6; zero warnings.
- Pre-wave-1 step: `cargo clippy --all-targets`: zero warnings.
- Recorded 2026-11-08 at `17c72ce`: serial `make all` exit 0, zero warnings; unit 74 passed / 2 ignored; integration 11; subscribe_golden, slow_consumer and test-hooks green.
- Working tree before wave 1: `planning/GOALS.md` modified (G79–G81 added); `planning/ARCHITECTURE_REVIEW_2026-11-08_FOLLOWUP.md` untracked. Commit these planning files before wave 1 so the task diffs stay clean.

## Wave Board

### Wave 1: re-registration keeps container binding (G79)
- [x] **Task 28.1.1**: `register_agent` keeps `container_id`, `con_to_agent` and `status` on re-registration

### Wave 2: rebinding releases old container (G79)
- [x] **Task 28.1.2**: `register_agent_window` releases the old mapping; `on_window_destroyed` removes only the current owner

### Wave 3: CLI and Pi derive the same `app_id` (G79)
- [x] **Task 28.1.3**: Pi client `app_id` matches the CLI format; Rust format pin

### Wave 4: binding and creation publish to subscribers (G79)
- [x] **Task 28.1.4**: `run_daemon_loops` publishes after a window bind

### Wave 5: unchanged diff subscriber with a closed channel is removed (G81)
- [x] **Task 28.2.1**: `broadcast_diff` detects a closed channel before the unchanged-state return

### Wave 6: full and diff writers share one lifetime owner (G81)
- [x] **Task 28.2.2**: one writer function for both subscription modes

### Wave 7: monitor argv through one helper (G80)
- [x] **Task 28.3.1**: `spawn_monitor_terminal` builds argv from a list; spaced log path stays one argument

### Wave 8: argv-capture test replaces substring assertions (G80)
- [x] **Task 28.3.2**: mock terminal writes one argument per line; exact argv assertions

### Wave 9: panel launch passes `status --watch` as argv (G80)
- [x] **Task 28.3.3**: panel argv uses separate entries

### Wave gates
- [x] After wave 1: `make all` serial, exit 0, zero warnings.
- [x] After wave 4: `make all` serial, exit 0, zero warnings. G79 exit criteria checked.
- [x] After wave 6: `make all` serial, exit 0, zero warnings. G81 exit criteria checked.
- [x] After wave 9: `make all` serial, exit 0, zero warnings. G80 exit criteria checked.

### Orchestrator steps
- [x] `git status` clean at each wave end (planning files excepted). Nothing pushed.

## Deviations
- None yet.

## Task Reports

### 28.1.1 (done)
- Status: done. Files: `src/daemon.rs` `register_agent` (~152-165) carries `container_id` and `status` from the existing record; new test `test_reregister_keeps_container_binding` at the test module bottom.
- Tests: `cargo test --lib test_reregister` 1 passed (red first: container_id None). `make all` serial exit 0, zero warnings, unit 75 passed / 2 ignored.
- Deviations: none. Q1 answered: keep lifecycle hook call unchanged.
- Gotcha: a re-registration with a new pid leaves the old pid in `pid_to_agent` (kept per spec).

### 28.1.2 (done)
- Status: done. Files: `src/daemon.rs` `register_agent_window` (~652) drops the old `con_to_agent` entry; `on_window_destroyed` (~249) returns early for `Some(other)` owner; new test `test_rebind_releases_old_container`.
- Tests: `cargo test --lib test_rebind` 1 passed; `test_window_destroyed` 4 passed. `make all` serial exit 0, zero warnings, unit 76 passed / 2 ignored.
- Deviations: red run skipped (test written and run only after the fix). Q1 answered: `container_id == None` counts as owned.

### 28.1.3 (done)
- Status: done. Direction changed from the spec's hyphen-to-dot transform (lossy for hyphenated agent names) to lossless: CLI `app_id = "agent.{agent_id}"` (`src/cli.rs:547`). Pi code unchanged except a pointer comment (`swayve-client.ts:70`).
- Tests: Pi `npm test` 8/8 (two new pins passed first run). `cargo test --lib test_spawn_plan` 8 passed after a red step on the old assertion at line 771. New `test_spawn_plan_app_id_pin` (`agent.claude-w2-7`). `make check` exit 0, zero warnings.
- Deviations: `test_spawn_plan_substitutes_app_id_and_title` (`src/cli.rs:795`) also updated to the new format (outside the plan, required).

### 28.1.4 (done) and wave 4 gate
- Status: done. `src/daemon.rs` `run_daemon_loops` `WindowCreated` arm calls `broadcast_status()` when actions are non-empty (~588). New test `subscriber_sees_bound_container` in `tests/integration_tests.rs`.
- Red: test timed out waiting for the bind broadcast before the fix. Green after the fix.
- Gate: `make all` serial exit 0, zero warnings; unit 77 passed / 2 ignored; integration 12 passed; subscribe_golden and slow_consumer green.
- G79 exit criteria (a) registration→binding→re-registration (28.1.1), (b) rebinding→old-window close (28.1.2), (c) binding→subscriber observation (28.1.4) are covered.

### 28.2.1 (done)
- Status: done. `src/daemon.rs` `broadcast_diff` (~108) early return is `!sub.tx.is_closed()`. New test `test_diff_subscriber_removed_when_unchanged`.
- Red: subscriber count 1, expected 0. Green: `test_diff_subscriber` 3 passed; `slow_consumer` 1 passed; `make check` exit 0, zero warnings.
- Deviations: none.

### 28.2.2 (done) and wave 6 gate
- Status: done. `src/daemon.rs` `subscriber_writer(rx, writer)` (~435) is the one writer for both modes; `handle_client` merges the two subscribe arms (~483). Tests `test_writer_ends_on_socket_close_full` and `test_writer_ends_on_socket_close_diff`.
- Design change (approved): no oneshot and no half-close signal, so shell clients that half-close keep their stream. Accepted limit: an idle writer notices a closed socket only on its next write.
- Red: compile failure (missing `subscriber_writer`). Green: writer tests pass.
- Gate: `make all` serial exit 0, zero warnings; unit 80 passed / 2 ignored; integration 12 passed; subscribe_golden, slow_consumer green.
- G81 exit criteria: diff-subscriber closed-channel removal (28.2.1), writer-cleanup tests (28.2.2), overflow and slow-consumer tests and both wire formats unchanged.

### 28.3.1 (done)
- Status: done. `src/cli.rs` `expand_terminal_args` (~529) is the shared pure argv helper (also reusable by 28.3.3). `src/daemon.rs` `monitor_argv` (~771) feeds `spawn_monitor_terminal`; shlex loop removed there. `spawn_plan` unchanged.
- Red: compile failure on missing helpers. Green: `test_monitor_argv` 2 passed; `test_spawn_plan` 8 passed (spec said 7; the filter matches 8).
- Gate: `make check` exit 0, zero warnings. Full `make all` runs at wave 9.
- Deviations: none.

### 28.3.2 (done)
- Status: done. `tests/integration_tests.rs` mock now `printf '%s\n' "$@"`; monitor assertions are one exact `assert_eq!` on argv lines (~986-1003). Spawn-hook assertions unchanged.
- Red: after the mock change alone, the old `contains` assertion failed. Green: `monitor` 1 passed; `make check` exit 0, zero warnings.
- Integration count: 12 passed (plan expected 11; the wave-4 test already added one).
- Discovered: the worker did not write `planning/tasks/28.3.2.todo.md`; the plan text is in the orchestrator's record only.

### 28.3.3 (done), wave 9 gate, closing gate
- Status: done. `src/cli.rs` `panel_terminal_args` (~533) feeds the orchestrator panel launch (~349). Before: `["-e","swayve status --watch"]`. After: `["-e","swayve","status","--watch",...]`.
- Red: compile failure on missing helper. Green: `test_panel_argv` and `test_status_watch_cmd_parsing` pass.
- Gate: serial `make all` exit 0, zero warnings; unit 83 passed / 2 ignored; integration 12 passed; Pi `npm test` 8/8.
- Closing: GOALS.md moves G79, G81, G80 to Completed with commit hashes.
- Discovered (open, not in this orchestration): `spawn_plan` still uses its own shlex loop instead of `expand_terminal_args`.

