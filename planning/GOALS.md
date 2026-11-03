# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-02
Source State: STATE.md (Wave 12 complete; ADRs 0001-0010 accepted)

## Circled Goal (Active)

- **[G1]** Daemon focus follows Sway's actual focus, so `focus --cycle` continues from the window the user is really on.
  - Why now: `last_focused` (`src/daemon.rs:271`) only changes when the daemon itself focuses. `src/sway.rs` subscribes to window create and close events, not focus. After any manual focus change, the next cycle starts from a stale window. Every focus action depends on this (ADR 0007).
  - Exit criteria: a test changes focus outside the daemon, runs a focus cycle, and lands on the correct sibling. `cargo test` passes with no new warnings.
  - First action: subscribe to Sway window-focus events in `src/sway.rs` and send a focus event to the daemon that updates `last_focused`.

## Sequenced Goals (Dependency DAG)

- **[G2]** Record the focus-state decision as ADR 0011 (`last_focused` as the source of truth, fed by Sway focus events).
  - Depends on: G1

## Independent Backlog

- **[G3]** A malformed `~/.config/swayve/config.toml` reports an error instead of silently using defaults (ADR 0010; `src/config.rs` `load_config`).
- **[G4]** A test confirms that when a child closes, the children mark moves to the surviving sibling (ADR 0007 negative).
- **[G5]** The subscriber channel's drop-on-full behavior is recorded and covered by a test (ADR 0008; `try_send` at `src/daemon.rs:49`).
- **[G6]** Daemon-down fallback is decided and documented: keep the flat cycle and document it, or make the CLI start the daemon. Current default: keep flat and document (ADR 0003 rejected a shared state file).
- **[G7]** Planning docs reconciled: stale open questions in `planning/tasks/4.1.todo.md` closed, and the design docs' JSON-RPC sections marked superseded by ADR 0004.
- **[G8]** A test-layout and CI ADR is written (candidate listed in `docs/decisions/README.md`).

## Deferred

- **[G9]** Full-state broadcast cost is reduced to diffs (ADR 0008 upgrade path).
  - Trigger: measurable HUD lag, or agent counts that make full-list pushes noticeable.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path (ADR 0010 negative).
  - Trigger: a user request or a packaging need.

## Blocked / Pruned

- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.
