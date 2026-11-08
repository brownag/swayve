#!/usr/bin/env node
// Antigravity sidecar: sends one Swayve IPC line per run over the daemon socket.
// Best-effort: every failure logs one stderr line and exits 0 (D5). Never reads the socket.

import net from "node:net";
import path from "node:path";
import { pathToFileURL } from "node:url";

const SIMPLE_STATUSES = ["idle", "thinking", "executing_tool", "awaiting_human_input", "completed"];

export function registerLine({ agentId, projectId, pid, parentAgentId }) {
  const parent = parentAgentId === undefined || parentAgentId === null || parentAgentId === "" ? null : parentAgentId;
  return JSON.stringify({
    method: "agent.register",
    params: {
      agent_id: agentId,
      agent_type: "antigravity",
      project_id: projectId,
      pid,
      app_id: `agent.${agentId}`,
      parent_agent_id: parent,
    },
  });
}

export function statusLine({ agentId, status }) {
  let value;
  if (SIMPLE_STATUSES.includes(status)) {
    value = status;
  } else if (typeof status === "string" && status.startsWith("failed:")) {
    value = { failed: status.slice("failed:".length) };
  } else {
    throw new Error(`unknown status: ${status}`);
  }
  return JSON.stringify({ method: "agent.update_status", params: { agent_id: agentId, status: value } });
}

export function focusLine() {
  return JSON.stringify({ method: "daemon.focus" });
}

// Rule: SWAYVE_SOCKET (non-empty) wins; else $XDG_RUNTIME_DIR/swayve/swayve.sock; else null.
export function resolveSocketPath(env) {
  if (env.SWAYVE_SOCKET) return env.SWAYVE_SOCKET;
  if (env.XDG_RUNTIME_DIR) return path.join(env.XDG_RUNTIME_DIR, "swayve", "swayve.sock");
  return null;
}

export function resolveEnv(env, ppid) {
  const agentId = env.SWAYVE_AGENT_ID;
  if (!agentId) return null;
  const pidText = env.SWAYVE_AGENT_PID ?? "";
  // ponytail: strict digits only, so "12abc" falls back to ppid.
  const pid = /^[1-9]\d*$/.test(pidText) ? Number(pidText) : ppid;
  return {
    agentId,
    projectId: env.SWAYVE_PROJECT_ID ?? "",
    pid,
    parentAgentId: env.SWAYVE_PARENT_ID || null,
  };
}

function logAndSkip(msg) {
  process.stderr.write(`swayve-sidecar: ${msg}\n`);
}

function send(socketPath, line) {
  const socket = net.createConnection(socketPath, () => {
    socket.end(line + "\n");
  });
  socket.on("error", (err) => {
    logAndSkip(`send failed: ${err.message}`);
  });
}

export function main(argv, env, ppid) {
  const [cmd, value] = argv;
  const socketPath = resolveSocketPath(env);
  if (!socketPath) {
    logAndSkip("no socket path (set SWAYVE_SOCKET or XDG_RUNTIME_DIR); skipped");
    return;
  }
  const agent = resolveEnv(env, ppid);
  if (!agent) {
    logAndSkip("SWAYVE_AGENT_ID is not set; skipped");
    return;
  }
  let line;
  try {
    if (cmd === "register") {
      line = registerLine(agent);
    } else if (cmd === "status") {
      line = statusLine({ agentId: agent.agentId, status: value });
    } else if (cmd === "focus") {
      line = focusLine();
    } else {
      logAndSkip(`unknown subcommand: ${cmd ?? "(none)"}; expected register|status|focus`);
      return;
    }
  } catch (err) {
    logAndSkip(err.message);
    return;
  }
  send(socketPath, line);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main(process.argv.slice(2), process.env, process.ppid);
}
