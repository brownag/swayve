# Execution Plan 40: OpenCode Session Discovery and Sway Window Association (G93)

Target repo: `/home/andrew/workspace/swayve`
Target branch: `feat/opencode-integration1`
Recon base: `planning/GOALS.md`, `planning/STATE_39.md`, OpenCode v2.3.2 plugin/API documentation, current Swayve integration and daemon code.

## Goal & Scope

**Goal.** Every project-scoped OpenCode session returned by session enumeration, including sessions that existed before the plugin loaded, is represented in Swayve and receives lifecycle updates. A session is associated with its correct Sway window agent only when a stable identity proves the match. The HUD makes that association explicit without conflating it with OpenCode's own parent-session hierarchy. Missing or ambiguous identity never results in a guessed association. Sessions are retired only on authoritative evidence.

**Outcomes and acceptance evidence.**
- Existing and newly created project sessions are represented independently and receive lifecycle updates.
- Sessions in multiple tabs in one OpenCode window, and sessions across multiple OpenCode windows, are mapped only to their correct window agents.
- A shared project directory is never used as window-identity evidence.
- Missing or ambiguous identity leaves a session explicitly unassociated.
- Deleted sessions are retired only when OpenCode provides a verified deletion signal; no inactivity timeout or inferred deletion is introduced.
- Automated tests cover enumeration/pagination, pre-existing and new sessions, multi-session/multi-window association, ambiguous/missing identity, lifecycle statuses, and safe retirement.
- A user-assisted Sway/OpenCode check demonstrates visible HUD association and lifecycle changes. The user explicitly identifies or triggers the sessions/actions; no model turn is run by the agent without separate, specific approval.

**In scope.**
- Verify OpenCode v2.3.2 session enumeration, pagination, event timing/replay behavior, session deletion events, and the process/window identity exposed to a server plugin.
- Verify Swayve's exact `app_id` binding, reported PID/process ancestry, re-registration behavior, existing `parent_agent_id` effects, and HUD representation.
- Add a dedicated optional session-to-window association rather than overloading `parent_agent_id`, unless source/runtime evidence proves the existing field has precisely the required semantics and no unwanted layout/focus effects.
- Add a narrow, validated way to update/clear that association and retire a session agent on authoritative deletion, preserving unrelated window agents.
- Enumerate the complete project-scoped session list (including historical sessions) and keep it synchronized with verified create/delete events.
- Add tests, update OpenCode integration documentation, and perform gated user-assisted live validation.

**Out of scope.**
- Treating project-directory equality, titles, or coincidental PIDs as proof of a window association.
- Changing unrelated integrations or lifecycle status mapping, adding dependencies, or redesigning the HUD beyond making the relationship explicit.
- Mapping OpenCode's `parentID` to Swayve `parent_agent_id`; this remains G90 and is semantically separate from a hosting-window association.
- Automatically prompting a model, changing provider/auth settings, writing files through OpenCode, closing sessions/windows, or operating on unselected user sessions.
- Remote Git operations.

**Verbatim user constraints and requirements.**
- “Determine whether OpenCode emits `session.created` for a session that already exists when the plugin loads.”
- “Check whether the plugin can discover the active session during setup, or whether it must associate sessions through another event.”
- “Test an existing first-tab session and sessions created after startup, across multiple tabs in one window.”
- “Ensure every active session gets a Swayve agent and lifecycle updates, regardless of whether the session predates plugin initialization.”
- “Do not infer that all tabs or sessions belong to one window solely because they share a project directory.”
- The user chose to enumerate **all project sessions** returned by OpenCode, rather than only sessions confirmed to be open tabs.
- The user approved a **safe retirement path** for deleted/no-longer-relevant sessions; retirement must be based on authoritative evidence, not inactivity.
- The user approved focused Swayve IPC/state/HUD changes if the current contract cannot safely represent the relationship.
- The user chose to prefer an **explicit association** over `parent_agent_id` unless its current semantics are proven safe.
- User-assisted live validation may observe an existing session and sessions the user explicitly identifies or creates. The user may trigger a harmless turn/tool action; no such run occurs during planning.

