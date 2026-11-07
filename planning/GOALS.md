# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-07
Source State: STATE_16.md (Orchestration 16 complete; ADRs 0001-0011 accepted; `make all` 25 unit + 10 integration passing)

## Circled Goal (Active)

- None ready without an owner decision. Pick G6 or G8 below.

## Sequenced Goals (Dependency DAG)

- None open.

## Independent Backlog

- **[G6]** Daemon-down fallback is decided and documented: keep the flat cycle and document it, or make the CLI start the daemon. Current default: keep flat and document (ADR 0003 rejected a shared state file).
  - Needs an owner decision. Choosing "keep flat" is a docs-only change. Choosing "CLI starts the daemon" is a behavior change.
  - Awaiting decision: keep flat (docs-only) or CLI starts the daemon (behavior change).
- **[G8]** A test-layout and CI ADR is written (candidate listed in `docs/decisions/README.md`).
  - Needs the owner to state the convention to record. Only `make test` exists today.
  - Awaiting decision: the convention to record.

## Deferred

- **[G9]** Full-state broadcast cost is reduced to diffs (ADR 0008 upgrade path). Depends on G5 (complete).
  - Trigger: measurable HUD lag, or agent counts that make full-list pushes noticeable.
- **[G10]** `XDG_CONFIG_HOME` is honored for the config path (ADR 0010 negative).
  - Trigger: a user request or a packaging need.

## Blocked / Pruned

- **[G11]** Adopt the JSON-RPC 2.0 envelope from the design doc.
  - Status: Pruned
  - Reason: ADR 0004 rejected it; the newline-delimited JSON protocol is in use.

## Completed

- **[G1]** Daemon focus follows Sway's actual focus. Commit `436b190`; test `test_focus_follows_sway_focus` (`tests/integration_tests.rs`). Live-Sway check from plan 13 is still outstanding (`planning/STATE_15.md`).
- **[G2]** Focus-state decision recorded as ADR 0011. Commit `874d7db`.
- **[G3]** Malformed config reports an error and falls back to defaults. Commit `5788c3e`. Live check from plan 15 is still outstanding.
- **[G4]** Children-mark rehome covered with several surviving siblings. Commit `3422164`.
- **[G5]** Subscriber drop-on-full policy recorded in ADR 0008 and covered by `test_daemon_broadcast_drops_newest_when_subscriber_full`. Commit `55314d4` (plan 16).
- **[G7]** Planning docs reconciled: the five questions in `planning/tasks/4.1.todo.md` are marked resolved, each pointing to ADR 0007, ADR 0011, or the code. The JSON-RPC section of `planning/Sway Multi-Agent Viewer Architecture.md` carries a superseded note pointing to ADR 0004. Uncommitted at time of writing.
