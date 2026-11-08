# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-08
Source State: STATE_19.md (Orchestration 19 complete; `make all` passes at each wave gate; ADR 0012 recorded)

## Circled Goal (Active)

- **[G28]** No README or design text in this repo describes the daemon IPC as JSON-RPC. The `src/types.rs` link description matches ADR 0004 and ADR 0012.
  - Why now: `README.md:178` still says "JSON-RPC message framing" after the Socket Contract section (19.1.4) corrected the same claim at line 54. It is the only internal contradiction left in the contract. Leverage is low. It finishes the contract and nothing else depends on it.
  - Exit criteria:
    - `grep -n 'JSON-RPC' README.md` shows only the deliberate correction at line 54.
    - The `src/types.rs` link description names newline-delimited JSON with a `method` tag.
    - `make all` passes.
  - First action: Edit `README.md` line 178 to read "Central data structures and newline-delimited JSON message framing schemas." Then run the grep above.

## Sequenced Goals (Dependency DAG)

- None open. G22, G23, G24 and G26 are complete (see Completed).

## Independent Backlog

- None open. G27 is complete.

## Deferred

- **[G9]** Full-state broadcast cost is reduced to diffs (ADR 0008 upgrade path). Depends on G5 (complete).
  - Trigger: measurable HUD lag, or agent counts that make full-list pushes noticeable.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path (ADR 0010 negative).
  - Trigger: a user request or a packaging need. Kept separate from G17 by decision on 2026-11-07.

## Blocked / Pruned

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