## Recon

- Current branch is `feat/opencode-integration1`. `planning/GOALS.md` and `planning/RESEARCH_OPENCODE_INTEGRATION.md` are already modified; preserve the research-note edit and do not stage it unintentionally. No remote Git operations.
- Existing planning artifacts are tracked and prior plans use separate `docs(planning):` commits. User confirmed that Orchestration 40 artifacts should be tracked.
- Current OpenCode runtime is v2.3.2. First-party plugin context exposes `ctx.session.list()` with pagination. Official plugin docs state `ctx.location` is the plugin instance location, not every session's location. The public event subscription is a stream; documentation/source must be checked for replay/backfill behavior rather than assumed.
- The current plugin (`.opencode/plugins/swayve.ts`, 84 lines) reads `event.data.sessionID`, registers only on `session.created`, and reports its own `process.pid`. It has no setup-time session enumeration, window association, or unregister message.
- The focused OpenCode mock-socket test passed 4/4: `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs`. Node emitted the existing harmless `MODULE_TYPELESS_PACKAGE_JSON` warning. This suite is not included in `make test`.
- The Swayve daemon binds a window to an agent by exact `app_id` (`register_agent_window`, `crates/swayve/src/daemon.rs`); Sway window events expose `app_id` and optional PID (`crates/swayve/src/sway.rs`). Optional process-tree auditing can infer `parent_id` by walking `/proc`, but this does not prove session-to-window identity.
- `parent_id` drives daemon focus/layout hierarchy (`crates/swayve/src/daemon.rs`, `crates/swayve/src/layout.rs`). `AgentWindow` has no distinct hosting-window association. The HUD renders a flat agent list (`crates/swayve-hud/src/client.rs`); it does not currently show parent relationships.
- The current IPC has register, update-status, focus, and subscribe messages; there is no agent-unregister/retirement message. Diff frames already support removed IDs.
- ADRs 0003, 0004, 0006, 0007, 0008, 0009, and 0012 are relevant. In particular, preserve daemon ownership (0003), newline-delimited JSON (0004), exact app-ID identity (0006), parent hierarchy semantics (0007), and thin socket integrations (0009). No ADR is assumed superseded; if implementation requires superseding one, stop for an explicit decision and name its ID.
- Full baseline `make all` passed on 2026-11-18: workspace check, clippy, Rust tests (90 passed, 2 ignored; integration/golden/HUD/types suites passed), Pi tests (8/8), Antigravity tests (14/14), Claude hook tests, and release build. `make all` does not include OpenCode tests; their focused baseline is recorded above.
- Key files and sizes: `.opencode/plugins/swayve.ts` (84), `integrations/opencode/sidecar.test.mjs` (217), `integrations/opencode/README.md` (60), `crates/swayve/src/daemon.rs` (1774), `crates/swayve/src/sway.rs` (232), `crates/swayve-types/src/lib.rs` (current shared wire/domain types), `crates/swayve-hud/src/client.rs` (HUD renderer), and `crates/swayve-hud/src/fold.rs` (diff folding).
- The HUD crate has additional local guidance in `crates/swayve-hud/AGENTS.md`; read it before any HUD implementation.

## Decision Log

