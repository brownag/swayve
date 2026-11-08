# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-08 (Orchestration 22 complete; G45 and G48 done; G46 circled; architecture review triaged into G54–G58)
Source State: STATE_22.md (Orchestration 22 complete; `make all` green, zero warnings, at `8a7f002`; ADRs 0012–0015 recorded). `origin/main` = `96f18af` = local `main` (Orchestration 22 pushed by the user).

## Circled Goal (Active)

- **[G46]** A socket-level test shows that an over-limit IPC line is dropped and logged, the client stays connected, and the next valid line is still processed.
  - Why now: G45 is complete, so the test can use a unique socket path. The configured limit path (G48) lets the test set a small burst.
  - Exit criteria: a test in `tests/` sends more than 200 lines on one connection, then a valid `daemon.focus`, and asserts a `FocusWindow` command. `cargo test` passes.
  - First action: copy the connection setup from `tests/slow_consumer.rs`, with `common::unique_path`.

## Sequenced Goals (Dependency DAG)

- **[G47]** The README Socket Contract documents the per-client limit: 100 lines/s sustained, burst 200, over-limit lines dropped and logged, and the client kept connected.
  - Depends on: G46 (the docs describe behavior a test has verified).
  - Exit criteria: the README section matches `src/ratelimit.rs`. ADR 0015 links to it.
  - First action: add three bullets under Socket Contract in `README.md`.
## Independent Backlog

- **[G49]** The two focus-ordering sleeps at `tests/integration_tests.rs` 633 and 638 are replaced with a barrier event.
  - Why: the test sends a `WindowFocused` event and then a `daemon.focus` over the socket. Both reach the daemon through the same channel, but from different tasks. The sleep only guesses the order.
  - Approach: add an internal `DaemonEvent::Sync(oneshot::Sender<()>)`. The daemon replies when it reaches the event. The test sends `Sync` after the focus event and awaits it. No wire change, no IPC ack. The enum is internal (`src/daemon.rs`).
  - Rejected alternative: a daemon ack on the socket. Clients are fire-and-forget (ADR 0009, ADR 0014), so an ack would change the protocol. That is deferred as G52.
  - Exit criteria: no `sleep` orders events in `test_focus_follows_sway_focus`; `subscribe_golden` unchanged; integration suite passes 5 runs.
  - Depends on: none.
- **[G50]** The 300 ms sleep at `tests/integration_tests.rs` 948 (spawned hooks and auto-monitor) is replaced with a poll of the flag files the test already asserts on.
  - Why: this sleep was counted as unobservable in D2. It is observable. The test reads `spawn_flag_path` and `monitor_flag_path` after the sleep, so it can poll them with a timeout.
  - Exit criteria: no fixed sleep in `test_hooks_and_monitoring`; the same assertions run unchanged; suite passes 5 runs.
  - Depends on: none.
- **[G51]** CI runs `make all` on every push.
  - Why: AGENTS.md says there is no CI. The workflow file can be written locally. It runs only after a push, which needs your go-ahead.
  - Exit criteria: a `.github/workflows` file that runs `make all` on Linux with Sway-free tests. The workflow is valid YAML, and the workflow file is committed locally. No push.
  - Depends on: none. Needs `github-actions-ci` skill conventions.

- **[G54]** Window lifecycle rules live in `DaemonState`, behind two methods: `on_window_created(con_id, app_id, pid)` and `on_window_destroyed(con_id)`. Each returns `Vec<LayoutAction>` and `Vec<MonitorRequest>`. The run loop only executes spawns.
  - Source: [architecture review](ARCHITECTURE_REVIEW_2026-11-08.md), candidate 1. Confirmed in code: the auto-monitor rule appears in `register_agent` (`src/daemon.rs` ~162, gated on `is_subagent`) and in the WindowCreated arm (~500, gated on process-tree auditing also succeeding). The gates differ, so the refactor must make that difference explicit, not merge it silently.
  - Exit criteria: one method owns the auto-monitor decision; `test_auto_monitor_subagent_registration` tests it without a channel; `LayoutAction` shape unchanged (ADR 0005); `make all` passes.
  - Depends on: none. Plan-wave before any change.
- **[G55]** The process-tree depth-limit test proves `max_depth` is respected.
  - Source: [architecture review](ARCHITECTURE_REVIEW_2026-11-08.md), candidate 2, confirmed. `test_early_termination_at_depth_limit` (`src/daemon.rs` ~819) starts at PID 1 and asserts `None`, which passes even if `max_depth` is ignored.
  - Approach: pass the parent lookup as a closure to `find_registered_parent_ancestor`. The `/proc` read stays as the production adapter. The test uses a map-backed chain longer than the limit, and asserts the walk stops at the limit.
  - Exit criteria: the test fails if the depth check is removed; `make all` passes.
  - Depends on: none.
