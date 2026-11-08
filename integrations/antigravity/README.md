# Antigravity Integration for Swayve

This document details how the **Antigravity CLI** (`agy`) and associated sidecar services integrate with **Swayve** for agent window management, status tracking, and hierarchical focus navigation.

## Overview

Google Antigravity orchestrates AI coding workflows across agents and subagent trees. When running within a Sway tiling environment managed by Swayve, Antigravity components use environment variables and Unix domain socket IPC to sync agent state, window layouts, and focus state with Swayve daemon.

---

## Environment Variables

These variables are supplied by the launcher, not by the swayve daemon. `swayve spawn` sets `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, `SWAYVE_PROJECT_ID` and, for a subagent, `SWAYVE_PARENT_ID` (`src/cli.rs:564-570`). The daemon does not set `SWAYVE_PARENT_ID`, `SWAYVE_AGENT_PID`, or any other variable below.

| Variable | Description | Example |
| :--- | :--- | :--- |
| `SWAYVE_SOCKET` | Path to the active Swayve IPC Unix domain socket. Empty counts as unset. Default: $XDG_RUNTIME_DIR/swayve/swayve.sock (`src/cli.rs:118-126`, `src/cli.rs:73-82`). Set SWAYVE_SOCKET to override. | `$XDG_RUNTIME_DIR/swayve/swayve.sock` |
| `SWAYVE_AGENT_ID` | Unique ID of the agent instance. Supplied by the launcher. Required by the sidecar; when missing, the sidecar logs and sends nothing. | `agy-agent-8f3a` |
| `SWAYVE_PROJECT_ID` | Project / workspace ID. Supplied by the launcher. | `swayve-dev` |
| `SWAYVE_PARENT_ID` | Agent ID of the spawning parent agent (if subagent). Supplied by the launcher. Empty or unset means a root agent (`parent_agent_id: null`). | `parent-agent-001` |
| `SWAYVE_AGENT_PID` | PID reported in `agent.register`. Supplied by the launcher. Must be a positive integer. If unset or invalid, the sidecar falls back to `process.ppid`. The fallback is unverified and is checked by the live test. | `48201` |

---

## Antigravity Sidecar Hook Architecture

Antigravity CLI and sidecars read `$SWAYVE_SOCKET` upon startup and establish a persistent or per-event Unix domain socket connection to Swayve.

### 1. Registration (`agent.register`)

Upon initializing a new agent window or subagent process, the Antigravity sidecar sends an `agent.register` IPC message:

```json
{
  "method": "agent.register",
  "params": {
    "agent_id": "agy-agent-8f3a",
    "agent_type": "antigravity",
    "project_id": "swayve-dev",
    "pid": 48201,
    "app_id": "agent.agy-agent-8f3a",
    "parent_agent_id": "parent-agent-001"
  }
}
```

This enables Swayve to match the window container in Sway (`app_id`) and track process ancestry.

Notes: `parent_agent_id` is `null` for a root agent. `app_id` is always `agent.<agent_id>`.

---

### 2. Status Updates (`agent.update_status`)

During task execution, Antigravity sidecars emit status lifecycle updates to keep Swayve UI and border indicators up to date:

- **Thinking / Planning**:
  ```json
  {"method": "agent.update_status", "params": {"agent_id": "agy-agent-8f3a", "status": "thinking"}}
  ```

- **Executing Tool**:
  ```json
  {"method": "agent.update_status", "params": {"agent_id": "agy-agent-8f3a", "status": "executing_tool"}}
  ```

- **Awaiting User Input**:
  ```json
  {"method": "agent.update_status", "params": {"agent_id": "agy-agent-8f3a", "status": "awaiting_human_input"}}
  ```

- **Idle**:
  ```json
  {"method": "agent.update_status", "params": {"agent_id": "agy-agent-8f3a", "status": "idle"}}
  ```

- **Completion**:
  ```json
  {"method": "agent.update_status", "params": {"agent_id": "agy-agent-8f3a", "status": "completed"}}
  ```

- **Failure**:
  ```json
  {"method": "agent.update_status", "params": {"agent_id": "agy-agent-8f3a", "status": {"failed": "Task timed out"}}}
  ```

---

### 3. Hierarchical Focus Cycling (`daemon.focus`)

Antigravity keybindings or CLI triggers can request Swayve to cycle focus through sibling agent windows:

```json
{
  "method": "daemon.focus"
}
```

Swayve processes this request via its internal hierarchical focus algorithm, navigating between parent and child agent windows.

---

## Run and test

The sidecar is `integrations/antigravity/sidecar.mjs` (Node 22, no dependencies).

```bash
cd integrations/antigravity && node --test sidecar.test.mjs
```

Send one message by hand (the launcher normally sets the env vars):

```bash
SWAYVE_SOCKET=/run/user/1000/swayve/swayve.sock SWAYVE_AGENT_ID=a-33-1 SWAYVE_PROJECT_ID=swayve \
  node integrations/antigravity/sidecar.mjs register
```

Subcommands: `register`, `status <value>`, `focus`. The sidecar writes one line and does not read a reply. If no socket path or `SWAYVE_AGENT_ID` is set, it writes one line to stderr and exits 0. Socket errors also log one line and exit 0. Status reporting is best-effort, so the sidecar never fails a hook.

## Example Shell Injection

To automatically hook Antigravity CLI terminal sessions into Swayve, include the following in your session startup configuration:

```bash
eval "$(swayve init-shell)"
```
