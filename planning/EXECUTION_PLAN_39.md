# Execution Plan 39: Proof-First OpenCode v2.3.0 Lifecycle Integration (G89)

Target repo: `/home/andrew/workspace/swayve`
Target branch: `feat/opencode-integration1`
Recon base: Orchestration 38 and `planning/RESEARCH_OPENCODE_INTEGRATION.md`; focused OpenCode tests passed at plan time.

## Goal & Scope

**Goal.** First prove that OpenCode v2.3.0 delivers its lifecycle events to the Swayve plugin and that messages formed from the verified `event.data` shape reach the active daemon. Only after positive live evidence, permanently correct event/session/workspace extraction, pin the v2.3.0 event shape in mock-socket tests, and update integration documentation. If the live probe fails to prove that path, stop before permanent implementation and report the observed boundary.

**In scope.**
- Use one temporary, sanitized, file-backed plugin probe at `.development/opencode-trace.jsonl`; correlate the single session internally with a fixed alias such as `probe-session`, never write the raw OpenCode session ID to the trace or report.
- Run one harmless, read-only model turn using `openai/gpt-6-luna`.
- Capture callback event types, top-level/data keys, a stable redacted session alias, Unix-socket connect/write outcomes, and corresponding daemon state.
- On positive evidence, use only the verified v2.3.0 event payload shape (`event.data.sessionID`, `event.data.location.directory`) with a tested plugin-location fallback.
- Add mock-socket tests using real v2.3.0-shaped events and update the OpenCode integration README's API-support statement.

**Out of scope.**
- OpenCode `parentID` to Swayve `parent_agent_id` mapping (G90).
- Swayve Rust/daemon changes, root Makefile changes, new dependencies, other integrations, provider/auth configuration, and remote Git operations.
- Permanent diagnostics, raw payload/log retention, and claims of compatibility beyond the tested v2.3.0 release.

**Verbatim user constraints.**
- “Require live proof first”
- “Yes, one more turn”
- “Use temporary trace file”
- “Proceed only on proof”
- “v2.3.0 shape only”
- “Focused + live”
- Target model: `openai/gpt-6-luna`.
- Planning artifacts are tracked separately from implementation.

## Recon

- `.opencode/plugins/swayve.ts` (84 lines) subscribes to all eight desired lifecycle event names, but reads session identity from `event.properties`; `send()` suppresses socket errors and does not report connect/write completion.
- `integrations/opencode/sidecar.test.mjs` (218 lines) currently constructs `properties.info` events. It tests registration/status mapping, fallback, cleanup, and best-effort socket failure.
- `integrations/opencode/README.md` (59 lines) currently says no release pin is claimed and describes `session.directory`.
- `planning/RESEARCH_OPENCODE_INTEGRATION.md` now records first-party v2.3.0 source findings: event names are present; IDs are in `event.data.sessionID`; creation location is `event.data.location.directory`; prior live log capture did not expose callback diagnostics.
- First-party v2.3.0 source defines `LocationRef.directory` and the lifecycle payloads; see research note citations and tag `v2.3.0`.
- Current focused baseline: `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` passed 4/4 at plan time. Node emitted the existing harmless `MODULE_TYPELESS_PACKAGE_JSON` warning.
- Last full workspace baseline is historical: Orchestration 36 `make all` in `planning/STATE_36.md`; this isolated integration task uses the focused test and live trace only. Reconfirm branch, worktree, runtime, and daemon immediately before Wave 1.
- Accepted ADRs 0004, 0006, 0009, and 0012 constrain newline-delimited IPC, agent identity, thin socket clients, and socket path behavior. No ADR is superseded.
- Branch is `feat/opencode-integration1`. `planning/GOALS.md` is part of this planning update; `planning/RESEARCH_OPENCODE_INTEGRATION.md` already has an unrelated/uncommitted research addition and must be preserved, not staged with this plan.

## Decision Log

- **D1 (2026-11-17): Proof before permanent fix.** First run a temporary probe. Do not make permanent implementation changes unless live evidence positively confirms callback delivery and socket/daemon behavior; otherwise stop and report.
- **D2 (2026-11-17): One additional model turn.** Use exactly one harmless read-only prompt with `openai/gpt-6-luna`; do not fall back to another model.
- **D3 (2026-11-17): Trace sink.** Write sanitized callback type/key/session and connection/write results to `.development/opencode-trace.jsonl`; delete it and revert probe instrumentation after the run. Do not depend on OpenCode's console-log routing.
- **D4 (2026-11-17): Payload contract.** Permanent implementation targets only the verified v2.3.0 `event.data` shape; do not retain the unsupported `event.properties` fallback.
- **D5 (2026-11-17): Workspace identity.** On session creation use `event.data.location.directory`; retain `ctx.location.directory` as fallback if creation location is absent.
- **D6 (2026-11-17): Acceptance gate.** Require positive live evidence plus the focused mock-socket suite. Run `make all` only if work expands outside the OpenCode integration.
- **D7 (2026-11-17): Planning and branch.** Track planning artifacts in separate `docs(planning):` commits; remain on `feat/opencode-integration1`; no remote operations.
- **D8 (2026-11-17): G90 deferred.** Observe but do not implement parent mapping.

## Hazard Notes

