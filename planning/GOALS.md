# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-08
Source State: STATE_21.md (Orchestration 21 complete; `make all` green, zero warnings, at `dc2c4e5`; ADRs 0012–0014 recorded)

## Circled Goal (Active)

- None open. No remaining goal has downstream dependents, so there is no root to circle. The highest-leverage item is **[G38]**, and it needs your approval (remote write).

## Sequenced Goals (Dependency DAG)

- None open. G29–G37 are complete (see Completed).

## Independent Backlog

- **[G39]** The `status --watch --diff` fold is covered by an end-to-end test, not only by parse tests.
  - Why: the fold in `run_status_watch` (`src/cli.rs`) is the only diff consumer in this repo, and it has no behavioural test.
  - Exit criteria: a test feeds a snapshot, an upsert and a remove line through the fold and asserts the rendered agent list. Refactor the fold into a pure helper only if needed to test it.
  - First action: read `run_status_watch` and decide between a pure helper and a test that drives the socket.
  - Depends on: none.
- **[G40]** The 10 event-ordering sleeps left in `tests/integration_tests.rs` (G37, D8) are replaced with explicit daemon acknowledgements.
  - Why: those sleeps are the remaining flake risk in the integration suite.
  - Exit criteria: no fixed sleep orders daemon events in `tests/integration_tests.rs`; `cargo test --test integration_tests` passes 5 runs in a row.
  - First action: decide whether the daemon needs an ack message, or whether each sleep can poll an observable state (the status query on a second connection, as in `tests/slow_consumer.rs`).
  - Depends on: none. May need a `src/` change, so it needs its own plan.
- **[G41]** Delete the three merged local branches `docs/adr-distillation`, `docs/agents-md-structure` and `docs/task-4.1-checkoff`.
  - Why: all three are fully merged into `main` (verified 2026-11-08). They are leftover clutter.
  - Exit criteria: `git branch --list` shows only `main`.
  - First action: `git branch -d` each one; `-d` refuses if any is unmerged.
  - Depends on: none.

## Deferred

- **[G42]** `AgentWindow` derives `PartialEq` and the diff broadcast compares with `==` (revisit of G32).
  - Trigger: the release N=50 K=10 measurement passes 1 ms, or the fleet grows past about 50 agents with 10 subscribers. Current: 621 µs at N=50 K=10, 2.4 ms at N=200 K=10 (`STATE_21.md`).
- **[G43]** Per-client rate limit on undecodable or malformed IPC lines (ADR 0014 negative consequence).
  - Trigger: a real client is observed flooding bad lines, or a log shows a noisy skip.

## Blocked / Pruned

- **[G38]** Publish the local commits to `origin/main`. `origin/main` is 59 commits behind `main`.
  - Status: Blocked / User Approval
  - Reason: a remote write needs your explicit go-ahead (CLAUDE.md: no remote write operations). Nothing in the repo blocks it. Say the word and I will push.
- **[G25]** The external Antigravity sidecar ships against the documented socket contract.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. Its internal prerequisites, G22 and G24, are complete.
- **[G15]** An Antigravity sidecar integration exists in this repo. `integrations/antigravity/` has a README only.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. Its internal prerequisites are complete, so only the external work remains.
- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.

## Completed