- **D1 (2026-11-18): Enumerate all project sessions.** At setup, enumerate the complete project-scoped OpenCode session list, including persisted/historical sessions; do not restrict to tabs proven open.
- **D2 (2026-11-18): Safe retirement.** Add retirement only on authoritative deletion evidence; do not infer deletion from inactivity or project changes.
- **D3 (2026-11-18): Association model.** Prefer a dedicated session-to-window association. Reuse `parent_agent_id` only if evidence proves its semantics and layout/focus effects are correct for this distinct relationship.
- **D4 (2026-11-18): Conditional daemon scope.** Focused daemon IPC/state/HUD changes are allowed if needed to represent a proven association or retirement safely; unrelated changes remain out of scope.
- **D5 (2026-11-18): Live acceptance.** Include user-assisted validation of an existing first-tab session and sessions the user explicitly identifies or creates. The user, not the agent, triggers any provider-backed turn/tool action unless separately authorized. Do not manipulate unselected sessions/windows.
- **D6 (2026-11-18): Planning artifacts.** Track the plan, prompt, and state files; keep planning-only commits separate from implementation. No remote operations.
- **D7 (2026-11-18): Feature branch.** Stay on `feat/opencode-integration1`; preserve pre-existing research-note changes.
- **D8 (2026-11-18): Discovery gate.** Wave 1 must establish the exact session discovery, event, deletion, and window identity contract. If no safe exact association source exists, stop before implementing a guessed mapping and return the evidence for a user decision.
- **D9 (2026-11-18): Identity-options investigation.** After 40.1.1 found no identity in the OpenCode session API, investigate viable external/explicit mechanisms within this orchestration before deciding whether implementation can proceed. No specific mechanism is approved yet.
- **D10 (2026-11-18): Explicit user-attested binding approved.** The user approved pursuing a deliberate bind that selects the exact OpenCode session and Swayve window-agent identity. No heuristic association is allowed; stale/unknown endpoints remain unassociated and require a fresh bind after window recreation. This authorizes scoped implementation of the bind contract, not live validation or broader runtime actions.

## Hazard Notes

- Do not equate plugin PID, OpenCode server PID, session ID, Sway window PID, `app_id`, or project directory. Record what each value identifies and how it is observed.
- OpenCode session enumeration may include historical sessions. This is intentional per D1; only an authoritative deletion event may retire one. Confirm pagination, scope, and deletion semantics.
- A stream subscription may not replay events that happened before setup. Register known sessions from enumeration and avoid double registration when enumeration races with create events; re-registration must preserve status/window binding as documented by G79.
- A successful socket write does not prove daemon application. For live checks correlate plugin output with daemon state and HUD changes.
- An OpenCode server may be shared across multiple Sway windows. Process ancestry/PID matching is acceptable only if the exact one-to-one or many-to-one relation is proven; otherwise leave sessions unassociated.
- A session may be registered before its window agent. Association updates must be idempotent and must not attach to a target that is absent or ambiguous.
- Retiring a session agent must not remove its hosting window agent or another session. Window destruction must not leave a dangling association.
- Existing project files may contain sensitive data. Runtime validation should use only user-selected sessions; do not log raw session IDs or payloads in durable planning reports.
- Run full suites serially at wave end; integration tests use local sockets and environment state.
- If a task discovers the wire/domain design conflicts with an accepted ADR, stop and request an explicit ADR decision before implementation.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 40.1.1 | Establish OpenCode session discovery/event/deletion and Sway window identity facts; recommend a safe contract | Read-only source files; `planning/STATE_40.md` | — |
| 1 | 40.1.2 | Investigate explicit/configuration-based and process/server-based ways to prove session-to-Sway-window identity; recommend a verifiable contract | Read-only sources/docs; `planning/STATE_40.md` | 40.1.1 |
| 2 | 40.2.1 | Add an explicit, validated Swayve session-to-window relation and authoritative session retirement | `crates/swayve-types/src/lib.rs`, `crates/swayve/src/daemon.rs`, focused Rust tests | 40.1.2 and D10 |
| 3 | 40.3.1 | Render the explicit relation in the HUD without using parent hierarchy | `crates/swayve-hud/src/client.rs`, HUD tests | 40.2.1 |
| 3 | 40.3.2 | Enumerate project sessions, handle create/delete races, accept only explicit user-attested binds, and report lifecycle | `.opencode/plugins/swayve.ts`, `integrations/opencode/sidecar.test.mjs`, `integrations/opencode/README.md` | 40.1.2, 40.2.1 and D10 |
| 4 | 40.4.1 | Verify the first-tab, later-tab, multi-window, lifecycle, ambiguity, and retirement behavior with user-assisted live evidence; run final gates | `planning/STATE_40.md` | 40.3.1, 40.3.2 |

