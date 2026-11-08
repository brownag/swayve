# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-07
Source State: STATE_17.md (Orchestration 17 complete; `make all` 26 unit + 10 integration passing)

## Circled Goal (Active)

- **[G17]** The daemon socket and agent logs live in a per-user runtime directory, and the daemon never deletes a socket it does not own.
  - Why now: the socket is at a fixed path in world-writable `/tmp` (`src/cli.rs:74`). The daemon removes whatever file is there before binding (`src/daemon.rs:198-199`), so another local user can squat the path and block the daemon. Agent logs under `/tmp/swayve/logs` (`src/cli.rs:242-244`) can hold prompt text and are also world-readable.
  - Exit criteria: the default socket path is `$XDG_RUNTIME_DIR/swayve/swayve.sock` in a directory created with mode 0700. Logs go to the same directory. Stale-socket removal happens only when the file is a socket owned by the current user. `make all` passes. `SWAYVE_SOCKET` is still honored when set.
  - First action: read `src/cli.rs` lines 70-80 and `src/daemon.rs` lines 190-205, then write the plan.

## Sequenced Goals (Dependency DAG)

- None open.

## Independent Backlog

- **[G18]** The README CLI reference lists every subcommand, including `init-shell` and `status --watch`. Docs only.
- **[G19]** `cargo clippy --all-targets` is clean: 15 warnings in the lib, 6 in the integration tests. Mechanical, no behavior change.
- **[G20]** `make test` also runs the Pi client tests (`npm test`) and `integrations/claude/test-hooks.sh`.
- **[G21]** The Sway command strings built in `src/sway.rs` and the types in `src/types.rs` are pinned by unit tests. Both files have none today.

## Deferred

- **[G9]** Full-state broadcast cost is reduced to diffs (ADR 0008 upgrade path). Depends on G5 (complete).
  - Trigger: measurable HUD lag, or agent counts that make full-list pushes noticeable.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path (ADR 0010 negative).
  - Trigger: a user request or a packaging need. Kept separate from G17 by decision on 2026-11-07.

## Blocked / Pruned

- **[G15]** An Antigravity sidecar integration exists in this repo. `integrations/antigravity/` has a README only.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. It depends on Antigravity's API, which is not available here.
- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.

## Completed

- **[G14]** Planning waves 1–14 archived into `planning/archive/` with `git mv` (17.1.3). `AGENTS.md` points at `EXECUTION_PLAN_N.md` and `STATE_N.md` with the highest N as current.
- **[G13]** Daemon-down fallback test: `test_flat_fallback_cycles_agent_windows_in_tree_order` pins the flat selection (17.1.1). README and ADR 0003 say the fallback cycles agent windows only (17.1.2).
- **[G16]** Stale docs and CLI trap fixed: Pi and Antigravity README drift, stale `TODO.md` removed, `spawn --parent` documents the full agent ID and warns when the parent is unregistered. Commits `398fa92`, `66ef55c`.
- **[G12]** Malformed config is reported live: `swayve status` with a malformed `config.toml` prints the path and parse error to stderr, exits 0, and runs with defaults. Checked 2026-11-07 (`planning/STATE_16.md`).
- **[G1]** Daemon focus follows Sway's actual focus. Commit `436b190`. Live check passed on 2026-11-07 (`planning/STATE_16.md`).
- **[G2]** Focus-state decision recorded as ADR 0011. Commit `874d7db`.
- **[G3]** Malformed config reports an error and falls back to defaults. Commit `5788c3e`. Unit test plus the G12 live check.
- **[G4]** Children-mark rehome covered with several surviving siblings. Commit `3422164`. Live check passed on 2026-11-07: the mark moved to the surviving sibling.
- **[G5]** Subscriber drop-on-full policy recorded in ADR 0008 and covered by `test_daemon_broadcast_drops_newest_when_subscriber_full`. Commit `55314d4`.
- **[G6]** Daemon-down fallback decided: the CLI keeps the flat fallback and does not start the daemon. Recorded in the README note on the `$mod+a` binding and ADR 0003. No new ADR. Commit `d313105`.
- **[G7]** Planning docs reconciled. Commit `ccf52a7`.
- **[G8]** Test layout and CI convention recorded in `AGENTS.md`, not an ADR. Commit `d313105`.
