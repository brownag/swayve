import { Plugin } from "@opencode/plugin"
import net from "node:net"
import path from "node:path"

const eventTypes = [
  "session.created",
  "session.execution.started",
  "session.execution.succeeded",
  "session.execution.failed",
  "session.execution.interrupted",
  "session.tool.called",
  "session.tool.success",
  "session.tool.failed",
]

function socketPath() {
  return process.env.SWAYVE_SOCKET ||
    (process.env.XDG_RUNTIME_DIR && path.join(process.env.XDG_RUNTIME_DIR, "swayve", "swayve.sock"))
}

function send(message: object) {
  const address = socketPath()
  if (!address) return
  try {
    const socket = net.createConnection(address, () => socket.end(`${JSON.stringify(message)}\n`))
    socket.setTimeout(1000, () => socket.destroy())
    socket.on("error", () => {})
  } catch {
    // Swayve is best-effort; reporting must not affect OpenCode.
  }
}

export default Plugin.define({
  id: "swayve",
  setup(ctx) {
    const controller = new AbortController()
    const registered = new Set<string>()

    void (async () => {
      try {
        for await (const event of ctx.event.subscribe({ types: eventTypes, signal: controller.signal })) {
          const session = "data" in event ? event.data : undefined
          const sessionID = session?.sessionID
          if (!sessionID) continue

          if (event.type === "session.created" && !registered.has(sessionID)) {
            registered.add(sessionID)
            const workspace = session.location?.directory || ctx.location.directory
            send({
              method: "agent.register",
              params: {
                agent_id: sessionID,
                agent_type: "opencode",
                project_id: path.basename(workspace),
                pid: process.pid,
                app_id: `agent.${sessionID}`,
                parent_agent_id: null,
              },
            })
          }

          const status =
            event.type === "session.execution.started" ? "thinking" :
            event.type === "session.tool.called" ? "executing_tool" :
            event.type === "session.execution.succeeded" ? "completed" :
            event.type === "session.execution.failed" || event.type === "session.tool.failed"
              ? { failed: "OpenCode execution failed" }
              : event.type === "session.execution.interrupted" ? "idle" :
            event.type === "session.tool.success" ? "thinking" : undefined
          if (status) {
            send({
              method: "agent.update_status",
              params: { agent_id: sessionID, status },
            })
          }
        }
      } catch (error) {
        if (!controller.signal.aborted) console.error("swayve: event subscription failed", error)
      }
    })()

    return () => controller.abort()
  },
})
