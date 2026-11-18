# TODO

## Goal
Validate the Swayve OpenCode plugin against the installed OpenCode v2.3.0 runtime and active Swayve daemon, making only focused fixes needed for live registration and lifecycle reporting.

## Tasks

### 1. Prepare the smoke test
- [x] Confirm branch `feat/opencode-integration1`, inspect `git status`, and preserve any unexpected changes.
- [x] Confirm OpenCode v2.3.0, Sway, the Swayve daemon, and the active socket path.
- [x] Start `swayve status --watch --json` as the live observer.

### 2. Run and verify the live smoke test
- [ ] From the repository root, run `opencode run --standalone --model openai/gpt-6-luna "Read the first heading in README.md and report it. This is read-only; do not create or modify files."`
- [ ] Verify OpenCode loads `.opencode/plugins/swayve.ts` and Swayve receives a registration with the session ID, `agent_type=opencode`, workspace-basename project ID, plugin PID, and `app_id=agent.<session_id>`.
- [x] Investigate why the Swayve dashboard showed only `Idle` during the successful live turn; inspect read-only OpenCode logs/event timing and the live Swayve protocol/dashboard behavior before considering code changes.
- [ ] If a focused plugin defect is identified, make only the minimal approved fix and add/update regression coverage; if the issue is observer latency/refresh behavior, document that instead of changing lifecycle code.
- [ ] Repeat the harmless live turn only if needed to verify a diagnosis/fix; observe the resulting lifecycle statuses (`thinking`, `executing_tool`, `thinking`, and terminal status) and record what actually appears without inferring missing events. Do not implement parent-ID mapping.
- [ ] Stop the status observer, remove any temporary output, and confirm the repository has no unintended changes.

### 3. Make focused fixes only if the smoke test finds a defect
- [ ] Keep any required fix within `.opencode/plugins/swayve.ts`, `integrations/opencode/sidecar.test.mjs`, `integrations/opencode/README.md`, or sourced notes in `planning/RESEARCH_OPENCODE_INTEGRATION.md`.
- [ ] Run `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs`; if implementation files changed, run `make all`.
- [ ] Commit verified implementation changes locally in a focused commit; do not perform remote Git actions.

### 4. Close and archive the plan
- [ ] Record the observed runtime, registration fields, statuses, tests, and any unresolved issues in the archived plan.
- [ ] Remove the unapproved Orchestration 37 draft files (`planning/EXECUTION_PLAN_37.md`, `planning/PROMPT_TEMPLATE_37.md`, `planning/STATE_37.md`); they are superseded by this simpler plan.
- [ ] Move this completed plan to `planning/TODO_20261117-1.md` and commit planning changes separately from implementation.

## Notes
- Branch: `feat/opencode-integration1`; do not switch branches.
- A single harmless model turn is authorized and may incur provider usage. Do not change provider/auth settings, write files, or expose secrets.
- Any repeat provider-backed turn must be limited to verifying the lifecycle diagnosis/fix; ask before additional runs if the investigation requires broader usage.
- Parent-session mapping remains out of scope; record any observed `parentID` evidence for a future goal only.
- Keep Swayve integrations thin per ADR 0009. Do not change daemon/Rust code, other integrations, the root Makefile, or dependencies.
- Planning artifacts are tracked. The approved root TODO and its final archive use separate planning-only commits; implementation commits remain separate. No push, pull request, or other remote action is authorized.
- The existing focused OpenCode mock-socket suite passed 4/4 and `make all` passed after Orchestration 36; see `planning/STATE_36.md`.
- Initial default-model attempt returned OpenCode Models `Insufficient balance` before reading the README. The session was nevertheless registered (`opencode-1-54955`, project `swayve`, app ID `agent.opencode-1-54955`, observed idle); the user authorized retrying with `openai/gpt-6-luna`. No provider or authentication settings will be changed.
- Investigation found that `swayve status --watch --json` is not JSON/event history: watch mode calls the dashboard subscriber and repeatedly clears/renders status snapshots. A second approved read-only turn observed through `swayve status --watch --diff` still showed only Idle. OpenCode's default local log confirms the plugin file loads but does not include the subscribed session lifecycle events; no plugin status updates appeared in the Swayve delta output. This points to a likely mismatch between the plugin's mocked event names/payload assumptions and the installed runtime, but does not yet prove the exact cause. No implementation changes made; further runtime event tracing or a scoped plugin fix needs a follow-up approved plan.

## Approved Follow-up: OpenCode v2.3.2 Event Payload Fix

### Goal
Fix the OpenCode plugin's lifecycle extraction to use the observed v2.3.2 `event.data` payload, cover it with mock-socket tests, and document the tested contract without requiring another live model turn.

### Tasks
- [x] Read `event.data.sessionID` for registration and lifecycle status events.
- [x] Derive the project basename from `session.created`'s `event.data.location.directory`, falling back to the plugin load directory.
- [x] Preserve current status mapping, best-effort socket behavior, cleanup, and unsupported parent mapping.
- [x] Update mock events/assertions to use the observed v2.3.2 `data` payload shape; cover registration, lifecycle statuses, missing identity/location fallback, duplicate creation, cleanup, and unavailable socket behavior.
- [x] Update the OpenCode README to state the tested v2.3.2 payload contract without claiming broader compatibility.
- [x] Run `node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs` and `git diff --check`.

### Notes
- The follow-up planning choice is tracked separately from implementation. Existing unrelated modification to `planning/RESEARCH_OPENCODE_INTEGRATION.md` must remain untouched and unstaged.
- Scope is limited to `.opencode/plugins/swayve.ts`, `integrations/opencode/sidecar.test.mjs`, and `integrations/opencode/README.md`, plus this plan.
- No second live model turn is required or authorized by this follow-up. The prior v2.3.2 probe observed IDs under `data.sessionID` and successful status-message writes; it did not prove daemon state application.
- Keep parent-session mapping unsupported; do not change daemon/Rust code, dependencies, root Makefile, provider/auth configuration, or other integrations. No remote Git operations.
