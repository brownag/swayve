# OpenCode Integration Research

Last researched: 2026-11-17

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
- Swayve Claude integration: `integrations/claude/README.md`, `integrations/claude/install-hooks.sh`, `integrations/claude/test-hooks.sh`
- Swayve Antigravity integration: `integrations/antigravity/README.md`, `integrations/antigravity/sidecar.mjs`, `integrations/antigravity/sidecar.test.mjs`
- Swayve goal inventory: `planning/GOALS.md`
