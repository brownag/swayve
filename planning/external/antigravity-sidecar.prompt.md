# Agent prompt: Antigravity sidecar against the Swayve socket contract (G25, G15)

You are working in the Antigravity sidecar repository, which is outside the swayve repo. Its location is `{{SIDECAR_REPO}}`. Do not edit anything in the swayve repo. Do not push or run any remote-write git command.

## Goal
The sidecar reports Antigravity agent state to the Swayve daemon, using only the documented socket contract. Done means every item in "Acceptance" passes against a live `swayve daemon` or `make daemon-dry` (run from the swayve repo, read-only).

## Socket contract (verified against swayve `src/types.rs` and `README.md` Socket Contract)
- Socket path: `$SWAYVE_SOCKET` if set and non-empty (an empty value counts as unset, per `src/cli.rs:93`); otherwise `$XDG_RUNTIME_DIR/swayve/swayve.sock`. Without `XDG_RUNTIME_DIR` and `SWAYVE_SOCKET`, the sidecar must not guess a path. It should log and skip.
- Framing: one JSON object per line, terminated by `\n`. It is not JSON-RPC 2.0. There is no envelope and no reply.
- The daemon never replies to these messages. Treat the connection as fire-and-forget.
- Env vars passed to the sidecar: `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, `SWAYVE_PROJECT_ID`, and `SWAYVE_PARENT_ID` (see the gap below).

## Messages to implement (exact `method` strings)
1. `agent.register`: `{"method":"agent.register","params":{"agent_id":"...","agent_type":"antigravity","project_id":"...","pid":N,"app_id":"agent.<agent_id>","parent_agent_id":"..."|null}}`. Send on startup of each agent or subagent.
2. `agent.update_status`: `{"method":"agent.update_status","params":{"agent_id":"...","status":"..."}}`. Accepted statuses are the `AgentStatus` serde variants in `src/types.rs:21-27`: `idle`, `thinking`, `executing_tool`, `awaiting_human_input`, `completed`, and `{"failed":"<reason>"}`. Use only the ones you need; do not invent others.
3. `daemon.focus`: `{"method":"daemon.focus"}`. Use for hierarchical focus cycling.

Do not send any other method. Do not add an ack or a reply path.

## Known gap to resolve
The daemon does not set `SWAYVE_PARENT_ID` for agents the sidecar starts. (`swayve spawn --parent` does set it, `src/cli.rs:568-570`, but that path is not used here.) For a subagent, the sidecar must supply `parent_agent_id` on `agent.register` itself, from the spawning parent's `agent_id`. Do not invent a parent when none exists. Send `null` for a root agent. Note: the daemon may still fill `parent_id` from the process tree when the window is created (`src/daemon.rs:217-225`, on by default per `src/config.rs:80-82`), so a `null` sent here does not guarantee a null parent in daemon state. Do not rely on that either way.

## Work
1. Read the sidecar's current IPC code. Do not assume the message shapes above match it. Record each difference.
2. Make the sidecar emit exactly the three messages above, with the parent id wired through.
3. Add one test per message shape. Use the smallest runnable check: a unit test that serializes the message and compares it to the JSON above.
4. Run the sidecar's own test command. Report the exact command and the pass count.

## Acceptance
- Register sends `parent_agent_id` for a subagent, and `null` for a root agent. A test covers both.
- Status and focus lines match the contract byte for byte, except key order.
- A missing socket path logs one line and does not panic or spin.
- The sidecar's own test command passes, with no new warnings.
- Optional live check, if the swayve daemon is available: start `make daemon-dry` from the swayve repo, point `SWAYVE_SOCKET` at its socket, and confirm the register line appears in the daemon log.

## Report (15 lines or fewer)
- Status: done or blocked.
- Files changed, with line ranges.
- Test command and results.
- Differences found between the sidecar's old IPC code and the contract.
- Anything you could not verify, such as the parent-id source in the spawning code.

## Supporting implementation facts (from research)

Facts below are the ones this sidecar needs. Citations are to the swayve repo at `bee60cd`.

### Wire format
| Message | Field | Type | Required | Source |
|---|---|---|---|---|
| `agent.register` | `method` | `"agent.register"` | yes | `src/types.rs:49-56` |
| | `params.agent_id` | string | yes | `src/types.rs:73` |
| | `params.agent_type` | string | yes | `src/types.rs:74` |
| | `params.project_id` | string | yes | `src/types.rs:75` |
| | `params.pid` | u32 | yes | `src/types.rs:76` |
| | `params.app_id` | string, must be `agent.<agent_id>` | yes | `src/types.rs:77` |
| | `params.parent_agent_id` | string or null | optional; missing and `null` both mean none | `src/types.rs:78` |
| `agent.update_status` | `params.agent_id`, `params.status` | string, AgentStatus | yes (`state` is accepted as an alias) | `src/types.rs:83-85` |
| `daemon.focus` | none | no params | n/a | `src/types.rs:60-62` |

- Extra keys are ignored. Unknown methods are logged, not answered.
- `"id"` on the envelope is ignored by the daemon, so omit it.

### AgentStatus values
`"idle"`, `"thinking"`, `"executing_tool"`, `"awaiting_human_input"`, `"completed"`, and `{"failed":"<message>"}` (object form). Source: `src/types.rs:19-28`. Pinned by `types::tests::agent_status_roundtrip` (`src/types.rs:93-105`). `"bogus"` is rejected.

### Socket and framing
- Path: `SWAYVE_SOCKET`, else `$XDG_RUNTIME_DIR/swayve/swayve.sock` (`src/cli.rs:91-98`). If `XDG_RUNTIME_DIR` is unset or empty, that is an error (`src/cli.rs:44-50`).
- Empty `SWAYVE_SOCKET=""` is accepted by code as a path (`src/cli.rs:93`). Treat empty as unset.
- One JSON object per line. Blank lines are skipped (`src/daemon.rs:469-471`). Non-UTF-8 lines are skipped (`src/daemon.rs:465-467`, ADR 0014).
- Malformed JSON is logged (`src/daemon.rs:496`) and the connection stays open.
- Rate limit is checked before parse (`src/daemon.rs:461-463`). Defaults: 100 lines/s sustained, burst 200 (`src/ratelimit.rs:7-11`). Over-limit lines are dropped with no reply (ADR 0016).
- No reply is ever written to a non-subscribe connection (`src/daemon.rs:478-489`). Fire and forget.

### Corrections that apply to this prompt
1. The status list above is the full set. The prompt listed three.
2. `SWAYVE_PARENT_ID` is set only by `swayve spawn --parent` (`src/cli.rs:568-570`). The daemon never sets it. An Antigravity process not launched by `swayve spawn` will not have it.
3. Sending `parent_agent_id: null` does not guarantee a null parent in daemon state. When a window with an `agent.*` app_id is created and `process_tree_audit_enabled` is on (default, `src/config.rs:80-82`), the daemon fills a missing `parent_id` from `/proc` ancestors (`src/daemon.rs:217-225`, `652-667`).
4. The daemon stores an unregistered `parent_agent_id` as given and only logs it (`src/daemon.rs:140-158`). Send the full `agent_id` of the parent.

### Antigravity first-party findings
Sources (official): https://antigravity.google/docs/sdk/overview and https://antigravity.google/docs/hooks?tab=ide.
- Hook config: workspace `.agents/hooks.json` at project root; global `~/.gemini/config/hooks.json`. `~/.gemini/antigravity-cli/settings.json` hooks are not loaded.
- Events: `SessionStart`, `PreToolUse`, `PostToolUse`, `PreInvocation`, `PostInvocation`, `Stop`. Handler: `type: "command"`, required `command`, `timeout` in seconds (default 30).
- Hook I/O: JSON on stdin and stdout, camelCase fields. Named fields include `conversationId`, `workspacePaths`, `transcriptPath`, `artifactDirectoryPath`, `modelName`, `lastUserInput`.
- Subagent tool names are matchable: `invoke_subagent`, `manage_subagents`, `send_message`, `define_subagent`.

Not found in primary source (do not assume these):
- Environment variables passed to hook commands.
- Any PID given to hooks or SDK agents.
- Parent or subagent ids given to hooks. Only `conversationId`, `stepIdx`, `invocationNum`, `executionNum` are named.
- Whether hooks fire for tool calls inside a subagent.
- An official way for an external process to subscribe to session events.

Implication: the sidecar must get `pid`, `agent_id` and `parent_agent_id` from its own launcher or config. `conversationId` (present on `SessionStart` and `Stop`) can be used for correlation, but no documented mapping to swayve ids exists. Third-party leads (Medium, docs.rs, pub.dev, `$GEMINI_SESSION_ID` and similar) are unverified. Do not use them.

### Open questions for the sidecar
- Do `SessionStart` and `Stop` fire for subagents, or only the root agent? Not stated.
- What is the parent-id source for a subagent? Not found. The launcher must know it.
- Should `conversationId` map to `agent_id`? Neither source specifies it. Decide and record it.

### Test pins to mirror
- `types::tests::agent_status_roundtrip` (`src/types.rs:93`): status wire strings.
- `daemon::tests::test_focus_ipc_roundtrip` (`src/daemon.rs:952`): `{"method":"daemon.focus"}`.
- `cli::tests::test_spawn_plan_sets_parent_env_when_given` (`src/cli.rs:840`) and `test_spawn_plan_omits_parent_env_when_absent` (`src/cli.rs:846`).
- `tests/spawn_register.rs` `test_register_and_wait_sees_registration_and_times_out_otherwise`: register over the socket.
- `daemon::tests::test_malformed_ipc_line_keeps_client` (`src/daemon.rs:873`), `test_non_utf8_ipc_line_is_skipped` (`src/daemon.rs:887`).
- Gap: no swayve test pins a raw `agent.register` with `parent_agent_id` set to a value or to `null`. The sidecar's unit test should compare against both forms and against `agent.update_status` with `"status":"executing_tool"`.

### Unverified
- Whether a sidecar root agent ends up with `parent_id` from process audit. Code-verified, not run live. Check with `make daemon-dry` and a window whose ancestor is a registered pid.
- Antigravity hook env vars, PIDs, parent ids, subagent coverage, and an external-observer API. Not found in primary source.