Wave 3 tasks have disjoint implementation files and may run in parallel after Wave 2. Task 40.1.2 is a discovered Wave 1 task that investigates identity options after the source-only identity gap found by 40.1.1. D10 approves the explicit user-attested binding contract; no automatic mapping is permitted. Task 40.4.1 still requires separate user-assisted live-validation confirmation and user-triggered actions.

## Task 40.1.1: Establish Session and Window Identity Contract

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — read only:** OpenCode v2.3.2 first-party source/API/docs; `.opencode/plugins/swayve.ts`; `integrations/opencode/README.md`; `planning/RESEARCH_OPENCODE_INTEGRATION.md`; `planning/STATE_39.md`; `crates/swayve/src/{sway.rs,daemon.rs,layout.rs,config.rs}`; `crates/swayve-types/src/lib.rs`; `crates/swayve-hud/src/{client.rs,fold.rs}`; relevant ADRs.
- **Files in scope — may update:** `planning/STATE_40.md` only, for the task report.
- **Depends on:** none.
- **Spec:** Inspect the pinned v2.3.2 plugin context and session APIs. Establish pagination and directory filtering for `ctx.session.list`, whether it includes child sessions, event names/payloads for create/delete and lifecycle, whether subscription replays/backfills events, what session records expose, and whether setup can enumerate existing sessions without causing model/tool actions. Separately trace Sway window identity (`con_id`, `app_id`, PID), registered agent identity, process-tree auditing and its limitations, re-registration semantics, IPC/HUD schemas, and existing removal behavior. Do not inspect live user session contents or run an OpenCode prompt. Produce a concise, source-cited report in `STATE_40.md` that distinguishes proven facts from unknowns and recommends a safe association/deletion contract. If a stable exact mapping or authoritative retirement signal is unavailable, name the precise blocker and stop the implementation waves pending user decision.
- **Constraints (do not touch):** Implementation files, user session data, OpenCode/Sway live state, provider/auth settings, unrelated workspace files, or remotes. Do not infer a window from shared project directory.
- **Grep anchors/reference:** `ctx.session.list`, `Session.Info`, `session.created`, `session.deleted`; plugin `eventTypes`, `setup`, `send`; daemon `register_agent_window`, `apply_process_tree_auditing`, `on_window_destroyed`, `update_status`; `AgentWindow`, `AgentIPCMessage`; HUD `render_hud`; ADRs 0003/0004/0006/0007/0009/0012.
- **Acceptance:** `STATE_40.md` records v2.3.2 source locations/links, setup enumeration and event replay/deletion facts, identity evidence and unknowns, race/retirement implications, and a recommendation with no guessed mapping. No implementation/runtime changes. The existing focused OpenCode test remains green: `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` (4/4 at planning baseline).

## Task 40.1.2: Investigate Verifiable Session-to-Window Identity Options

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — read only:** OpenCode v2.3.2 first-party source/API/docs; Sway IPC/window identity docs and relevant local Swayve integration/config/daemon code; `.opencode/plugins/swayve.ts`; `crates/swayve/src/{sway.rs,daemon.rs,config.rs}`; `crates/swayve-types/src/lib.rs`; relevant ADRs.
- **Files in scope — may update:** `planning/STATE_40.md` only.
- **Depends on:** 40.1.1.
- **Spec:** Investigate viable identity mechanisms without presuming OpenCode must expose Sway identity. At minimum assess (a) explicit user-provided per-window/session binding or launch metadata and its lifecycle/ambiguity handling, and (b) mapping OpenCode server/process identity to Sway windows, including shared-server, multiple-window, tabs, PID reuse, and process ancestry cases. Consider any other evidence-backed option found in source/docs. For each option, state required signals, how they are obtained, whether they distinguish multiple sessions/windows, failure/ambiguity behavior, implementation footprint, and a concrete falsifiable validation method. Recommend the narrowest safe contract or conclude none is currently viable. Do not inspect session message content, operate live OpenCode/Sway, run prompts, or implement an option. User decision is required before implementation.
- **Constraints (do not touch):** Implementation files, user session data, live OpenCode/Sway state, provider/auth settings, unrelated workspace files, or remotes. Do not claim project directory, title, session ID, plugin PID, or server PID alone proves a window association.
- **Grep anchors/reference:** v2.3.2 `ctx.session.list`, session records and plugin context; Sway `get_tree`, `con_id`, `app_id`, PID; Swayve `register_agent_window`, `apply_process_tree_auditing`, config and launcher integration; findings in `STATE_40.md`; ADRs 0003/0004/0006/0007/0009/0012.
- **Acceptance:** `STATE_40.md` compares at least the two required option classes, identifies evidence and limits, gives a testable recommendation and explicit decision point, and keeps Waves 2–4 blocked pending user approval. No implementation/runtime changes.