- **[G29]** `daemon.subscribe` bytes are pinned by a golden test (`tests/subscribe_golden.rs`, `cab1daf`, 21.1.5). The test accepts the two byte-exact orderings because the daemon's `HashMap` order is not stable. Orchestration 21.
- **[G30]** A stalled diff subscriber recovers with a snapshot at socket level (`tests/slow_consumer.rs`, `b78343b`, 21.1.6). Orchestration 21.
- **[G31]** The diff path is measured on the G9 grid (`measure_broadcast_cost_diff`, `3c2475f`, 21.1.1). Release N=50 K=10 = 621 µs, below the 1 ms threshold. Full grid in `STATE_21.md`. Orchestration 21.
- **[G32]** Not included. G31 measured 621 µs at N=50 K=10, under the 1 ms threshold, so `to_value` is kept and `PartialEq` is not derived. Recorded in ADR 0013. Orchestration 21.
- **[G33]** `swayve status --watch --diff` opts into the diff stream; `--diff` requires `--watch` (`459c583`, 21.3.2). Default stays the full list. Orchestration 21.
- **[G34]** Foreign-owner checks are tested by injecting the owner uid: `ensure_private_dir_for` (`ab74cba`, 21.1.2) and `prepare_socket_path` (`235adf3`, 21.2.1). ADR 0012 amended. Orchestration 21.
- **[G35]** A non-UTF-8 IPC line is logged and skipped; the client stays connected (`235adf3`, 21.2.1). Test `test_non_utf8_ipc_line_is_skipped`. Orchestration 21.
- **[G36]** A relative or empty `XDG_CONFIG_HOME` logs a one-line warning before falling back (`7d003c7`, 21.1.3). Orchestration 21.
- **[G37]** Startup sleeps in `tests/integration_tests.rs` are replaced with `wait_for_socket` (`82f8355`, 21.1.4): 11 replaced, 10 event-ordering sleeps kept. Orchestration 21.
- **[G9]** Diff subscriptions for opt-in clients. `daemon.subscribe_diff` streams snapshot and delta lines; a subscriber whose channel fills gets a snapshot on its next line. Measured baseline recorded in `STATE_20.md` (20.1.1, release N=50 K=10 = 27 µs). Routing `8d6ec67`; integration test `b016cbc` replays the stream against daemon state; docs `53ae8c9`. `daemon.subscribe` keeps its bare-array format; the existing subscription tests pass. Orchestration 20. ADR 0013 pending the post-wave distillation.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path when absolute; `$HOME/.config` is the fallback. `config_path_from` in `src/config.rs` (`c6c483a`, 20.1.2); README Configuration note and ADR 0010 amendment (`f4e9e33`, 20.1.3). Orchestration 20.
- **[G28]** No README text in this repo describes the daemon IPC as JSON-RPC, except the deliberate correction at `README.md:54`. The `src/types.rs` link description now names newline-delimited JSON framing (one-line edit at `README.md:178`; `make all` passes).
- **[G22]** The daemon's runtime socket path is enforced by tests and documented as the external client contract. Tests `cc3c7b3` (ensure_private_dir), `30f576e` (stale socket, malformed line). README Socket Contract `40c2777`. Recorded as ADR 0012 (`b6d76b3`). Orchestration 19.
- **[G23]** `planning/GOALS.md` and `planning/STATE_18.md` agree on G17 status. Done in the 2026-11-08 rewrite.
- **[G24]** The Antigravity integration documents the runtime socket default (`161c0de`, 19.1.3).
- **[G26]** `handle_client` drops no client on a malformed IPC line. Test `test_malformed_ipc_line_keeps_client` (`30f576e`, 19.1.2). Non-UTF-8 lines still close the client; that is the accepted limit in `STATE_19.md`.
- **[G27]** Orchestrations 15–17 planning files moved to `planning/archive/` (`40f8af6`, 19.2.1).
- **[G17]** The daemon socket and agent logs live in `$XDG_RUNTIME_DIR/swayve/` (0700). Refuses to start without `XDG_RUNTIME_DIR`. Stale-socket removal only deletes a socket owned by the current user. Commit `cc35bfd`. Live check passed on 2026-11-08 (`planning/STATE_18.md`).
- **[G18]** README CLI reference lists `init-shell`, `status [--json] [--watch]`, and `focus --cycle` (18.2.1).
- **[G19]** `cargo clippy --all-targets` is clean (18.2.2).
- **[G20]** `make test` runs Rust, Pi and hook tests (18.2.3).
- **[G21]** Unit tests pin the Sway command strings and `AgentStatus`/`AgentId` JSON and marks (18.2.4).
- **[G14]** Planning waves 1–14 archived into `planning/archive/` with `git mv` (17.1.3).
- **[G13]** Daemon-down fallback test: `test_flat_fallback_cycles_agent_windows_in_tree_order` (17.1.1). README and ADR 0003 say the fallback cycles agent windows only (17.1.2).
- **[G16]** Stale docs and CLI trap fixed. Commits `398fa92`, `66ef55c`.
- **[G12]** Malformed config reported live. Checked 2026-11-07 (`planning/archive/STATE_16.md`).
- **[G1]** Daemon focus follows Sway's actual focus. Commit `436b190`. Live check passed on 2026-11-07.
- **[G2]** Focus-state decision recorded as ADR 0011. Commit `874d7db`.
- **[G3]** Malformed config reports an error and falls back to defaults. Commit `5788c3e`.
- **[G4]** Children-mark rehome covered with several surviving siblings. Commit `3422164`. Live check passed on 2026-11-07.
- **[G5]** Subscriber drop-on-full policy recorded in ADR 0008 and covered by a test. Commit `55314d4`.
- **[G6]** Daemon-down fallback decided: the CLI keeps the flat fallback. Commit `d313105`.
- **[G7]** Planning docs reconciled. Commit `ccf52a7`.
- **[G8]** Test layout and CI convention recorded in `AGENTS.md`. Commit `d313105`.