- Confirm the live daemon/socket and branch before the smoke; never edit daemon/Rust code or alter provider/auth configuration.
- The read-only turn may incur provider usage. It must not write files.
- Only the specific temporary trace file may be created under `.development/`; remove it and temporary instrumentation. Preserve unrelated `.development/` content.
- Do not record raw session IDs: use a fixed alias such as `probe-session` to correlate the single session across callback and socket records. Do not record full event payloads, prompt output, secrets, or raw logs. The concise Task 39.1.1 report in `planning/STATE_39.md` is the durable sanitized evidence record.
- A successful socket write is not itself proof of daemon application. Correlate with an active `swayve status --watch --diff` observer.
- If callback delivery, session identity, or write/state evidence is absent or contradictory, mark the gate failed/inconclusive and do not proceed to Wave 2.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 39.1.1 | Temporarily trace v2.3.0 event delivery, `data` payloads, socket outcomes, and daemon state with one read-only turn | `.opencode/plugins/swayve.ts`, `.development/opencode-trace.jsonl` (temporary only) | — |
| 2 | 39.2.1 | After positive Wave 1 proof, permanently map v2.3.0 payloads and cover/document the contract | `.opencode/plugins/swayve.ts`, `integrations/opencode/sidecar.test.mjs`, `integrations/opencode/README.md` | 39.1.1 positive proof |

## Task 39.1.1: Live-Probe OpenCode v2.3.0 Event and Socket Path

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit temporarily:** `.opencode/plugins/swayve.ts`; `planning/STATE_39.md` for a concise sanitized evidence report only.
- **Files in scope — can create temporarily:** `.development/opencode-trace.jsonl` only; remove before completion. Preserve all other `.development/` content.
- **Depends on:** none.
- **Spec:** Record Git status, runtime version, active Swayve socket, and initial daemon state from a concurrent `swayve status --watch --diff` observer (the `--json` flag is not a watch-mode state feed). Add temporary instrumentation before session-ID extraction that records only event type, top-level/data key names, and the fixed alias `probe-session`; temporarily use `event.data.sessionID` to form registration/status messages. Record per-send socket connect and write completion/error to the trace file. Keep the observer attached while running exactly one turn: `opencode run --standalone --model openai/gpt-6-luna "Read the first heading in README.md and report it. This is read-only; do not create or modify files."` Capture the observer's initial snapshot and subsequent state refreshes, including the retained final state. Correlate callback trace, connect/write records, and daemon state for the same agent, keeping the raw session ID only transiently in process memory for correlation. Record only sanitized evidence in the Task 39.1.1 report in `planning/STATE_39.md`. Revert the plugin probe and delete the trace file. Do not make the permanent fix in this task.
- **Constraints (do not touch):** tests, README, Rust/daemon code, provider/auth settings, root Makefile, unrelated files, parent mapping, or remotes. Do not run a second model turn or fallback to another model. Do not commit temporary instrumentation/output.
- **Grep anchors/reference:** `.opencode/plugins/swayve.ts` (`eventTypes`, `send`, `setup`, `event.properties` extraction); `planning/RESEARCH_OPENCODE_INTEGRATION.md` v2.3.0 contract section; `.development/` is gitignored.
- **Acceptance:** Preserve sanitized evidence that `session.created` and execution/tool lifecycle callbacks were yielded with `data.sessionID`; trace successful connect/write outcomes for registration and status messages; correlate the redacted alias to the same daemon agent. The watch's initial snapshot must not contain that new agent; subsequent diff/dashboard state must show its registration and a retained lifecycle status (expected `completed`), distinguishing the update from baseline. If callback/send evidence or the matching daemon state transition is absent or contradictory, report the exact missing boundary and mark Wave 1 inconclusive. Temporary instrumentation and trace file are absent at task end. No permanent implementation changes or test runs.

## Task 39.2.1: Implement and Test the Proven v2.3.0 Payload Mapping

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit:** `.opencode/plugins/swayve.ts`; `integrations/opencode/sidecar.test.mjs`; `integrations/opencode/README.md`; `planning/STATE_39.md` for the task report.
- **Depends on:** positive evidence recorded for Task 39.1.1. If Wave 1 is inconclusive or fails, do not execute this task.
- **Spec:** Replace the unsupported `event.properties` extraction with `event.data.sessionID`. On `session.created`, derive workspace/project basename from `event.data.location.directory`, retaining plugin-load-directory fallback when missing. Update mock events and assertions to use v2.3.0 event payloads; cover registration, status mapping, missing identity/location fallback, duplicate creation, cleanup, and best-effort socket behavior without adding dependencies. Update README API support text to identify the verified v2.3.0 shape and avoid claims beyond that version.
- **Constraints (do not touch):** daemon/Rust code, other integrations, root Makefile, research note, provider/auth settings, dependencies, parent mapping, or remotes.
- **Grep anchors/reference:** `.opencode/plugins/swayve.ts` (`sessionID`, `workspace`, `status`); `integrations/opencode/sidecar.test.mjs` (`event`, lifecycle test, fallback test); `integrations/opencode/README.md` (`API support`, session identity); first-party field citations in `planning/RESEARCH_OPENCODE_INTEGRATION.md`; ADRs 0004, 0006, 0009, 0012.
- **Acceptance:** `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` passes with assertions built from `event.data` v2.3.0 shapes. The live proof from Task 39.1.1 is positive and recorded in sanitized form. `git diff --check` passes. README states the actual tested contract and bounded version claim. `make all` is not required unless scope expands beyond the OpenCode integration.