## Task 40.2.1: Add Explicit Swayve Association and Safe Retirement

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit:** `crates/swayve-types/src/lib.rs`; `crates/swayve/src/daemon.rs`; focused `crates/swayve/tests/` integration coverage and the relevant inline tests in `daemon.rs`/`swayve-types/src/lib.rs`. Update existing `AgentWindow` struct-literal fixtures only in `crates/swayve/src/cli.rs` and `crates/swayve-hud/src/{client.rs,fold.rs}` to supply the new optional/default field; no CLI/HUD behavior changes.
- **Files in scope — may create:** No new files unless the existing test layout requires one; if so, report before expanding scope.
- **Depends on:** Authoritative-retirement evidence in 40.1.1, explicit-binding recommendation in 40.1.2, and user approval recorded as D10.
- **Spec:** Add a distinct optional association from an OpenCode session agent to its verified Sway window agent; do not repurpose `parent_agent_id`. The association is created only after a deliberate user-attested bind naming both exact endpoints; no automatic heuristic matching is in scope. Add the smallest newline-delimited IPC operation(s) needed to set/clear the association idempotently and retire a session agent on verified deletion. Track enough authoritative agent-kind/origin metadata for the daemon to reject retirement of window agents; do not rely on session-ID/app-ID string guesses. Validate target and source semantics so unknown IDs and ambiguous targets cannot corrupt state. Clear dangling associations when a window agent is destroyed while preserving the session; a recreated window requires a fresh explicit bind. Broadcast all changes through existing diff/full subscription paths. Preserve unrelated agent re-registration, status, focus, and layout behavior.
- **Constraints (do not touch):** OpenCode plugin/integration files, HUD rendering, unrelated integrations, root Makefile, dependencies, provider/auth configuration, or remotes. Do not change `parent_agent_id` semantics. If an accepted ADR must change, stop for explicit approval naming the ADR.
- **Grep anchors/reference:** `AgentWindow`, `AgentIPCMessage`, `RegisterParams`; daemon `register_agent`, `on_window_destroyed`, `update_status`, `handle_client`, broadcast/subscriber helpers; G79 re-registration tests; existing `tests/integration_tests.rs` socket patterns; ADRs 0003/0004/0006/0007/0008/0009/0012.
- **Acceptance:** Rust unit/socket tests prove user-requested explicit association set/clear (no inference), unknown/stale-target rejection, retirement allowed only for explicitly marked OpenCode session agents, no removal of a window agent, dangling-link cleanup after window destruction with no automatic reassignment, idempotence, and diff/full subscriber updates. Run `cargo test -p swayve-types` and `cargo test -p swayve` (including integration tests). `git diff --check` passes.

## Task 40.3.1: Show Session-to-Window Association in HUD

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit:** `crates/swayve-hud/src/client.rs` and its inline tests.
- **Depends on:** 40.2.1.
- **Spec:** Render the explicit session-to-window association in each relevant HUD row using a clear, stable window-agent identifier. Keep current status/project/app-id display and sorting unless a minimal change is required for clarity. Do not use or visually imply `parent_id` as the hosting-window relation. Unassociated sessions must render as unassociated rather than guessed. Follow `crates/swayve-hud/AGENTS.md`.
- **Constraints (do not touch):** Shared protocol/types, daemon, plugin integration, fold logic, unrelated HUD behavior, dependencies, or remotes.
- **Grep anchors/reference:** `render_hud`, `test_render_hud_sorted_agents`, HUD `AgentWindow` fixtures; `crates/swayve-hud/AGENTS.md`.
- **Acceptance:** HUD tests prove correct explicit relation display for multiple sessions pointing at distinct/same window agents and no link when association is absent; `cargo test -p swayve-hud` passes; `git diff --check` passes.

