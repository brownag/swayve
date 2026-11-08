# Orchestration 33: Antigravity sidecar against the Swayve socket contract (G25, G15)

Status: draft, awaiting user approval.
Base commit: `08a98d7` (main).
Source prompt: `planning/external/antigravity-sidecar.prompt.md`.

## Goal & Scope

**Goal.** Build the Antigravity sidecar in `integrations/antigravity/` as a small Node client that reports agent state to the Swayve daemon using only the documented socket contract. Done means the acceptance checks in Task 33.1.1 pass, and one `agent.register` line is confirmed in the daemon log from `make daemon-dry`.

**In scope.**
- Sidecar module and test file under `integrations/antigravity/` (G25, G15).
- Emit exactly three wire messages: `agent.register`, `agent.update_status`, `daemon.focus`.
- Parent id wiring for subagents.
- Correction of `integrations/antigravity/README.md` where it disagrees with the contract (env table, pid row, test command).

**Out of scope (deferred, not silently dropped).**
- Antigravity hook configuration (`.agents/hooks.json`, `~/.gemini/config/hooks.json`) that calls the sidecar. Not asked for. Add when a hook installer is planned.
- Any change to the daemon so it sets `SWAYVE_PARENT_ID`, `SWAYVE_AGENT_PID`, or other env vars. The daemon is not touched.
- Any edit outside `integrations/antigravity/`, including the root `.gitignore`.
- Resolving `conversationId` to a swayve id (see D4).
- The HUD consumer (`planning/external/hud-diff-consumer.prompt.md`, G53). Separate goal.
- Remote pushes. Local commits only.

**Verbatim constraints (from the input prompt).**
- "Edit only files under `integrations/antigravity/`. Do not edit other swayve files, such as `src/` or `planning/`. Do not push or run any remote-write git command. Local commits are allowed."
- "Use Node.js with the built-in `node:test` runner and no dependencies. Keep it to one module plus one test file."
- "Make the sidecar emit exactly the three messages above, with the parent id wired through."
- "Do not send any other method. Do not add an ack or a reply path."
- "Do not invent a parent when none exists. Send `null` for a root agent."
- "Without `XDG_RUNTIME_DIR` and `SWAYVE_SOCKET`, the sidecar must not guess a path. It should log and skip."

## Decision Log

- **D1 (2026-11-08).** Sidecar files use `.mjs`, not `.js`. Reason: `.gitignore` ignores `*.js`, so `.js` files would never be committed. Node 22 runs `.mjs` directly with no build step. Files: `sidecar.mjs`, `sidecar.test.mjs`.
- **D2 (2026-11-08).** Parent id comes from env `SWAYVE_PARENT_ID`. Unset or empty means `null` (root agent). No state file, no lookup by conversation. Same name as `swayve spawn --parent` (`src/cli.rs:569`).
- **D3 (2026-11-08).** Pid comes from env `SWAYVE_AGENT_PID` (positive integer). If unset or invalid, fall back to `process.ppid`. The fallback is unverified and the live check (D6) tests it. README records it as a fallback.
- **D4 (2026-11-08).** `agent_id` comes only from env `SWAYVE_AGENT_ID`. `conversationId` is not mapped and not sent. Missing `SWAYVE_AGENT_ID` means log one line and send nothing.
- **D5 (2026-11-08).** Missing socket path or failed write: log one line to stderr, exit 0. Status reporting is best-effort, so it must not fail an agent hook.
- **D6 (2026-11-08).** Acceptance includes a live check: run `make daemon-dry` from the repo root, point `SWAYVE_SOCKET` at its socket, send one register line, and confirm it in the daemon log. Required, not optional.
- **D7 (planner default, not asked).** Sidecar is a CLI with subcommands (`register`, `status <value>`, `focus`) plus exported pure builders used by the tests. One connection per message, written and closed. Each message is one line ending in `\n`. No ack is read.
- **D8 (planner default, not asked).** On closing, G25 and G15 move to Completed in `planning/GOALS.md`, in the closing step of this orchestration, after Task 33.1.1 is verified.

## Wire contract (what the code must produce)

Source: swayve `src/types.rs:49-85`, `src/types.rs:19-28`, and README Socket Contract. Corrections recorded in the research are already applied here.

