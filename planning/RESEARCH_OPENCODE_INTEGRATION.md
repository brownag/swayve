# OpenCode Integration Research

Last researched: 2026-11-17

## Lifecycle Status Investigation (2026-11-17)

### Conclusion

The evidence leans toward an OpenCode plugin event/subscription or payload-extraction mismatch, rather than a general inability of the running Swayve daemon to apply status updates. This is not proven: the live run established plugin loading and agent registration, but did not trace lifecycle events into plugin callbacks or prove whether status IPC writes completed.

### Evidence

- `.opencode/plugins/swayve.ts:5-14` subscribes to `session.created`, four `session.execution.*` events, and three `session.tool.*` events. At lines 42-44 it extracts the session ID from `event.properties.info.id`, falling back to `event.properties.sessionID`; events without either are skipped. Registration is sent only for `session.created` (lines 46-59).
- The official API describes a typed event stream and the public V2 event schema includes session creation and execution/tool events, but the schema pages are not pinned to OpenCode v2.3.0. In particular, `session.created` is not part of the documented `Session.Event.Durable` set even though it appears in the broader event union. Official server-plugin docs state that unknown event subscription types are ignored with a server-log warning. Therefore, the current public docs do not establish that every subscribed event is emitted to plugins in v2.3.0 or confirm the plugin's payload extraction paths for that version. Sources: [OpenCode V2 API event schema](https://opencode.ai/docs/api#schema-OpenCodeEvent), [server plugin events](https://opencode.ai/docs/build/plugins/server#events).
- Swayve's daemon parses newline-delimited IPC and forwards valid messages to its event loop; malformed UTF-8/JSON and rate-limited lines are logged and skipped (`crates/swayve/src/daemon.rs:513-558`). For a parsed `agent.update_status`, it calls `state.update_status`, then broadcasts state (`daemon.rs:596-620`). The Swayve status enum accepts `thinking`, `executing_tool`, `completed`, `idle`, and the serialized failed-status form (`crates/swayve-types/src/lib.rs:79-89,170-176`).
- An update for an unregistered agent is not applied: `update_status` returns `None` for an unknown ID (`crates/swayve/src/daemon.rs:324-339`). The daemon logs receipt before checking that result (`daemon.rs:596-603`), so a “status update” log line alone would not prove state changed.
- `swayve status --watch` is a state subscriber, not a raw IPC/event log. It consumes snapshots by default and consumes the diff stream with `--diff`, rendering a refreshed dashboard either way (`crates/swayve/src/cli.rs:765-817`). The diff subscription sends an initial snapshot and later agent-state changes (`crates/swayve/src/daemon.rs:64-146`). It does not reveal whether the plugin received OpenCode events or attempted socket writes.
- The approved repeat read-only run with `swayve status --watch --diff` still displayed only the registered agent as Idle. OpenCode's local log showed the project plugin being loaded, but the default log output did not include the session lifecycle events. The log is not version-pinned event-schema evidence.
- Plugin `send()` is best-effort and suppresses socket errors; it neither awaits nor logs connection/write completion (`.opencode/plugins/swayve.ts:21-30`). Thus absence of daemon status changes cannot distinguish “no matching OpenCode event,” “event skipped due to payload extraction,” and “socket delivery failed.”

### How to distinguish the causes

1. Trace the actual OpenCode event `type` and properties delivered to this plugin in v2.3.0. If callbacks show no relevant events, investigate the subscribed names/version contract; if they show events without the expected session ID path, investigate payload extraction.
2. Trace each attempted status IPC send and its socket connection/write result. A successful write establishes plugin emission, but not daemon application.
3. Check daemon logs during the same run. A parsed `Agent <id> status update` confirms an update reached the daemon loop; verify that ID was registered first and compare the following state/diff because unknown IDs are silently unapplied.
4. If the daemon's agent state changes but the subscriber does not reflect it, investigate subscriber delivery/rendering. The current watcher output alone cannot establish IPC arrival.

### Unknowns

- Whether each lifecycle event is emitted to server plugins by OpenCode v2.3.0 and its exact event-property shape.
- Whether the v2.3.0 server logged an unknown-subscription warning.
- Whether callbacks received lifecycle events that were skipped by the current ID extraction.
- Whether the daemon received status IPC before registration, under a mismatched ID, or not at all.
- Whether the plugin's socket connection and write completed during the smoke run.

## Verification Record (2026-11-17)

