import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import {
  registerLine,
  statusLine,
  focusLine,
  hookPayload,
  hookLines,
  resolveSocketPath,
  resolveEnv,
  main,
} from "./sidecar.mjs";

// Asserts structural equality (JSON.parse + deepEqual) and byte equality with key order from the contract.
function checkWire(line, expected) {
  assert.deepEqual(JSON.parse(line), expected);
  assert.equal(line, JSON.stringify(expected));
}

test("register for a subagent sends the parent id", () => {
  const line = registerLine({ agentId: "a-1", projectId: "swayve", pid: 42, parentAgentId: "p-0" });
  const expected = {
    method: "agent.register",
    params: {
      agent_id: "a-1",
      agent_type: "antigravity",
      project_id: "swayve",
      pid: 42,
      app_id: "agent.a-1",
      parent_agent_id: "p-0",
    },
  };
  checkWire(line, expected);
});

test("register for a root agent sends null for null, empty, and undefined parent", () => {
  for (const parentAgentId of [null, "", undefined]) {
    const line = registerLine({ agentId: "a-1", projectId: "swayve", pid: 42, parentAgentId });
    assert.equal(JSON.parse(line).params.parent_agent_id, null);
  }
});

test("status executing_tool matches the contract byte for byte", () => {
  const line = statusLine({ agentId: "a-1", status: "executing_tool" });
  const expected = { method: "agent.update_status", params: { agent_id: "a-1", status: "executing_tool" } };
  assert.equal(line, '{"method":"agent.update_status","params":{"agent_id":"a-1","status":"executing_tool"}}');
  checkWire(line, expected);
});

test("failed:timeout parses to a failed status object", () => {
  const line = statusLine({ agentId: "a-1", status: "failed:timeout" });
  assert.deepEqual(JSON.parse(line).params.status, { failed: "timeout" });
  checkWire(line, { method: "agent.update_status", params: { agent_id: "a-1", status: { failed: "timeout" } } });
});

test("unknown status throws", () => {
  assert.throws(() => statusLine({ agentId: "a-1", status: "bogus" }), /unknown status/);
});

test("focus line is exactly the contract message", () => {
  assert.equal(focusLine(), '{"method":"daemon.focus"}');
  checkWire(focusLine(), { method: "daemon.focus" });
});

test("resolveSocketPath: explicit wins, empty falls through to XDG, both missing gives null", () => {
  assert.equal(resolveSocketPath({ SWAYVE_SOCKET: "/x/s.sock", XDG_RUNTIME_DIR: "/run/u" }), "/x/s.sock");
  assert.equal(
    resolveSocketPath({ SWAYVE_SOCKET: "", XDG_RUNTIME_DIR: "/run/u" }),
    "/run/u/swayve/swayve.sock",
  );
  assert.equal(resolveSocketPath({}), null);
});

test("resolveEnv: missing SWAYVE_AGENT_ID returns null", () => {
  assert.equal(resolveEnv({ SWAYVE_PROJECT_ID: "swayve" }, 99), null);
});

test("resolveEnv: bad SWAYVE_AGENT_PID falls back to ppid", () => {
  for (const bad of ["12abc", "0", "-3", ""]) {
    const env = { SWAYVE_AGENT_ID: "a-1", SWAYVE_AGENT_PID: bad };
    assert.equal(resolveEnv(env, 99).pid, 99, `pid input ${JSON.stringify(bad)}`);
  }
  assert.equal(resolveEnv({ SWAYVE_AGENT_ID: "a-1", SWAYVE_AGENT_PID: "512" }, 99).pid, 512);
});

test("resolveEnv: empty SWAYVE_PARENT_ID means null", () => {
  assert.equal(resolveEnv({ SWAYVE_AGENT_ID: "a-1", SWAYVE_PARENT_ID: "" }, 1).parentAgentId, null);
});

