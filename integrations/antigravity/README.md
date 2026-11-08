# Antigravity Integration for Swayve

This document details how the **Antigravity CLI** (`agy`) and associated sidecar services integrate with **Swayve** for agent window management, status tracking, and hierarchical focus navigation.

## Overview

Google Antigravity orchestrates AI coding workflows across agents and subagent trees. When running within a Sway tiling environment managed by Swayve, Antigravity components use environment variables and Unix domain socket IPC to sync agent state, window layouts, and focus state with Swayve daemon.

---

## Environment Variables

When Swayve launches an agent container or window, or when an Antigravity subagent is spawned, the following environment variables are passed down:

| Variable | Description | Example |
| :--- | :--- | :--- |
| `SWAYVE_SOCKET` | Path to the active Swayve IPC Unix domain socket. Default: $XDG_RUNTIME_DIR/swayve/swayve.sock. Set SWAYVE_SOCKET to override. | `$XDG_RUNTIME_DIR/swayve/swayve.sock` |
| `SWAYVE_AGENT_ID` | Unique ID of the agent instance | `agy-agent-8f3a` |
| `SWAYVE_PROJECT_ID` | Project / workspace ID | `swayve-dev` |
| `SWAYVE_PARENT_ID` | Agent ID of the spawning parent agent (if subagent). Not set by the swayve daemon today; the Antigravity sidecar must supply it | `parent-agent-001` |

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

## Example Shell Injection

To automatically hook Antigravity CLI terminal sessions into Swayve, include the following in your session startup configuration:

```bash
eval "$(swayve init-shell)"
```
