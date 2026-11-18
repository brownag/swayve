# Execution Plan 37: Trace OpenCode Lifecycle Status Path (G89)

Target repo: `/home/andrew/workspace/swayve`
Target branch: `feat/opencode-integration1`
Recon base: Orchestration 36 complete at `40f7947`; branch clean at planning start.

## Goal & Scope

**Goal.** Trace the project-local Swayve plugin's lifecycle events and status IPC against OpenCode v2.3.0 and the active Swayve daemon. The trace must distinguish absent/unrecognized events, payload extraction skips, socket delivery failures, and daemon application. Make only focused fixes required to resolve the observed failure.

**In scope.**
- Trace actual OpenCode event types/properties delivered to plugin callbacks, including session-ID extraction.
- Trace attempted status IPC sends and connection/write outcomes.
- Run the documented controlled smoke test with OpenCode v2.3.0 and active Swayve runtime; correlate plugin evidence with daemon logs/state.
- Use one harmless prompt that reads existing content and requests no file changes; external model usage may incur provider cost.
- Observe plugin load, session registration, event callbacks, IPC outcomes, and resulting daemon state.
- If verification exposes a defect, apply narrowly scoped fixes to the OpenCode plugin, its focused tests, or integration README.
- Record observed event/session field facts in the OpenCode research note without storing secrets or raw provider output.

**Out of scope.**
- Mapping OpenCode `parentID` to Swayve `parent_agent_id` (G90).
- OpenCode focus commands, tools, daemon/Rust changes, root Makefile changes, or new dependencies.
- Broad changes to unrelated integrations or configuration.
- File-writing prompts, provider configuration changes, account/auth changes, or any remote Git action.

**Verbatim user constraints.**
- “Allow focused fixes” limited to issues necessary for live loading/registration/status verification; parentID mapping remains G90.
- “Allow one harmless turn” — a no-file-change prompt is acceptable, even if it contacts a configured provider and incurs usage.
- Target runtime: installed OpenCode `v2.3.0`.
- Target branch: `feat/opencode-integration1`.
- Planning artifacts are tracked, with planning-only commits separate from implementation (carried forward from the approved Orchestration 36 decision).

## Recon (facts)

- Orchestration 36 implementation is complete; `planning/STATE_36.md` records targeted OpenCode tests 4/4 and a green final `make all`.
- The current branch is `feat/opencode-integration1`; worktree was clean when this plan began.
- `.opencode/plugins/swayve.ts` (84 lines) observes session/tool events and sends `agent.register` / `agent.update_status`.
- `integrations/opencode/sidecar.test.mjs` (218 lines) exercises actual plugin setup through a built-in resolver hook and mock Unix socket. Run with `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs`.
- `integrations/opencode/README.md` (59 lines) contains the manual smoke checklist.
- `planning/RESEARCH_OPENCODE_INTEGRATION.md` records first-party event/API findings and workspace fallback caveats.
- OpenCode v2.3.0 is installed; Sway 1.9 is active; `swayve status --json` reports the daemon active.
- `Session.Info.parentID` is documented by OpenCode; whether/how it appears on `session.created` is reserved for G90 after the live contract is observed.
- ADR 0009 requires a thin socket integration; ADRs 0004 and 0012 define the newline-delimited JSON and socket path contracts.
- The latest complete baseline is Orchestration 36's `make all`, recorded in `STATE_36.md`; no implementation files changed since.
- Root `make test` does not include the OpenCode integration, by prior explicit decision. Keep its focused test command independent.

## Decision Log

- **D1 (2026-11-17): Focused repair allowed.** Repair only defects blocking live plugin loading, registration, or lifecycle reporting; G90 parentID mapping remains out of scope.
- **D2 (2026-11-17): Provider-backed verification allowed.** One harmless read-only prompt may contact the configured model provider and incur usage. Do not write files or change provider/auth configuration.
- **D3 (2026-11-17): Runtime target.** Validate against the installed OpenCode v2.3.0, active Sway 1.9, and active Swayve daemon. Do not claim a broader compatibility range.
- **D4 (2026-11-17): Test strategy.** Keep dependency-free mock-socket tests as regression coverage; perform live validation as the task's manual/runtime acceptance.
- **D5 (2026-11-17): Planning artifacts tracked.** Planning-only commits remain separate from implementation.
- **D6 (2026-11-17): Branch.** Stay on `feat/opencode-integration1`; no branch switching.
- **D7 (2026-11-17): Read-only prompt.** Read the top-level README and report its first heading; do not create or modify files.
- **D8 (2026-11-17): Diagnostic priority.** Trace plugin callback event types/properties and each status send/write before attributing the failure to the daemon or changing event subscriptions.
- **D9 (2026-11-17): Plan supersession.** Update Orchestration 37 artifacts in place because its live run left the lifecycle path unresolved; retain tracked planning artifacts, with planning-only commits separate.

