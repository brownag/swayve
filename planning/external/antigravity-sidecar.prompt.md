# Agent prompt: Antigravity sidecar against the Swayve socket contract (G25, G15)

You are working in the Antigravity sidecar repository, which is outside the swayve repo. Its location is `{{SIDECAR_REPO}}`. Do not edit anything in the swayve repo. Do not push or run any remote-write git command.

## Goal
The sidecar reports Antigravity agent state to the Swayve daemon, using only the documented socket contract. Done means every item in "Acceptance" passes against a live `swayve daemon` or `make daemon-dry` (run from the swayve repo, read-only).

## Socket contract (verified against swayve `src/types.rs` and `README.md` Socket Contract)
- Socket path: `$SWAYVE_SOCKET` if set; otherwise `$XDG_RUNTIME_DIR/swayve/swayve.sock`. Without `XDG_RUNTIME_DIR` and `SWAYVE_SOCKET`, the sidecar must not guess a path. It should log and skip.
- Framing: one JSON object per line, terminated by `\n`. It is not JSON-RPC 2.0. There is no envelope and no reply.
- The daemon never replies to these messages. Treat the connection as fire-and-forget.
- Env vars passed to the sidecar: `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, `SWAYVE_PROJECT_ID`, and `SWAYVE_PARENT_ID` (see the gap below).

## Messages to implement (exact `method` strings)
1. `agent.register`: `{"method":"agent.register","params":{"agent_id":"...","agent_type":"antigravity","project_id":"...","pid":N,"app_id":"agent.<agent_id>","parent_agent_id":"..."|null}}`. Send on startup of each agent or subagent.
2. `agent.update_status`: `{"method":"agent.update_status","params":{"agent_id":"...","status":"..."}}`. Statuses: `thinking`, `executing_tool`, `idle`, plus any others the daemon accepts. Check the `AgentStatus` serde variants in `src/types.rs` before using a new one.
3. `daemon.focus`: `{"method":"daemon.focus"}`. Use for hierarchical focus cycling.

Do not send any other method. Do not add an ack or a reply path.

## Known gap to resolve
`SWAYVE_PARENT_ID` is not set by the daemon today. For a subagent, the sidecar must supply `parent_agent_id` on `agent.register` itself, from the spawning parent's `agent_id`. Do not invent a parent when none exists. Send `null` for a root agent.

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
