#!/usr/bin/env node
// Antigravity sidecar: sends one Swayve IPC line per run over the daemon socket.
// Best-effort: every failure logs one stderr line and exits 0 (D5). Never reads the socket.

import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { pathToFileURL } from "node:url";

const SIMPLE_STATUSES = ["idle", "thinking", "executing_tool", "awaiting_human_input", "completed"];

const HOOK_DEFAULT_STATUS = {
  SessionStart: "thinking",
  PreToolUse: "executing_tool",
  PostToolUse: "thinking",
  PreInvocation: "thinking",
  PostInvocation: "thinking",
  Stop: "idle",
};

function normalizeParentId(parentAgentId) {
  return parentAgentId ? parentAgentId : null;
}

export function hookPayload(event) {
  if (event === "PreToolUse") return JSON.stringify({ decision: "allow" });
  if (event === "Stop") return JSON.stringify({ decision: "allow_stop" });
  return "{}";
}

export function hookLines(event, agent, overrideCmd, overrideValue) {
  if (overrideCmd === "register") return [registerLine(agent)];
  if (overrideCmd === "status") return [statusLine({ agentId: agent.agentId, status: overrideValue })];
  if (overrideCmd === "focus") return [focusLine()];
  if (overrideCmd) throw new Error(`unknown hook action: ${overrideCmd}`);
  const status = HOOK_DEFAULT_STATUS[event];
  if (!status) throw new Error(`unknown hook event: ${event ?? "(none)"}`);
  if (event === "SessionStart") {
    return [registerLine(agent), statusLine({ agentId: agent.agentId, status })];
  }
  return [statusLine({ agentId: agent.agentId, status })];
}

export function registerLine({ agentId, projectId, pid, parentAgentId }) {
  return JSON.stringify({
    method: "agent.register",
    params: {
      agent_id: agentId,
      agent_type: "antigravity",
      project_id: projectId,
      pid,
      app_id: `agent.${agentId}`,
      parent_agent_id: normalizeParentId(parentAgentId),
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

export function resolveEnv(env, ppid, stdinJson) {
  const agentId = env.SWAYVE_AGENT_ID || env.ANTIGRAVITY_CONVERSATION_ID || stdinJson?.conversationId;
  if (!agentId) return null;
  const pidText = env.SWAYVE_AGENT_PID ?? "";
  // ponytail: strict digits only, so "12abc" falls back to ppid.
  const pid = /^[1-9]\d*$/.test(pidText) ? Number(pidText) : ppid;
  const workspaceProject =
    Array.isArray(stdinJson?.workspacePaths) && stdinJson.workspacePaths[0]
      ? path.basename(stdinJson.workspacePaths[0])
      : "";
  return {
    agentId,
    projectId: env.SWAYVE_PROJECT_ID || env.ANTIGRAVITY_PROJECT_ID || workspaceProject || "",
    pid,
    parentAgentId: normalizeParentId(env.SWAYVE_PARENT_ID),
  };
}

function logAndSkip(msg) {
  process.stderr.write(`swayve-sidecar: ${msg}\n`);
}

function send(socketPath, lines) {
  const payload = (Array.isArray(lines) ? lines.join("\n") : lines) + "\n";
  const socket = net.createConnection(socketPath, () => {
    socket.end(payload);
  });
  socket.setTimeout(2000, () => {
    logAndSkip("send timed out");
    socket.destroy();
  });
  socket.on("error", (err) => {
    logAndSkip(`send failed: ${err.message}`);
  });
}

function parseStdinJson(stdinText) {
  if (!stdinText || !stdinText.trim()) return null;
  try {
    return JSON.parse(stdinText);
  } catch {
    return null;
  }
}

export function main(argv, env, ppid, options = {}) {
  const [cmd, value, extraCmd, extraValue] = argv;
  const writeStdout = options.writeStdout ?? ((s) => process.stdout.write(s));

  if (cmd === "hook") {
    const event = value;
    writeStdout(hookPayload(event) + "\n");
    const socketPath = resolveSocketPath(env);
    if (!socketPath) {
      logAndSkip("no socket path (set SWAYVE_SOCKET or XDG_RUNTIME_DIR); skipped");
      return;
    }
    if (extraCmd === "focus") {
      send(socketPath, focusLine());
      return;
    }
    const stdinJson = parseStdinJson(options.stdinText);
    const agent = resolveEnv(env, ppid, stdinJson);
    if (!agent) {
      logAndSkip("SWAYVE_AGENT_ID is not set; skipped");
      return;
    }
    try {
      const lines = hookLines(event, agent, extraCmd, extraValue);
      send(socketPath, lines);
    } catch (err) {
      logAndSkip(err.message);
    }
    return;
  }

  const socketPath = resolveSocketPath(env);
  if (!socketPath) {
    logAndSkip("no socket path (set SWAYVE_SOCKET or XDG_RUNTIME_DIR); skipped");
    return;
  }
  if (cmd === "focus") {
    send(socketPath, focusLine());
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
    } else {
      logAndSkip(`unknown subcommand: ${cmd ?? "(none)"}; expected register|status|focus|hook`);
      return;
    }
  } catch (err) {
    logAndSkip(err.message);
    return;
  }
  send(socketPath, line);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  let stdinText = "";
  if (process.argv[2] === "hook" && !process.stdin.isTTY) {
    try {
      stdinText = fs.readFileSync(0, "utf8");
    } catch {
      stdinText = "";
    }
  }
  main(process.argv.slice(2), process.env, process.ppid, { stdinText });
}
