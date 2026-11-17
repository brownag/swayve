# Execution Plan 36: OpenCode Agent Integration (G85)

Target repo: `/home/andrew/workspace/swayve`
Target branch: `feat/opencode-integration1` (created from `refactor/workspace2` with user approval)
Recon: 2026-11-17

## Goal & Scope

**Goal.** Add a project-local OpenCode server plugin that registers OpenCode sessions as Swayve agents and reports session/tool lifecycle statuses through the existing newline-delimited Unix socket protocol. Keep the integration a thin socket client; resolve identity from OpenCode's documented event data, add mocked-socket tests, and document setup and a manual real-runtime smoke test.

**In scope.**
- Add a project-local plugin in `.opencode/plugins/` that uses OpenCode's documented server plugin event API.
- Map sessions to Swayve identity: session ID as agent ID, `agent.<session_id>` app ID, project ID as workspace-directory basename, process PID as plugin process PID, and null parent unless a reliable parent relationship is confirmed.
- Send existing `agent.register` and `agent.update_status` lines over Swayve's socket, best-effort.
- Add focused, independently runnable mock Unix-socket tests and setup/verification documentation.
- Keep generated planning artifacts tracked; commit planning separately from implementation after approval.

**Out of scope.**
- Daemon protocol or Rust changes.
- OpenCode commands, tools, focus controls, or other richer SDK behavior.
- Publishing a reusable npm package or adding test dependencies.
- Adding OpenCode tests to root `make test`.
- Implementing parent relationships absent a confirmed OpenCode event/API contract.

**Verbatim user constraints.**
- “Yes, thin socket plugin”
- “Project-local plugin”
- “Use this mapping” (session ID; `agent.<session_id>`; project from workspace; PID from plugin process; parent only when reliable)
- “Yes (Recommended)” for best-effort IPC, mock sockets, and manual OpenCode smoke test
- “Minimal Node dependencies”
- “Independent test command”
- Target branch: `feat/opencode-integration1`

## Recon (facts)

- Existing integrations are in `integrations/claude/`, `integrations/antigravity/`, and `integrations/pi/`; there is no OpenCode directory.
- Antigravity's `sidecar.mjs` and tests are dependency-free Node examples for one-way socket IPC and lifecycle mapping.
- `SWAYVE_SOCKET` overrides the default `$XDG_RUNTIME_DIR/swayve/swayve.sock` (ADR 0012).
- Existing IPC is newline-delimited JSON (ADR 0004); supported messages include `agent.register` and `agent.update_status`.
- `agent.register` carries `agent_id`, `agent_type`, `project_id`, `pid`, `app_id`, `parent_agent_id`; types are in `crates/swayve-types/src/lib.rs`.
- ADR 0009 requires thin socket integrations, not agent SDKs; the OpenCode plugin API is the runtime extension seam only.
- Official research is recorded in `planning/RESEARCH_OPENCODE_INTEGRATION.md`; event names/payloads must be verified against current official docs/types before implementation.
- Test commands: Antigravity `node --test sidecar.test.mjs`; Pi `npm test` and `npm run build`; Claude `integrations/claude/test-hooks.sh`. Root full gate: `make all`.
- Baseline `make all` is explicitly a pre-wave-1 verification step; it has not been run during planning.
- Current branch is `feat/opencode-integration1`; worktree has pre-existing planning edits (`planning/GOALS.md`, `planning/RESEARCH_OPENCODE_INTEGRATION.md`).

## Decision Log

- **D0 (2026-11-17): Plan tracking.** Track planning artifacts in Git; keep planning-only commits separate from implementation.
- **D1 (2026-11-17): Thin socket plugin.** Follow accepted ADR 0009. Use OpenCode's plugin API only to observe lifecycle events; send existing Swayve messages without adding richer runtime behaviors.
- **D2 (2026-11-17): Project-local delivery.** Place plugin under `.opencode/plugins/`; do not package/publish it.
- **D3 (2026-11-17): Identity.** Use OpenCode session ID as Swayve agent ID, `agent.<session_id>` app ID, workspace basename as project ID, and plugin process PID. Parent ID is null unless official event data establishes a reliable relation.
- **D4 (2026-11-17): Best-effort verification.** Socket failures must not affect OpenCode. Use mock Unix sockets for automated tests and document manual smoke testing against a supported OpenCode runtime.
- **D5 (2026-11-17): Minimal dependencies and independent tests.** Use Node built-ins for socket operations; do not add a test framework or root Makefile integration.
- **D6 (2026-11-17): Sequential waves.** One task per wave; avoid coordination overhead for tightly coupled plugin/test/docs work.
- **D7 (2026-11-17): Planning artifacts tracked.** Keep plan, prompt, state, and task TODOs in repository; planning commits stay separate from implementation.
- **D8 (2026-11-17): Branch.** Target `feat/opencode-integration1`, created from `refactor/workspace2` with user approval.

