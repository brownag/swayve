# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-07
Source State: STATE_16.md (Orchestration 16 complete; ADRs 0001-0011 accepted; `make all` 25 unit + 10 integration passing; live Sway focus check passed)

## Circled Goal (Active)

- None. G12 is complete (see Completed).

## Sequenced Goals (Dependency DAG)

- None open.

## Independent Backlog

- **[G13]** The daemon-down focus fallback has a test that pins its flat behavior. Covers the gap ADR 0003 records.
  - Low leverage. Nothing depends on it.
- **[G14]** Planning waves 1–14 are archived out of `planning/` into `planning/archive/`. `AGENTS.md`'s "Find things" section is updated to match.
  - Low leverage. Cleanup only, and reversible through git.

## Deferred

- **[G9]** Full-state broadcast cost is reduced to diffs (ADR 0008 upgrade path). Depends on G5 (complete).
  - Trigger: measurable HUD lag, or agent counts that make full-list pushes noticeable.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path (ADR 0010 negative).
  - Trigger: a user request or a packaging need.

## Blocked / Pruned

- **[G15]** An Antigravity sidecar integration exists in this repo. `integrations/antigravity/` has a README only.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. It depends on Antigravity's API, which is not available here.
- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.

## Completed

- **[G13]** Daemon-down fallback test: `test_flat_fallback_cycles_agent_windows_in_tree_order` pins the flat selection (Orchestration 17, task 17.1.1). README and ADR 0003 now say the fallback cycles agent windows only (17.1.2).
- **[G14]** Planning waves 1–14 archived into `planning/archive/` with `git mv` (17.1.3). `AGENTS.md` points at `EXECUTION_PLAN_N.md` and `STATE_N.md` with the highest N as current.
- **[G1]** Daemon focus follows Sway's actual focus. Commit `436b190`. Live check passed on 2026-11-07 (`planning/STATE_16.md`).
- **[G2]** Focus-state decision recorded as ADR 0011. Commit `874d7db`.
- **[G12]** Malformed config is reported live: `swayve status` with a malformed `config.toml` prints the path and parse error to stderr, exits 0, and runs with defaults. Checked 2026-11-07 (`planning/STATE_16.md`).
- **[G3]** Malformed config reports an error and falls back to defaults. Commit `5788c3e`. Unit test plus the G12 live check.
- **[G4]** Children-mark rehome covered with several surviving siblings. Commit `3422164`. Live check passed on 2026-11-07: the mark moved to the surviving sibling.
- **[G5]** Subscriber drop-on-full policy recorded in ADR 0008 and covered by `test_daemon_broadcast_drops_newest_when_subscriber_full`. Commit `55314d4`.
- **[G6]** Daemon-down fallback decided: the CLI keeps the flat fallback and does not start the daemon. Recorded in the README note on the `$mod+a` binding and ADR 0003. No new ADR. Commit `d313105`.
- **[G7]** Planning docs reconciled. Commit `ccf52a7`.
- **[G8]** Test layout and CI convention recorded in `AGENTS.md`, not an ADR. Commit `d313105`.
- **[G16]** Stale docs and CLI trap fixed: Pi and Antigravity README drift, stale `TODO.md` removed, `spawn --parent` documents the full agent ID and warns when the parent is unregistered. Commits `398fa92`, `66ef55c`.