## Hazard Notes

- **H1. Provider use:** The allowed read-only prompt may incur usage. Do not alter model/provider/auth configuration or send repository-sensitive content.
- **H2. Worktree safety:** OpenCode runs in the Swayve repository. Use a prompt that requires no writes; inspect `git status` before and after and preserve any unexpected user changes.
- **H3. Event timing:** `session.created` and lifecycle events may be brief. Observe Swayve status concurrently or use daemon watch output; do not infer missing events from a late snapshot.
- **H4. Socket discovery:** The plugin uses `SWAYVE_SOCKET`, otherwise `$XDG_RUNTIME_DIR/swayve/swayve.sock`; verify the active daemon path without changing daemon configuration.
- **H5. Parent ID:** Do not implement or assert parent mapping in this task. Record only observed evidence for G90.
- **H6. Scope:** If a defect requires out-of-scope Swayve daemon, OpenCode configuration, dependency, or root test-gate changes, stop and report rather than expanding scope.
- **H7. Privacy:** Do not commit raw logs, session transcripts, prompt output, or secrets; record only concise observations.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 37.1.1 | Instrument and trace OpenCode v2.3.0 callbacks, IPC writes, and daemon application; fix only the diagnosed plugin issue | `.opencode/plugins/swayve.ts`, `integrations/opencode/sidecar.test.mjs`, `integrations/opencode/README.md`, `planning/RESEARCH_OPENCODE_INTEGRATION.md` | — |

## Task 37.1.1: Live-Smoke OpenCode v2.3.0

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit:** `.opencode/plugins/swayve.ts`; `integrations/opencode/sidecar.test.mjs`; `integrations/opencode/README.md`; `planning/RESEARCH_OPENCODE_INTEGRATION.md` (sourced observations/corrections only).
- **Files in scope — can create:** `.development/` for temporary, non-sensitive smoke-test notes/output only. These artifacts are untracked and must be removed before completion; only concise observations may be retained in the research note.
- **Depends on:** none.
- **Spec:** Record initial `git status`; add focused temporary diagnostics in the plugin for each received event type/properties, extracted session ID, and status send attempt/connection/write result. Start daemon state/log observation, then run `opencode run --standalone "Read the first heading in README.md and report it. This is read-only; do not create or modify files."` from the repository root. Use OpenCode v2.3.0, active Sway/Swayve, and the discovered socket path; stop observers after the run. Correlate callback evidence with daemon logs and state changes, distinguishing no callback, skipped extraction, failed send, and daemon non-application. If evidence reveals a plugin defect, make the smallest plugin/test/doc correction required, rerun the focused test, and repeat the smoke check. Remove temporary diagnostics or make them appropriately bounded/documented. Capture concise results and any `parentID` event evidence for G90 without asserting a parent value or implementing parent mapping.
- **Constraints (do not touch):** Rust/daemon code, other integrations, root Makefile, provider/auth settings, unrelated workspace files, or remotes. No file-writing prompts or new dependencies. Do not map `parentID` in this task.
- **Grep anchors/reference:** `.opencode/plugins/swayve.ts` (`eventTypes`, `socketPath`, `send`, `setup`); `integrations/opencode/sidecar.test.mjs`; `integrations/opencode/README.md` Manual smoke test; `planning/RESEARCH_OPENCODE_INTEGRATION.md`; ADRs 0004, 0006, 0009, 0012.
- **Acceptance:** OpenCode v2.3.0 loads `.opencode/plugins/swayve.ts` without plugin error; plugin diagnostics record actual callback event types/properties, session-ID extraction results, and status send connection/write outcomes. Correlate with daemon log/state evidence to identify the failure boundary or confirm the lifecycle path works. Verify registered agent identity fields (`agent_id`, `agent_type`, `project_id`, `pid`, `app_id`) and report which statuses were actually observed; do not infer absent statuses. Do not assert parent value (reserved for G90). Final Git status shows no unintended workspace edits and no temporary artifacts remain. If implementation changed, `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` passes. Record exact commands, observations, and blockers; do not claim a full compatibility guarantee from this single runtime.
