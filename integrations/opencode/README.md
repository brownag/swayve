# OpenCode Integration for Swayve

This project-local OpenCode server plugin reports session lifecycle state to
Swayve over its newline-delimited Unix socket protocol. It observes events only;
it does not add OpenCode tools, commands, or focus controls.

## Setup

The plugin source is `.opencode/plugins/swayve.ts`. OpenCode discovers
project-local plugins from that directory when run from the project. The plugin
uses the `@opencode/plugin` API supplied by OpenCode; no package installation
or extra runtime dependency is required.

Set `SWAYVE_SOCKET` to the active Swayve socket if the default is not correct.
Otherwise, the plugin uses `$XDG_RUNTIME_DIR/swayve/swayve.sock` when
`XDG_RUNTIME_DIR` is set. Start Swayve before OpenCode to receive updates.

Session ID is used as the Swayve agent ID and in `agent.<session_id>`.
`project_id` is the basename of the event session directory, falling back to
the plugin load directory when session directory metadata is absent. The
reported PID is the plugin process PID. Parent-session mapping is unsupported
and unverified; `parent_agent_id` is always null.

## API support

The event names and payload fields used here were verified against the
first-party OpenCode plugin documentation and public event schema on
2026-11-17 (see `planning/RESEARCH_OPENCODE_INTEGRATION.md`). OpenCode does not
specify a supported release pin, so this integration does not claim compatibility
with a particular release. Recheck the documented plugin and event API if
OpenCode changes its plugin interface.

## Tests

Requires Node.js v22.23.5 for built-in synchronous module hooks and TypeScript stripping.
Tests use only Node built-ins and a mock Unix socket; no OpenCode runtime is
needed.

```bash
node --experimental-strip-types --test integrations/opencode/sidecar.test.mjs
```

## Manual smoke test

1. Start Swayve and confirm its socket is available (or set `SWAYVE_SOCKET` to
   its path).
2. From this project, start OpenCode and create a session in a known workspace.
3. Confirm Swayve shows a registered `opencode` agent with the session ID,
   workspace basename, plugin process PID, `agent.<session_id>` app ID, and
   no parent ID.
4. Start a turn, invoke a tool, complete a turn, and if practical exercise a
   failure/interruption. Confirm the observed statuses are respectively
   `thinking`, `executing_tool`, `completed`, failed, or `idle` as applicable.
5. Close/reload OpenCode and confirm plugin unload does not leave ongoing
   event processing or prevent OpenCode from exiting.

If registration or status reporting fails, verify the socket path and compare
the running OpenCode plugin/event API with the first-party contract referenced
above. Missing or unavailable sockets are best-effort and do not block OpenCode.
