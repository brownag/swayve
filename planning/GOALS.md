# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-08
Source State: magic-wand re-rank on 2026-11-08 (after Orchestration 30, HEAD `8d78ec0`; scope: swayve repo, planning docs, code markers, `cargo audit`). Earlier: STATE_28.md (Orchestration 28 closed at `b37b0e2`). Correction 2026-11-08: G80 was marked Completed in error; its residual is reopened as the Circled Goal. Earlier: STATE_27.md (Orchestration 27 closed at `1f79de0`). G79–G81 were added from `planning/ARCHITECTURE_REVIEW_2026-11-08_FOLLOWUP.md` (reviewed at `17c72ce`; static review, tests not run). Earlier: STATE_26.md (Orchestration 26 closed at `00a4675`; `make all` green; pushed to `origin/main`). Correction 2026-11-08: the architecture review candidates G67–G70 were already done in earlier orchestrations; the earlier ranking missed git history and is withdrawn.

## Circled Goal (Active)

- **[G82]** Daemon-spawned children are reaped. The daemon drops every `Child` it launches: lifecycle hook commands (`execute_lifecycle_hook`, `src/daemon.rs:~323`) and monitor terminals (`spawn_monitor_terminal`, `src/daemon.rs:~276`). The daemon is long-lived, so each exited child stays a zombie until the daemon exits. Scope widened from hooks alone during the 2026-11-08 audit.
  - Why now: it is a reproducible leak in the long-running process, and the fix is local. It is the only open defect the audit confirmed in the daemon. It also touches `daemon.rs`, so it is the change that G78 names as a trigger; the added lines are few, so G78 stays deferred.
  - Exit criteria: a test runs a daemon-spawned hook (`exit 0`) and a monitor launch, waits for them, and asserts neither is left as a zombie child of the test process (for example by reading `/proc/self/task/*/children` or the child's `/proc/<pid>/stat` state). Targeted `cargo test --lib` and `cargo test --test integration_tests` pass; serial `make all` exit 0, zero warnings.
  - First action: write the failing zombie test for the hook path in `src/daemon.rs`'s test module, then reap the `Child` (a waiter thread or a `tokio::process::Child`) in one helper that both launch paths call.

## Sequenced Goals (Dependency DAG)

None. No open goal depends on another.

## Independent Backlog

- **[G83]** `cargo audit` reports no unsound or vulnerable crate in `Cargo.lock`. Today it reports RUSTSEC-2026-0386 ("`AsyncFd::new` and related constructors violate I/O safety") against `tokio 1.53.1`, which swayve depends on directly. Found during the 2026-11-08 audit. Not verified: whether the advisory's code path is reachable from swayve, and whether a patched tokio release exists.
  - First action: read RUSTSEC-2026-0386 for the patched version and affected constructors. If a patched tokio exists, bump it and run `make all`. If none exists or the path is unreachable, record that in this entry and ignore the advisory with a reason in the audit configuration.
- **[G73]** `npm run build` (or a `typecheck` script) type-checks `integrations/pi/tests/`. Today `tsconfig` includes only `src/`, and `tsx --test` skips type checks. Found during Orchestration 26 (task 26.1.1).
- **[G75]** The planning design docs (`planning/Sway Multi Agent Architecture Design.md`, `planning/Sway Multi-Agent Viewer Architecture.md`) carry a header that marks them historical and points to the ADRs and the current protocol.

## Deferred

- **[G84]** Decisions from Orchestrations 25–30 that outlive their plans are recorded as ADRs: the single argv helper and bare-splice rule (`expand_terminal_args`, G80), the pure spawn plan (G60, G64), and the hook env contract and test seam (G74). The existing ADR 0006 already covers the env names. Found during the 2026-11-08 audit.
  - Trigger: before `planning/` archives EXECUTION_PLAN_25–30 (the `adr-distillation` step in `wave-orchestration`).

- **[G66]** The two timing benchmarks in `src/daemon.rs` (`measure_broadcast_cost`, `measure_broadcast_cost_diff`) run in a gate, or are retired.
  - Trigger: a suspected broadcast performance regression, or a decision to add a perf gate.

- **[G77]** A Waybar module for agent status (design doc section "Ambient Desktop Awareness").
  - Trigger: a request for a Waybar module. No code or README exists in the repo.
- **[G78]** `src/daemon.rs` tests move out of the 1365-line file.
  - Trigger: the next change to `daemon.rs` that needs more room. Size alone is not a defect.
  - Note: G79 and G81 both edit `daemon.rs`. If either adds more than a few hundred lines, reconsider G78 first.

## Blocked / Pruned

- **[G76]** The design doc's JSON-RPC 2.0 envelope and its four methods (`agent.heartbeat`, `agent.request_window_action`, `orchestrator.interrupt`, `orchestrator.focus_changed`) are adopted.
  - Status: Pruned
  - Reason: ADR 0004 rejected the envelope, and none of the four methods has a code hit in `src/` or `tests/`. If a client later needs one, open a new goal for that method.
- **[G71]** Name the spawn plan as a pure function extracted from the CLI arm (review candidate 5).
  - Status: Pruned
  - Reason: already delivered. G60 (Orchestration 25) moved the plan into the pure `spawn_plan`, and G64 (Orchestration 26) gave it a request struct.
- **[G65]** Daemon-launched monitor terminals get the same child env as `swayve spawn`.
  - Status: Pruned
  - Reason: `spawn_monitor_terminal` (`src/daemon.rs`, around line 287) runs the monitor with `.args(...)` and no env, and nothing in the repo reads `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, or `SWAYVE_PROJECT_ID` from that process (Orchestration 26, D3).
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

- **[G74]** Lifecycle hooks are tested. `lifecycle_hook_command` (pure builder, `src/daemon.rs`) pins the `sh -c` argv and the `SWAYVE_AGENT_ID`/`SWAYVE_AGENT_STATUS` env pair; `agent_status_to_str` covers all six variants; `test_lifecycle_status_hook_order` runs the daemon and asserts status-change lines arrive in `UpdateStatus` order (`43b183b`, `3004dd6`; Orchestration 30; `make all` serial exit 0, zero warnings).

- **[G80]** Agent, monitor and panel terminal launches share one argv path. `spawn_plan` shlex-splits the command and calls `expand_terminal_args`. An integration test runs the real binary and pins the agent argv (whitespace path, quoted prompt, shell template) (`bcbe4a0`, `c2ce284`; Orchestration 29; `make all` serial exit 0, zero warnings).
- **[G81]** Subscription lifetime owned in one place. Unchanged diff broadcasts drop closed subscriber channels, and one `subscriber_writer` serves full and diff modes (`04da0d4`, `7726496`; Orchestration 28; `make all` serial exit 0, zero warnings).
- **[G79]** Agent identity and window ownership are coherent: re-registration keeps the container binding, rebinding releases the old window, the CLI and Pi derive the same `app_id` (lossless, `agent.<agent_id>`), and binding publishes to subscribers (`3bdc05c`, `b5932dd`, `de3fc7a`, `256974c`; Orchestration 28; `make all` serial exit 0, zero warnings).
- **[G72]** `make check` runs `cargo clippy --all-targets -- -D warnings`, so `make all` fails on any lint warning (Orchestration 27, `1f79de0`; `make all` serial exit 0, zero warnings).
- **[G70]** One typed `DiffFrame` is shared by the daemon and the CLI for the status diff wire shape (`9843d6f`).
- **[G69]** `find_registered_parent_ancestor` takes a `parent_of` closure, so the depth-limit test runs against a fake tree (`8460157`).
- **[G68]** The focus cycle is a `FocusCycle` struct with `observe()` and `next()` (`1012fd5`, G58).
- **[G67]** The auto-monitor rule lives in one `DaemonState` method (`auto_monitor_wanted`) that returns a `MonitorRequest`; the loop executes the spawn (`55489c7`, G54).
- **[G64]** `spawn_plan` takes a `SpawnRequest<'a>` struct, and the `too_many_arguments` allow is removed (Orchestration 26, `9b31e73`; the 7 `spawn_plan` tests pass).
- **[G63]** Pi tests leave no socket files in the package directory. `dummy.sock` goes into a per-run `mkdtemp` dir, and the `test.sock` cleanup test is removed (Orchestration 26, `bf5feaf`; Pi `npm test` 6/6).
- **[G59]** `swayve spawn --parent X` sets `SWAYVE_PARENT_ID=X` in the child env, and the variable is unset without `--parent` (Orchestration 25, `7b72405`; test `test_spawn_plan_sets_parent_env_when_given` and `test_spawn_plan_omits_parent_env_when_absent`).
- **[G61]** The Pi client sends `parent_agent_id` on `agent.register` from `SWAYVE_PARENT_ID`, or `null` when unset. Its README lists the variable (Orchestration 25, `556c678`; 3 new Pi tests).
- **[G60]** The child env is built in the pure `spawn_plan` and the spawn arm executes it (Orchestration 25, `b0438ec`).
- **[G62]** Integration tests use per-run temp paths with no `/tmp` literal in the hook script or Pi test (Orchestration 25, `1fbb95e`).
- **[G58]** Focus cycle owns its cursor: `FocusCycle` with `observe()` and `next()` in `src/daemon.rs` (`1012fd5`, 24.1.2). Unit tests cover the cursor moves and the no-target case.
- **[G52]** Re-scoped and built (Orchestration 24, `8c9c420`). `swayve spawn` writes its register line and polls `daemon.subscribe` until the agent is listed, then launches (`register_and_wait`, `src/cli.rs`). The wire ack is pruned: no daemon reply is added, and ADR 0009 and 0014 are unchanged. 2 s timeout, then a warning and launch anyway.
- **[G54]** Completed in Orchestration 23 (`55489c7`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G56]** Completed in Orchestration 23 (`1b1c3b0, 9843d6f`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G55]** Completed in Orchestration 23 (`8460157`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G57]** Completed in Orchestration 23 (`fe98801`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G51]** Completed in Orchestration 23 (`602ffa1`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G50]** Completed in Orchestration 23 (`0d985c7`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G49]** Completed in Orchestration 23 (`51c62ca`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G47]** Completed in Orchestration 23 (`f1b5a37`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
- **[G46]** Completed in Orchestration 23 (`b4dd9fa`). See `EXECUTION_PLAN_23.md` task for this goal; `STATE_23.md` has the gate results.
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
