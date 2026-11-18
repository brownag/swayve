# Execution Plan 38: Trace OpenCode Lifecycle Status Path (G89)

Target repo: `/home/andrew/workspace/swayve`
Target branch: `feat/opencode-integration1`
Recon base: Orchestration 37 artifacts and research note; G89 diagnostic objective approved 2026-11-17.

## Goal & Scope

**Goal.** Trace the project-local Swayve plugin's lifecycle events and status IPC against OpenCode v2.3.0 and the active Swayve daemon. Distinguish absent/unrecognized events, payload extraction skips, socket delivery failures, and daemon application. Make only focused fixes required to resolve an observed plugin defect.

**In scope.**
- Trace event types/properties delivered to plugin callbacks and session-ID extraction.
- Trace attempted status IPC sends and connection/write outcomes.
- Run one controlled, harmless, read-only OpenCode turn and correlate plugin evidence with daemon logs/state.
- Apply a narrowly scoped plugin/test/documentation fix only if evidence identifies one.
- Record concise observations in `planning/RESEARCH_OPENCODE_INTEGRATION.md`.

**Out of scope.**
- OpenCode `parentID` mapping to Swayve `parent_agent_id` (G90).
- Swayve Rust/daemon changes, OpenCode focus actions, root Makefile changes, dependencies, unrelated integrations/configuration, and provider/auth changes.
- Raw logs, transcripts, secrets, or persistent diagnostics.

**Verbatim user constraints.**
- “The running Swayve daemon is less likely to be the cause, though it isn’t ruled out. Its code accepts valid status updates for registered agents and broadcasts state changes. The stronger lead is a mismatch between OpenCode v2.3.0’s lifecycle events and the plugin’s subscriptions or session-ID extraction; the live run didn’t trace events inside the plugin or confirm socket writes.”
- “No plugin code changed, and no further model turn was run.”
- Target runtime: installed OpenCode `v2.3.0`.
- Target branch: `feat/opencode-integration1`.
- Planning artifacts are tracked; planning-only commits remain separate from implementation.

## Recon

- `planning/STATE_37.md` records plugin loading and registration but leaves the live lifecycle path unverified.
- `.opencode/plugins/swayve.ts` (84 lines) subscribes to session/execution/tool events and sends `agent.register` / `agent.update_status`; `send` currently suppresses socket failures.
- `integrations/opencode/sidecar.test.mjs` (218 lines) exercises plugin setup through a mock Unix socket. Targeted command: `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs`.
- `integrations/opencode/README.md` (59 lines) contains the manual smoke checklist.
- `planning/RESEARCH_OPENCODE_INTEGRATION.md` records known event schema uncertainty, daemon behavior, and diagnostic distinctions.
- Orchestration 36's `make all` and focused OpenCode tests are the last complete green baseline (`planning/STATE_36.md`); no new implementation baseline is asserted.
- Active ADRs 0004, 0006, 0009, and 0012 constrain IPC framing, agent identity, thin socket integration, and socket-path behavior. No ADR is superseded.
- Worktree is on `feat/opencode-integration1`; tracked planning changes and untracked Orchestration 37 artifacts are present. Preserve them.

## Decision Log

- **D1 (2026-11-17): Diagnostic-first.** Trace callbacks, extraction, socket outcomes, and daemon application before attributing cause or changing subscriptions.
- **D2 (2026-11-17): Temporary diagnostics only.** Remove instrumentation after the controlled run unless a focused correction requires a permanent behavior change.
- **D3 (2026-11-17): One read-only model turn allowed.** The prompt reads the top-level README heading and requests no file changes; provider usage may incur cost.
- **D4 (2026-11-17): Runtime and branch.** Use installed OpenCode v2.3.0 on `feat/opencode-integration1`; no branch switch or remote Git action.
- **D5 (2026-11-17): Tracked planning.** Planning-only commits are separate from implementation commits.
- **D6 (2026-11-17): G90 remains deferred.** Observe parentID evidence only; do not implement or assert parent mapping.

## Hazard Notes

- Provider call may incur usage; do not alter provider/auth settings or send repository-sensitive content.
- Observe state/logs during the run; late snapshots cannot establish whether transient statuses occurred.
- Verify active socket path; do not change daemon configuration.
- Preserve unrelated work; inspect Git status before and after. Do not store raw logs or prompt output.
- If evidence requires Rust/daemon changes or other out-of-scope work, stop and report.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 38.1.1 | Temporarily instrument and trace OpenCode callbacks, IPC delivery, and daemon application; narrowly fix a confirmed plugin defect | `.opencode/plugins/swayve.ts`, `integrations/opencode/sidecar.test.mjs`, `integrations/opencode/README.md`, `planning/RESEARCH_OPENCODE_INTEGRATION.md` | — |

## Task 38.1.1: Trace OpenCode v2.3.0 Lifecycle Status

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit:** `.opencode/plugins/swayve.ts`; `integrations/opencode/sidecar.test.mjs`; `integrations/opencode/README.md`; `planning/RESEARCH_OPENCODE_INTEGRATION.md` (concise sourced observations only).
- **Files in scope — can create:** `.development/` for temporary, non-sensitive diagnostic output; remove it before completion.
- **Depends on:** none.
- **Spec:** Record initial Git status. Add temporary diagnostics for callback event type/properties, extracted session ID, and each status IPC attempt plus connection/write result. Start daemon state/log observation, then run from repository root: `opencode run --standalone "Read the first heading in README.md and report it. This is read-only; do not create or modify files."` Use installed OpenCode v2.3.0, active Sway/Swayve, and discovered socket path. Correlate event receipt, extraction, send outcome, daemon logs, and changed state to identify the failing boundary. If a plugin defect is confirmed, make the smallest correction and add/update focused mock-socket coverage. Remove temporary diagnostics and output. Record concise evidence and any parentID observation for G90 only.
- **Constraints (do not touch):** Rust/daemon code, other integrations, root Makefile, provider/auth configuration, unrelated workspace files, remotes, or parent mapping. No new dependencies; no file-writing prompt.
- **Grep anchors/reference:** `.opencode/plugins/swayve.ts` (`eventTypes`, `socketPath`, `send`, `setup`); `integrations/opencode/sidecar.test.mjs`; `integrations/opencode/README.md` manual smoke; `planning/RESEARCH_OPENCODE_INTEGRATION.md`; ADRs 0004, 0006, 0009, 0012.
- **Acceptance:** Record actual callback event types/properties and session-ID extraction outcomes, IPC connection/write outcomes, and corresponding daemon log/state evidence. Explicitly identify the failing boundary or show the lifecycle path works; do not infer unobserved events/statuses. Verify registered identity fields when available. Temporary instrumentation/output is removed. If implementation changes, `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` passes. Record commands and concise findings without claiming compatibility beyond this runtime.
