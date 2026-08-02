# Pi Integration for Swayve

This guide provides instructions and examples for integration of Pi agent extensions and plugins with **Swayve**.

## Overview

Swayve manages agent terminal windows and hierarchical layouts by listening on a Unix domain socket specified by the `$SWAYVE_SOCKET` environment variable. Pi plugins can register their subagents and update their execution status (`idle`, `thinking`, `executing_tool`, `awaiting_human_input`, `completed`, `failed`) in real time over this socket.

---

## Environment Variables

When Swayve launches an agent container or window, it injects the following environment variables:

| Variable | Description |
| :--- | :--- |
| `SWAYVE_SOCKET` | Path to the Swayve IPC Unix domain socket (e.g., `/tmp/swayve.sock`) |
| `SWAYVE_AGENT_ID` | Unique identifier assigned to this agent instance |
| `SWAYVE_PROJECT_ID` | Project or workspace identifier |
| `SWAYVE_PARENT_ID` | (Optional) Parent agent ID if this agent is spawned as a child subagent |

---

## System Prompt for Pi

Inject the following prompt into your Pi system configuration or subagent instructions:

```text
You are operating within the Swayve agent environment.
Whenever you start processing a task, execute a tool call, wait for user input, or finish execution, notify the Swayve daemon via the Unix socket specified in $SWAYVE_SOCKET.

Use the agent.update_status IPC method with your $SWAYVE_AGENT_ID and status string ("thinking", "executing_tool", "idle", "completed", or "failed").
```

---

## Mock Python Integration Code

```python
import os
import json
import socket
import sys

def send_swayve_ipc(payload: dict) -> bool:
    socket_path = os.environ.get("SWAYVE_SOCKET")
    if not socket_path or not os.path.exists(socket_path):
        return False
    
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.connect(socket_path)
        data = json.dumps(payload) + "\n"
        client.sendall(data.encode("utf-8"))
        client.close()
        return True
    except Exception as e:
        sys.stderr.write(f"[Swayve IPC Warning] Failed to send IPC payload: {e}\n")
        return False

def register_agent(agent_id: str, agent_type: str = "pi", project_id: str = "default", parent_id: str = None):
    payload = {
        "method": "agent.register",
        "params": {
            "agent_id": agent_id,
            "agent_type": agent_type,
            "project_id": project_id,
            "pid": os.getpid(),
            "app_id": f"agent.{agent_id}",
            "parent_agent_id": parent_id
        }
    }
    return send_swayve_ipc(payload)

def update_status(agent_id: str, status: str):
    payload = {
        "method": "agent.update_status",
        "params": {
            "agent_id": agent_id,
            "status": status
        }
    }
    return send_swayve_ipc(payload)

if __name__ == "__main__":
    agent_id = os.environ.get("SWAYVE_AGENT_ID", "pi-demo-agent")
    project_id = os.environ.get("SWAYVE_PROJECT_ID", "default-project")
    parent_id = os.environ.get("SWAYVE_PARENT_ID")

    print(f"Registering Pi agent {agent_id} with Swayve...")
    register_agent(agent_id=agent_id, agent_type="pi", project_id=project_id, parent_id=parent_id)

    print("Updating status to thinking...")
    update_status(agent_id, "thinking")

    print("Updating status to completed...")
    update_status(agent_id, "completed")
```

---

## Mock JavaScript / Node.js Integration Code

```javascript
const net = require('net');
const process = require('process');

function sendSwayveIPC(payload) {
    const socketPath = process.env.SWAYVE_SOCKET;
    if (!socketPath) {
        return Promise.resolve(false);
    }

    return new Promise((resolve) => {
        const client = net.createConnection({ path: socketPath }, () => {
            client.write(JSON.stringify(payload) + '\n', () => {
                client.end();
                resolve(true);
            });
        });

        client.on('error', (err) => {
            console.error(`[Swayve IPC Warning] ${err.message}`);
            resolve(false);
        });
    });
}

async function registerAgent(agentId, agentType = 'pi', projectId = 'default', parentId = null) {
    const payload = {
        method: 'agent.register',
        params: {
            agent_id: agentId,
            agent_type: agentType,
            project_id: projectId,
            pid: process.pid,
            app_id: `agent.${agentId}`,
            parent_agent_id: parentId
        }
    };
    return await sendSwayveIPC(payload);
}

async function updateStatus(agentId, status) {
    const payload = {
        method: 'agent.update_status',
        params: {
            agent_id: agentId,
            status: status
        }
    };
    return await sendSwayveIPC(payload);
}

// Example usage
(async () => {
    const agentId = process.env.SWAYVE_AGENT_ID || 'pi-js-agent';
    const projectId = process.env.SWAYVE_PROJECT_ID || 'default-project';
    const parentId = process.env.SWAYVE_PARENT_ID || null;

    await registerAgent(agentId, 'pi', projectId, parentId);
    await updateStatus(agentId, 'thinking');
    await updateStatus(agentId, 'idle');
})();
```