- `agent.register`: `{"method":"agent.register","params":{"agent_id","agent_type":"antigravity","project_id","pid","app_id":"agent.<agent_id>","parent_agent_id": string|null}}`. The `id` key is omitted.
- `agent.update_status`: `{"method":"agent.update_status","params":{"agent_id","status"}}`. Status is one of `idle`, `thinking`, `executing_tool`, `awaiting_human_input`, `completed`, or `{"failed":"<reason>"}`. Nothing else.
- `daemon.focus`: `{"method":"daemon.focus"}`. No params.
- Socket path: `SWAYVE_SOCKET` if set and non-empty; else `$XDG_RUNTIME_DIR/swayve/swayve.sock` if `XDG_RUNTIME_DIR` is set and non-empty; else no path (log and skip, D5).
- Framing: one JSON object per line, terminated by `\n`. No reply is read.

## Wave Table

| Wave | Task | Repo | Files (must edit) | Depends on | Parallel-safe |
|---|---|---|---|---|---|
| 1 | 33.1.1 Sidecar module, tests, README | swayve | `integrations/antigravity/sidecar.mjs` (new), `integrations/antigravity/sidecar.test.mjs` (new), `integrations/antigravity/README.md` | none | single task |

Wave 1 is a single task. Splitting it would mean two workers editing the same README and the module's CLI surface, which is the coupling the wave rules forbid.

Wave gate (orchestrator): none of the repo's `make all` suites cover `integrations/antigravity/`. Gate = Task 33.1.1 acceptance, plus `make check` (cargo check, unchanged code) to confirm no Rust file was touched.

## Hazard Notes (from recon)

- H1. `.gitignore` line 5 is `*.js`. Using `.js` silently drops the files from git. Hence D1.
- H2. The daemon never sets `SWAYVE_PARENT_ID` (only `swayve spawn --parent` does, `src/cli.rs:568-570`). The README claim that "the daemon passes these down" is wrong for `SWAYVE_PARENT_ID` and unverified for the rest.
- H3. The daemon may fill a missing `parent_id` from `/proc` ancestors when a window is created (`src/daemon.rs:217-225`, `652-667`, on by default per `src/config.rs:80-82`). Sending `null` does not guarantee a null parent in state. The sidecar must not rely on either outcome.
- H4. The daemon stores an unregistered `parent_agent_id` as given and only logs it (`src/daemon.rs:140-158`). Send the full parent `agent_id`.
- H5. The daemon never replies to these messages. Do not read from the socket. Malformed or over-rate lines are dropped silently (ADR 0014, 0015).
- H6. `SWAYVE_SOCKET=""` counts as unset (`src/cli.rs:93`).
- H7. `XDG_RUNTIME_DIR` unset or empty makes the daemon refuse to start (`src/cli.rs:44-50`, ADR 0012). The sidecar mirrors this by skipping, not guessing.
- H8. Hook env vars, PIDs and parent ids are not documented by Antigravity (research). The README must say the launcher supplies them.
- H9. README line "eval \"$(swayve init-shell)\"" is outside this task's contract. Leave it, and note it in the report if it reads as Antigravity-specific.

## Task Specs

### Task 33.1.1: Antigravity sidecar module, test file, README corrections

- **id:** 33.1.1
- **repo:** swayve
- **Files in scope.**
  - Must edit: `integrations/antigravity/README.md`.
  - Can create: `integrations/antigravity/sidecar.mjs`, `integrations/antigravity/sidecar.test.mjs`.
  - Scratch: `.development/` only (gitignored).
  - Off-limits: every other path, including `src/`, `planning/`, `.gitignore`, `integrations/claude/`, `integrations/pi/`.
- **Depends on.** none.
- **Reference implementation.** `integrations/pi/src/swayve-client.ts` (socket write pattern, `SWAYVE_SOCKET` use). Read its send function only, not the whole file.
- **Anchors.**
  - Contract: `src/types.rs:49-85` (`AgentIPCMessage`, `RegisterParams`, `UpdateStatusParams`), `src/types.rs:19-28` (`AgentStatus`).
  - Socket path: `src/cli.rs:91-98`.
  - Test pins: `types::tests::agent_status_roundtrip` (`src/types.rs:93`), `daemon::tests::test_focus_ipc_roundtrip` (`src/daemon.rs:952`).
  - Spawn env: `src/cli.rs:565-570`.
  - README: the env-var table, the register example, and the "Example Shell Injection" section.