## Task 40.3.2: Enumerate and Report OpenCode Sessions Safely

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — may edit:** `.opencode/plugins/swayve.ts`; `integrations/opencode/sidecar.test.mjs`; `integrations/opencode/README.md`.
- **Depends on:** 40.1.1 and 40.2.1.
- **Spec:** At plugin setup, enumerate every project-scoped session using the verified paginated API and register each without prompting, executing tools, or reading message contents. Subscribe to verified lifecycle/create/delete events and make enumeration/event races idempotent. Do not automatically resolve sessions to windows. Provide/use only the explicitly user-attested bind flow from D10, naming the exact session and window-agent endpoints; absent a deliberate bind, report no association. Never match by project directory, title, or PID. Retire only on the authoritative deletion event established in 40.1.1. Preserve best-effort socket behavior and current per-session status mapping. Update tests for pagination, pre-existing/new sessions, explicit binds, create/list races, unassociated sessions, status events, deletion, cleanup, and socket failures. Document the tested runtime/version boundary and limitations.
- **Constraints (do not touch):** Rust/HUD files, other integrations, root Makefile, dependencies, provider/auth settings, OpenCode message history/content, or remotes. Do not record raw session IDs/payloads in durable logs or reports.
- **Grep anchors/reference:** plugin `eventTypes`, `setup`, event iterator, `send`, registration set; tests `eventStream`, `mockSocket`, `startPlugin`; integration README API support/manual smoke; v2.3.2 findings in `STATE_40.md`.
- **Acceptance:** `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` passes all focused tests; coverage includes pre-existing and newly created sessions, complete pagination, distinct exact window links, safe no-link cases, no project-directory matching, authoritative removal, and lifecycle statuses. README claims no broader compatibility than tested. `git diff --check` passes.

## Task 40.4.1: User-Assisted Live Acceptance and Final Gate

- **Repo:** `/home/andrew/workspace/swayve`
- **Files in scope — read only:** live Sway/OpenCode state, relevant integration/HUD behavior, `Makefile`.
- **Files in scope — may update:** `planning/STATE_40.md` only, with sanitized results.
- **Depends on:** 40.3.1 and 40.3.2.
- **Spec:** Confirm the runtime/version, selected OpenCode project and active daemon/socket with the user. Observe the existing first-tab session and user-identified/created sessions across multiple tabs in one window and multiple windows. The user triggers any harmless turn/tool action; do not initiate a provider-backed turn, write workspace files, inspect unselected session content, or close/move windows. Correlate plugin registration/association, daemon state, and HUD. Verify distinct correct associations, lifecycle status updates for sessions present before and after plugin startup, no association based solely on common project directory, and retirement only after the verified deletion signal. If any step requires additional access/action, stop and ask. Run final `make all` serially and the focused OpenCode test; record sanitized evidence, exact commands, pass counts, deviations, and blockers in `STATE_40.md`.
- **Constraints (do not touch):** User data/session contents beyond the selected validation evidence, provider/auth configuration, implementation files, unrelated workspace files, or remotes. No agent-initiated model turns or file-writing prompts.
- **Grep anchors/reference:** `swayve status --watch --diff`; plugin association/session events; HUD session association display; task reports in `STATE_40.md`.
- **Acceptance:** User-assisted evidence demonstrates the requested first-tab, new-session, multi-tab, multi-window, lifecycle, ambiguity, and retirement outcomes; `make all` and the focused OpenCode test pass. If the runtime cannot provide safe identity or deletion evidence, report the exact boundary and do not claim acceptance.