test("resolveEnv: falls back to ANTIGRAVITY_CONVERSATION_ID and ANTIGRAVITY_PROJECT_ID, SWAYVE_* wins", () => {
  const fallback = resolveEnv(
    { ANTIGRAVITY_CONVERSATION_ID: "conv-123", ANTIGRAVITY_PROJECT_ID: "agy-proj" },
    77,
  );
  assert.deepEqual(fallback, {
    agentId: "conv-123",
    projectId: "agy-proj",
    pid: 77,
    parentAgentId: null,
  });

  const override = resolveEnv(
    {
      SWAYVE_AGENT_ID: "sway-1",
      SWAYVE_PROJECT_ID: "sway-proj",
      ANTIGRAVITY_CONVERSATION_ID: "conv-123",
      ANTIGRAVITY_PROJECT_ID: "agy-proj",
    },
    77,
  );
  assert.equal(override.agentId, "sway-1");
  assert.equal(override.projectId, "sway-proj");

  const fromStdin = resolveEnv({}, 77, {
    conversationId: "stdin-conv",
    workspacePaths: ["/home/user/workspace/swayve"],
  });
  assert.equal(fromStdin.agentId, "stdin-conv");
  assert.equal(fromStdin.projectId, "swayve");
});

test("focus runs without SWAYVE_AGENT_ID over a local Unix socket", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "swayve-agy-test-"));
  const sockPath = path.join(tmpDir, "test.sock");
  let server;
  try {
    const received = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("focus test timed out after 5 s")), 5000);
      server = net.createServer((conn) => {
        let buf = "";
        conn.setEncoding("utf8");
        conn.on("data", (chunk) => {
          buf += chunk;
        });
        conn.on("end", () => {
          clearTimeout(timer);
          server.close(() => resolve(buf));
        });
      });
      server.on("error", (err) => {
        clearTimeout(timer);
        reject(err);
      });
      server.listen(sockPath, () => {
        main(["focus"], { SWAYVE_SOCKET: sockPath }, 100);
      });
    });
    assert.equal(received, '{"method":"daemon.focus"}\n');
  } finally {
    if (server?.listening) server.close();
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("send gives up on a stalled daemon", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "swayve-agy-test-"));
  const sockPath = path.join(tmpDir, "stalled.sock");
  let server;
  let serverConn;
  try {
    await new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        serverConn?.destroy();
        reject(new Error("stalled daemon connection was not closed within 4 s"));
      }, 4000);
      server = net.createServer({ allowHalfOpen: true }, (conn) => {
        serverConn = conn;
        let probe;
        let start;
        conn.on("error", () => {});
        conn.on("data", () => {});
        // Start writing only after the client's 2 s idle timeout, so the probes do not reset it.
        conn.on("end", () => {
          start = setTimeout(() => {
            probe = setInterval(() => conn.write("p"), 50);
          }, 2500);
        });
        conn.on("close", () => {
          clearTimeout(timer);
          clearTimeout(start);
          clearInterval(probe);
          resolve();
        });
      });
      server.on("error", reject);
      server.listen(sockPath, () => {
        main(["focus"], { SWAYVE_SOCKET: sockPath }, 100);
      });
    });
  } finally {
    serverConn?.destroy();
    if (server?.listening) server.close();
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
});

test("hookPayload and hook mode emit required Antigravity stdout JSON and socket lines", async () => {
  assert.equal(hookPayload("PreToolUse"), '{"decision":"allow"}');
  assert.equal(hookPayload("Stop"), '{"decision":"allow_stop"}');
  assert.equal(hookPayload("PostToolUse"), "{}");
  assert.equal(hookPayload("SessionStart"), "{}");

  const agent = { agentId: "a-1", projectId: "swayve", pid: 42, parentAgentId: null };
  assert.equal(hookLines("SessionStart", agent).length, 2);
  assert.equal(JSON.parse(hookLines("SessionStart", agent)[0]).method, "agent.register");
  assert.equal(JSON.parse(hookLines("SessionStart", agent)[1]).params.status, "thinking");
  assert.equal(JSON.parse(hookLines("PreToolUse", agent)[0]).params.status, "executing_tool");
  assert.equal(JSON.parse(hookLines("PostToolUse", agent)[0]).params.status, "thinking");
  assert.equal(JSON.parse(hookLines("Stop", agent)[0]).params.status, "idle");

  // Even without a socket path, hook mode always writes valid JSON to stdout
  const out = [];
  for (const ev of ["PreToolUse", "PostToolUse", "SessionStart", "Stop"]) {
    main(["hook", ev], {}, 100, { writeStdout: (s) => out.push(s) });
  }
  assert.deepEqual(out, [
    '{"decision":"allow"}\n',
    "{}\n",
    "{}\n",
    '{"decision":"allow_stop"}\n',
  ]);
});