- **Spec.**
  1. `sidecar.mjs` exports pure builders that return a JSON string (no trailing newline) and do no I/O:
     - `registerLine({ agentId, projectId, pid, parentAgentId })` builds the `agent.register` object with `agent_type: "antigravity"` and `app_id: "agent.<agentId>"`. `parentAgentId` of `undefined`, `null`, or `""` becomes `null`.
     - `statusLine({ agentId, status })` builds `agent.update_status`. `status` is a string from the four unit variants, or the string `failed:<reason>`, which becomes `{"failed": "<reason>"}`. Any other value throws (never sent).
     - `focusLine()` returns `{"method":"daemon.focus"}`.
     - `resolveSocketPath(env)` applies the three-step rule in the contract above. Returns `null` when no path.
     - `resolveEnv(env, ppid)` reads `SWAYVE_AGENT_ID` (required), `SWAYVE_PROJECT_ID`, `SWAYVE_PARENT_ID` (empty means null), and `SWAYVE_AGENT_PID` (positive integer, else `ppid`). Missing `SWAYVE_AGENT_ID` returns `null`.
  2. `sidecar.mjs` also runs as a CLI when executed directly (`import.meta.url` check against `process.argv[1]`). Subcommands: `register`, `status <value>`, `focus`. Each resolves the socket path and env, logs one line to stderr and exits 0 when the socket path or `SWAYVE_AGENT_ID` is missing, then writes exactly one line with `socket.end(line + "\n")` via `node:net` `createConnection`. Socket errors log one line and exit 0. Do not read from the socket. Do not retry.
  3. `sidecar.test.mjs` uses `node:test` and `node:assert/strict`. Tests:
     - register for a subagent: `parent_agent_id` equals the given id.
     - register for a root agent: `parent_agent_id` is `null`, both when the env value is empty and when it is undefined.
     - status: `executing_tool` line equals `{"method":"agent.update_status","params":{"agent_id":"a-1","status":"executing_tool"}}`.
     - failed status: `failed:timeout` line parses to `status` `{"failed":"timeout"}`.
     - unknown status throws.
     - focus line equals `{"method":"daemon.focus"}` exactly.
     - `resolveSocketPath`: explicit path wins; empty `SWAYVE_SOCKET` falls through to XDG; both missing returns `null`.
     - `resolveEnv`: missing `SWAYVE_AGENT_ID` returns `null`; bad `SWAYVE_AGENT_PID` falls back to `ppid`.
     - Compare each builder's output by `JSON.parse` and `assert.deepEqual` against the contract object, plus one byte-level check per message: `JSON.stringify` of the expected object equals the builder output. Key order follows the contract; a byte match is required for status and focus.
  4. README corrections, in the same change:
     - Env table: state that `SWAYVE_PARENT_ID`, `SWAYVE_AGENT_PID` and `SWAYVE_AGENT_ID` are supplied by the launcher. The daemon does not set them. Add `SWAYVE_AGENT_PID` with the `process.ppid` fallback and mark the fallback unverified.
     - Register example: keep the JSON. Add a note that `parent_agent_id` is `null` for a root agent and `app_id` is `agent.<agent_id>`.
     - Add a "Run and test" section: `cd integrations/antigravity && node --test sidecar.test.mjs` and `node sidecar.mjs register` with the env vars, and the exit-0 behavior.
     - Note that `SWAYVE_PARENT_ID` is not set by the daemon even though the table previously implied otherwise.
     - Leave the Shell Injection section and the rest of the README as they are (H9).
- **Constraints (do not touch).**
  - No dependencies, no `package.json`, no `.js` files (H1).
  - No other message methods. No ack, no reply read.
  - No inventing a parent id.
  - No edits outside Files in scope.
- **Micro-plan limit.** The worker's TODO has at most 12 steps.
- **Acceptance (mechanical).**
  - `cd integrations/antigravity && node --test sidecar.test.mjs` exits 0, with every test passing. Record the pass count.
  - `cd integrations/antigravity && node --test sidecar.test.mjs 2>&1 | grep -i warn` prints nothing.
  - Missing-path check: `env -u SWAYVE_SOCKET -u XDG_RUNTIME_DIR node integrations/antigravity/sidecar.mjs focus; echo $?` prints one stderr line and `0`.
  - Live check (D6), run by the orchestrator: from the repo root, `make daemon-dry` in one terminal with `XDG_RUNTIME_DIR` set; in another, `SWAYVE_SOCKET=<daemon socket> SWAYVE_AGENT_ID=a-33-1 SWAYVE_PROJECT_ID=swayve SWAYVE_AGENT_PID=$$ node integrations/antigravity/sidecar.mjs register`; confirm the register line shows up in the daemon log. Record the exact log line in STATE_33.md. If the daemon log does not show it, stop and report; do not change code to fit.
  - `make check` exits 0, with no Rust file changed (`git diff --stat` shows only `integrations/antigravity/`).
- **Targeted test command.** `cd integrations/antigravity && node --test sidecar.test.mjs`.

## Closing

- After Task 33.1.1 is verified and committed (one commit, conventional format, no orchestration or wave numbers in the message), the orchestrator moves G25 and G15 to Completed in `planning/GOALS.md`, with a reason line naming the commit.
- Record the live check output in `planning/STATE_33.md`.