## Hazard Notes

- **H1. API drift:** OpenCode is actively evolving. Confirm exact event identifiers and fields from current first-party docs and types; pin/document supported version. Do not assume event names from unofficial sources.
- **H2. Directory scope:** `.opencode/plugins/` is project-local and tracked by Git, but Swayve's root `.gitignore` ignores `*.js`; prefer `.ts` source or explicitly handle ignore rules without broadening generated-file tracking.
- **H3. Identity validity:** Session ID format may have constraints. Confirm it is accepted by `AgentId` and Swayve registration before treating it as stable identity.
- **H4. Project identity:** `ctx.location` is plugin load location, not necessarily every session's workspace; use event/session workspace metadata when documented, falling back only as the user-approved mapping allows.
- **H5. Async lifecycle:** Event stream must be cancelled on plugin unload; socket sends must be bounded and failures swallowed/logged, not allowed to crash plugin setup or event processing.
- **H6. Existing modifications:** Preserve pre-existing edits to `planning/GOALS.md` and `planning/RESEARCH_OPENCODE_INTEGRATION.md`.
- **H7. Shared test resources:** Use unique temporary socket paths per test run; run target tests serially.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 36.1.1 | Verify OpenCode API contract and implement thin lifecycle plugin | `.opencode/plugins/swayve.ts`, `integrations/opencode/package.json` (if required for test/runtime metadata) | — |
| 2 | 36.2.1 | Add mock-socket tests and integration setup/smoke-test docs | `integrations/opencode/sidecar.test.mjs` or equivalent test file, `integrations/opencode/README.md` | 36.1.1 |

## Task 36.1.1: Verify OpenCode API Contract and Implement Plugin

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — must edit:** `.opencode/plugins/swayve.ts`; `planning/RESEARCH_OPENCODE_INTEGRATION.md` only if first-party verification adds or corrects sourced facts.
- **Files in scope — can create:** `.development/` scratch only; no additional files without stopping for scope approval.
- **Depends on:** none.
- **Spec:** Verify the official server plugin API/event schemas and supported version. Implement a project-local plugin that subscribes to the minimum session/tool lifecycle events and sends valid Swayve registration/status IPC lines. Preserve thin-client architecture, deterministic identity, bounded best-effort socket behavior, and unload cleanup. Resolve ambiguities from the official contract before coding; do not invent event types.
- **Constraints (do not touch):** Rust crates, daemon protocol, existing integrations, root Makefile, unrelated planning files. Do not add third-party runtime/test dependencies. Do not use OpenCode SDK actions beyond event observation. Do not assume child-session ancestry.
- **Grep anchors/reference:** `integrations/antigravity/sidecar.mjs` (`resolveSocketPath`, `registerLine`, `statusLine`, `send`, `main`); `crates/swayve-types/src/lib.rs` (`AgentWindow`, `AgentIPCMessage`); ADRs 0004, 0006, 0009, 0012; official API sources linked in `planning/RESEARCH_OPENCODE_INTEGRATION.md`.
- **Acceptance:** Plugin type-checks/loads against the stated supported OpenCode plugin API version. Session creation emits a valid registration with agreed identity fields; documented lifecycle transitions emit correct statuses; cleanup cancels subscription; missing socket and failed socket writes do not throw into OpenCode. Targeted tests are added in Wave 2.

## Task 36.2.1: Mock-Socket Tests and Setup Documentation

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — must edit:** `integrations/opencode/README.md`; `integrations/opencode/sidecar.test.mjs` or equivalent test path established in Wave 1.
- **Files in scope — can create:** `.development/` scratch only.
- **Depends on:** 36.1.1.
- **Spec:** Add dependency-free Node tests using unique temporary Unix socket paths to assert registration/status payloads, identity mapping, failure behavior, and lifecycle cleanup. Document project-local installation/configuration, supported OpenCode version, independent test command, and a manual smoke-test checklist. Clarify unsupported/unverified parent-session mapping.
- **Constraints (do not touch):** Do not modify daemon/Rust code, existing integration behavior, root Makefile, or unrelated README sections. No live OpenCode installation is required for automated tests.
- **Grep anchors/reference:** `integrations/antigravity/sidecar.test.mjs` for Node mock-server patterns; `integrations/antigravity/README.md` for env/socket and setup docs; `integrations/pi/tests/swayve-client.test.ts` for socket assertions.
- **Acceptance:** `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` passes with all cases; tests leave no socket/temp artifacts; README setup and manual smoke test agree with plugin behavior and documented API verification date.