- **[G56]** The diff frame has one shared type, `DiffFrame { snapshot, upsert, remove }`, used by the daemon to build frames and by `fold_diff_line` to read them.
  - Source: [architecture review](ARCHITECTURE_REVIEW_2026-11-08.md), candidate 3. Confirmed: the daemon builds the JSON with `serde_json::json!` (`src/daemon.rs` ~112) and the CLI indexes raw `Value`s.
  - Constraint: the wire bytes must not change. Add a golden test for `daemon.subscribe_diff` lines first. The existing golden pins only `daemon.subscribe`.
  - Depends on: a `subscribe_diff` golden test (part of this goal).
- **[G57]** The `spawn` command's plan (prompt escaping, `bash -c` wrapping, ID and title derivation) is a pure function, `spawn_plan(args, now_ms)`, and the command executes it.
  - Source: [architecture review](ARCHITECTURE_REVIEW_2026-11-08.md), candidate 5. Not fully verified: the review read only the first ~40 lines of the `Commands::Spawn` arm (`src/cli.rs` ~222). Read the whole arm before planning.
  - Exit criteria: the plan function has unit tests for escaping and ID shape; `make all` passes.
  - Depends on: none.

## Deferred

- **[G58]** The focus cycle owns its cursor: `FocusCycle { last }` with `.next(&agents)` and `.observe(con_id)`, replacing the `last_focused` local in `run_daemon_loops` (`src/daemon.rs` ~413).
  - Source: [architecture review](ARCHITECTURE_REVIEW_2026-11-08.md), candidate 4. The reviewer judged the gain small: the cycle rule is already one tested function (`next_focus_target`). Do not merge the daemon and CLI focus paths (ADR 0003 keeps the flat fallback).
  - Trigger: the next change to the focus cycle rules, or G54 landing (it would touch the same loop).

- **[G52]** The daemon sends a delivery ack for each IPC message.
  - Trigger: a client needs to know its line was applied (for example, a CLI that must wait for registration before it spawns a window), or a wire-format change is approved in an ADR review.
  - Why deferred: it changes the fire-and-forget contract (ADR 0009, 0014). No current client needs it. G49 covers the test-ordering need without it.

## Blocked / Pruned

- **[G25]** The external Antigravity sidecar ships against the documented socket contract.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. Its internal prerequisites, G22 and G24, are complete.
  - Agent prompt: `planning/external/antigravity-sidecar.prompt.md`. It covers `SWAYVE_PARENT_ID`, which the daemon does not set today.
- **[G15]** An Antigravity sidecar integration exists in this repo. `integrations/antigravity/` has a README only.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. Its internal prerequisites are complete, so only the external work remains. Same prompt as G25.
- **[G53]** The external HUD consumes `daemon.subscribe_diff` and folds the stream.
  - Status: Blocked / External Watch
  - Reason: the HUD is built and owned outside this repo. The daemon side is complete (G9).
  - Agent prompt: `planning/external/hud-diff-consumer.prompt.md`.
- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.

## Completed

- **[G48]** Per-client IPC rate limit is configurable: `[daemon] ipc_lines_per_sec` (default 100) and `ipc_burst` (default 200). Invalid values fall back with a `[config]` warning. ADR 0016 supersedes ADR 0015 (`5664429`). The socket-level check moved to G46. Unit tests cover config defaults, sanitizing, and a configured burst reaching `handle_client`.
- **[G45]** Test socket and flag paths are unique per process, under the temp dir, through `tests/common::unique_path` (`0a6d0e6`). No `/tmp` literal remains in `tests/` or `src/`. Two integration runs in parallel pass.
- **[G44]** Backlog close-out: G39–G43 are complete, `make all` green with zero warnings. Orchestration 22 (`EXECUTION_PLAN_22.md`, `STATE_22.md`). Pushed to `origin/main` (`fb34936..96f18af`) by the user.
- **[G43]** Per-client IPC rate limit, wired on: a token bucket per connection (100 lines/s sustained, burst 200); over-limit lines are dropped and logged, and the client stays connected (`src/ratelimit.rs`, `5d7ef87`, 22.2.1). ADR 0015 (`8a7f002`). Orchestration 22.
- **[G42]** `AgentWindow` derives `PartialEq`; the diff broadcast compares with `==` (`446d93c`, 22.1.1). Release N=1000 K=10 = 2339.6 µs, against 11.5 ms with `to_value`, so the 10 ms bar is met. ADR 0013 amended (`8a7f002`). Orchestration 22.
- **[G41]** The three merged local branches are deleted: `docs/adr-distillation`, `docs/agents-md-structure`, `docs/task-4.1-checkoff` (no commit; `git branch -d` after `git branch --merged main`). Orchestration 22.
- **[G40]** Seven registration sleeps in `tests/integration_tests.rs` are polls of the daemon agent list (`ddb9ed2`, 22.1.3). Three sleeps are kept by design: 616 and 621 (Sway event ordering) and 931 (spawned hooks and process execution). Orchestration 22.
- **[G39]** The `status --watch --diff` fold is a pure helper, `fold_diff_line` in `src/cli.rs`, with behavioural tests for snapshot, upsert/remove and non-JSON lines (`d5eac18`, 22.1.2). Orchestration 22.

- **[G38]** Local commits published to `origin/main` (`fb34936`). Verified 2026-11-08: `origin/main` and local `main` are equal.
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
