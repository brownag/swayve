import { test } from "node:test";
import assert from "node:assert/strict";
import {
  registerLine,
  statusLine,
  focusLine,
  resolveSocketPath,
  resolveEnv,
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

test("register for a root agent sends null for empty and undefined parent", () => {
  for (const parentAgentId of ["", undefined]) {
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
