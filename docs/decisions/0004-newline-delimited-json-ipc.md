---
id: "0004"
title: "Newline-Delimited JSON IPC with Method-Tagged Messages"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-28
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Commits 43497ae, 1410acc, 5914a63; src/types.rs AgentIPCMessage; Sway Multi-Agent Viewer Architecture.md (JSON-RPC 2.0 spec, not adopted)"
---

# 4. Newline-Delimited JSON IPC with Method-Tagged Messages

## Context and Problem Statement
Sub-agents and shell hooks must send status to the daemon with no SDK and no library beyond a socket tool. The design docs specify JSON-RPC 2.0 with `id`, response objects, and error codes. The implementation does not use that envelope.

## Decision Drivers
* Clients are shell scripts using `nc -U` or `socat` (see [0009](0009-thin-socket-integrations.md)).
* Messages are mostly fire-and-forget status updates.
* Wire format must stay readable in a log.

## Considered Options
1. JSON-RPC 2.0 as specified in the design doc (request `id`, response envelope, error codes).
2. Newline-delimited JSON, one object per line, `method` as the serde tag.
3. A binary protocol.

## Decision Outcome
Chosen option: "Newline-delimited JSON with `method` as the serde tag", because clients can send one `printf ... | nc -U` line with no framing library, and the daemon has no response to match.

Wire format (`src/types.rs`, `AgentIPCMessage`, `#[serde(tag = "method")]`):
* `{"method":"agent.register", ...}`
* `{"method":"agent.update_status", "params":{...}}`
* `{"method":"daemon.focus"}`
* `{"method":"daemon.subscribe"}`

### Consequences
* **Positive**: Hooks and wrappers need no JSON-RPC library. A status update is one line.
* **Negative**: No request IDs and no per-request responses. The daemon writes only to subscribers (`src/daemon.rs` `handle_client`). A client cannot confirm a registration succeeded. Callers that need confirmation must read the subscription stream.
* **Negative**: The design doc's JSON-RPC section now describes a protocol that does not exist. Its `agent.heartbeat` method was removed in commit `43497ae`.

## Validation & Compliance
`src/daemon.rs` tests assert exact wire strings, e.g. `{"method":"daemon.focus"}`.