- OpenCode v2.3.0 is installed in the current workspace environment. `SWAYSOCK` is set and `swaymsg -t get_version` reports Sway 1.9; the Swayve daemon reports active status.
- The official OpenCode v2 API documents `Session.Info.parentID` as the parent session identifier. This is sufficient schema evidence to investigate mapping it to Swayve `parent_agent_id`; confirm that `session.created` supplies it in the live plugin event before relying on it. Sources: [OpenCode v2 session API](https://dev.opencode.ai/v2/docs/api/session/v2-session-get), [OpenCode API](https://opencode.ai/docs/api).
- First-party server plugin docs confirm `Plugin.define({ id, setup(ctx) })`, `ctx.event.subscribe({ types, signal })` as an async iterable, and cleanup via `AbortController.abort()`. Server plugins do not receive `server.connected`.
- Current public event schema names verified for the integration: `session.created`, `session.execution.started`, `session.execution.succeeded`, `session.execution.failed`, `session.execution.interrupted`, `session.tool.called`, `session.tool.success`, and `session.tool.failed`. Event payloads carry session identity in their properties; creation carries session info including `id` and `directory`. Session `directory` is the workspace source when present.
- OpenCode docs do not specify a supported release pin. API verification date is 2026-11-17; no release version is asserted.
- `ctx.location.directory` identifies the plugin load location, not necessarily each session's workspace. It is only an explicitly documented fallback when event session directory is unavailable.

## Summary

OpenCode's documented extension point is its server-side JavaScript/TypeScript plugin API. A Swayve integration can live as a local plugin under `.opencode/plugins/` (or as a configured package/path); it need not depend on shell lifecycle hooks. The plugin can subscribe to the event stream and clean up its subscription on unload. Before implementation, verify the exact current event names and payloads against OpenCode's published API/types and a supported OpenCode release.

## Findings

- OpenCode discovers direct `.ts` and `.js` plugin files and immediate plugin package directories under project `.opencode/plugins/`, as well as the global config plugin directory. Other paths can be configured in `opencode.json(c)`. A project-root `plugins/` directory is not auto-discovered. [Plugins](https://opencode.ai/docs/plugins)
- A server plugin is a JavaScript/TypeScript module using `@opencode/plugin`; its `setup` gets a server-client-like context and may return cleanup logic. [Build plugins](https://opencode.ai/docs/build/plugins), [Server plugin API](https://opencode.ai/docs/build/plugins/server)
- The server plugin event API exposes `ctx.event.subscribe({ types, signal })`, an async iterable of typed public events. The docs explicitly say to abort the stream during plugin cleanup, and server plugins do not receive `server.connected`. [Server plugin API](https://opencode.ai/docs/build/plugins/server)
- Plugins also expose runtime hooks and registration/disposal APIs. These are stronger interception points than events, but status reporting should prefer passive events unless an actual interception behavior is required. [Server plugin API](https://opencode.ai/docs/build/plugins/server)
- OpenCode's plugin docs list configurable local paths and plugin package installation through the `opencode plugin` CLI. [Plugins](https://opencode.ai/docs/plugins)

## Relevance to Swayve

Swayve already has a newline-delimited JSON Unix socket contract and `agent.register`, `agent.update_status`, and `daemon.focus` messages. Existing integrations are intentionally narrow:

- Claude provides a settings hook installer and a wrapper, with lifecycle updates and a mock Unix socket test (`integrations/claude/`).
- Antigravity provides a dependency-free Node sidecar supporting direct commands and hook mode; it resolves Swayve/Antigravity environment fallbacks, sends one-way IPC, documents setup, and has Node tests (`integrations/antigravity/`).

An OpenCode plugin could map OpenCode session/tool events directly to Swayve messages and avoid polling or a separate wrapper. Candidate risks to resolve are stable session-to-agent identity, project/workspace identity, process/PID and window `app_id` semantics, child-session relationships, event-to-status mapping, and behavior when no socket/agent identity is available. The plugin must avoid blocking or destabilizing OpenCode on best-effort Swayve IPC failures.

## Primary sources

- OpenCode Plugins: https://opencode.ai/docs/plugins
- OpenCode Build Plugins: https://opencode.ai/docs/build/plugins
- OpenCode Server Plugin API: https://opencode.ai/docs/build/plugins/server
- OpenCode API event schemas: https://opencode.ai/docs/api#schema-OpenCodeEvent
- Swayve Claude integration: `integrations/claude/README.md`, `integrations/claude/install-hooks.sh`, `integrations/claude/test-hooks.sh`
- Swayve Antigravity integration: `integrations/antigravity/README.md`, `integrations/antigravity/sidecar.mjs`, `integrations/antigravity/sidecar.test.mjs`
- Swayve goal inventory: `planning/GOALS.md`
