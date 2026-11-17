import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { registerHooks } from "node:module";
import net from "node:net";
import os from "node:os";
import path from "node:path";

const pluginStub = `export const Plugin = { define: (definition) => definition };`;
registerHooks({
  resolve(specifier, context, nextResolve) {
    if (specifier === "@opencode/plugin") {
      return { url: `data:text/javascript,${encodeURIComponent(pluginStub)}`, shortCircuit: true };
    }
    return nextResolve(specifier, context);
  },
});

const pluginUrl = new URL("../../.opencode/plugins/swayve.ts", import.meta.url).href;
const { default: plugin } = await import(pluginUrl);

function event(type, sessionID = "session-1", directory) {
  const info = directory === undefined
    ? { id: sessionID }
    : { id: sessionID, directory };
  return { type, properties: { info } };
}

function eventStream() {
  const pending = [];
  const waiters = [];
  let ended = false;
  let signal;
  let abortCount = 0;
  const iterable = {
    [Symbol.asyncIterator]() {
      return {
        next() {
          if (pending.length) return Promise.resolve({ value: pending.shift(), done: false });
          if (ended) return Promise.resolve({ done: true });
          return new Promise((resolve) => waiters.push(resolve));
        },
      };
    },
  };
  return {
    iterable,
    attach(options) {
      signal = options.signal;
      signal.addEventListener("abort", () => {
        abortCount++;
        ended = true;
        while (waiters.length) waiters.shift()({ done: true });
      }, { once: true });
    },
    push(value) {
      const waiter = waiters.shift();
      if (waiter) waiter({ value, done: false });
      else pending.push(value);
    },
    get signal() { return signal; },
    get abortCount() { return abortCount; },
  };
}

async function mockSocket(run) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "swayve-opencode-test-"));
  const socketPath = path.join(directory, "swayve.sock");
  const lines = [];
  let server;
  try {
    server = net.createServer((connection) => {
      let buffer = "";
      connection.setEncoding("utf8");
      connection.on("data", (chunk) => {
        buffer += chunk;
        let newline;
        while ((newline = buffer.indexOf("\n")) !== -1) {
          lines.push(JSON.parse(buffer.slice(0, newline)));
          buffer = buffer.slice(newline + 1);
        }
      });
    });
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(socketPath, resolve);
    });
    await run({ socketPath, lines });
  } finally {
    if (server?.listening) {
      await new Promise((resolve) => server.close(resolve));
    }
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

async function flush(lines, expectedCount) {
  for (let attempt = 0; attempt < 50 && lines.length < expectedCount; attempt++) {
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

function startPlugin(stream, location = "/workspace/plugin") {
  const cleanup = plugin.setup({
    location: { directory: location },
    event: {
      subscribe(options) {
        stream.attach(options);
        return stream.iterable;
      },
    },
  });
  return cleanup;
}

test("registers sessions with event workspace identity and maps lifecycle statuses", async () => {
  await mockSocket(async ({ socketPath, lines }) => {
    process.env.SWAYVE_SOCKET = socketPath;
    const stream = eventStream();
    const cleanup = startPlugin(stream);
    try {
      stream.push(event("session.created", "s-123", "/home/user/workspace/project-x"));
      stream.push(event("session.execution.started", "s-123"));
      stream.push(event("session.tool.called", "s-123"));
      stream.push(event("session.tool.success", "s-123"));
      stream.push(event("session.execution.succeeded", "s-123"));
      stream.push(event("session.execution.failed", "s-123"));
      stream.push(event("session.execution.interrupted", "s-123"));
      stream.push(event("session.tool.failed", "s-123"));
      await flush(lines, 8);

      assert.deepEqual(lines, [
        { method: "agent.register", params: {
          agent_id: "s-123", agent_type: "opencode", project_id: "project-x",
          pid: process.pid, app_id: "agent.s-123", parent_agent_id: null,
        } },
        ...[
          "thinking", "executing_tool", "thinking", "completed",
          { failed: "OpenCode execution failed" }, "idle",
          { failed: "OpenCode execution failed" },
        ].map((status) => ({
          method: "agent.update_status",
          params: { agent_id: "s-123", status },
        })),
      ]);
    } finally {
      cleanup();
      delete process.env.SWAYVE_SOCKET;
    }
  });
});

test("falls back to plugin location and ignores unidentified/duplicate creation", async () => {
  await mockSocket(async ({ socketPath, lines }) => {
    process.env.SWAYVE_SOCKET = socketPath;
    const stream = eventStream();
    const cleanup = startPlugin(stream, "/workspace/fallback-project");
    try {
      stream.push({ type: "session.created", properties: {} });
      stream.push(event("session.created", "fallback-id"));
      stream.push(event("session.created", "fallback-id"));
      await flush(lines, 1);
      assert.deepEqual(lines, [{
        method: "agent.register",
        params: {
          agent_id: "fallback-id", agent_type: "opencode",
          project_id: "fallback-project", pid: process.pid,
          app_id: "agent.fallback-id", parent_agent_id: null,
        },
      }]);
    } finally {
      cleanup();
      delete process.env.SWAYVE_SOCKET;
    }
  });
});

test("cleanup aborts subscription and stops subsequent event handling", async () => {
  await mockSocket(async ({ socketPath, lines }) => {
    process.env.SWAYVE_SOCKET = socketPath;
    const stream = eventStream();
    const cleanup = startPlugin(stream);
    try {
      stream.push(event("session.created", "before-cleanup", "/work/before-cleanup"));
      await flush(lines, 1);
      cleanup();
      assert.equal(stream.signal.aborted, true);
      assert.equal(stream.abortCount, 1);
      stream.push(event("session.created", "after-cleanup", "/work/after-cleanup"));
      await new Promise((resolve) => setTimeout(resolve, 30));
      assert.equal(lines.length, 1);
      assert.equal(lines[0].params.agent_id, "before-cleanup");
    } finally {
      cleanup();
      delete process.env.SWAYVE_SOCKET;
    }
  });
});

test("missing or unavailable socket does not interrupt event processing", async () => {
  const stream = eventStream();
  delete process.env.SWAYVE_SOCKET;
  delete process.env.XDG_RUNTIME_DIR;
  const cleanup = startPlugin(stream);
  stream.push(event("session.created", "no-socket"));
  await flush([], 0);
  cleanup();

  process.env.SWAYVE_SOCKET = path.join(os.tmpdir(), `swayve-missing-${process.pid}.sock`);
  const failedStream = eventStream();
  const failedCleanup = startPlugin(failedStream);
  failedStream.push(event("session.created", "unavailable-socket"));
  failedStream.push(event("session.execution.started", "unavailable-socket"));
  await flush([], 0);
  failedCleanup();
  delete process.env.SWAYVE_SOCKET;
  assert.equal(failedStream.signal.aborted, true);
});
