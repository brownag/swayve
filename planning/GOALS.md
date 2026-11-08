# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-08
Source State: STATE_18.md (Orchestration 18 complete and committed at `01f2da4`; `make all` passes)

## Circled Goal (Active)

- **[G22]** The daemon's runtime socket path is enforced by tests and documented as the external client contract.
  - Why now: The Antigravity sidecar (G15) needs a stable, documented socket path to build against. Today the path is verified only by a manual live check (C2), and the Antigravity README still names the old `/tmp/swayve.sock` default (C3). Fixing both is the internal part of G15 and unblocks it.
  - Exit criteria:
    - An automated test asserts that the socket directory is created with mode 0700 and owned by the current uid, and that stale-socket removal refuses a non-socket and a foreign-owned path.
    - `integrations/antigravity/README.md` gives the default socket as `$XDG_RUNTIME_DIR/swayve/swayve.sock` and names `SWAYVE_SOCKET` as the override.
    - The socket path and newline-delimited JSON framing are stated once in `README.md` as the external client contract.
    - `make all` passes.
  - First action: Read `src/cli.rs` `runtime_dir_from` and `ensure_private_dir`, then write a failing unit test for the directory mode and ownership check.

## Sequenced Goals (Dependency DAG)

- **[G23]** `planning/GOALS.md` and `planning/STATE_18.md` agree on G17 status.
  - Depends on: none (trivial; do first, it can be done in the same commit as G22's planning update)
  - Note: the live check passed 2026-11-08, so G17 moves to Completed.

- **[G24]** The Antigravity integration documents the runtime socket default.
  - Depends on: G22 (exit criterion for the socket contract)

- **[G25]** The external Antigravity sidecar ships against the documented socket contract.
  - Depends on: G22, G24
  - Note: this is external work. Track it here only once G22 is complete.

## Independent Backlog

- **[G26]** `handle_client` drops no client on a malformed IPC line. A test sends a malformed line followed by a valid `focus` message and asserts the valid message is still delivered.
  - Status: Verified by reading (`src/daemon.rs:251-281` logs and continues on parse error). No test covers it yet.
  - Caveat: a non-UTF-8 line makes `lines.next_line()` return an error, which closes that client. Decide whether that is acceptable before writing the test.

- **[G27]** Old `EXECUTION_PLAN_15–17.md`, `STATE_15–17.md` and `PROMPT_TEMPLATE_15–17.md` move to `planning/archive/` once their work is closed.
  - Low value. Planning hygiene only.

## Deferred

- **[G9]** Full-state broadcast cost is reduced to diffs (ADR 0008 upgrade path). Depends on G5 (complete).
  - Trigger: measurable HUD lag, or agent counts that make full-list pushes noticeable.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path (ADR 0010 negative).
  - Trigger: a user request or a packaging need. Kept separate from G17 by decision on 2026-11-07.

## Blocked / Pruned

- **[G15]** An Antigravity sidecar integration exists in this repo. `integrations/antigravity/` has a README only.
  - Status: Blocked / External Watch
  - Reason: the sidecar is built and owned outside this repo. Its internal prerequisites are G22 and G24.
- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.

## Completed

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
