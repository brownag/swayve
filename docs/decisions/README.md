# Architecture Decision Records

Ordered by dependency: each record depends only on records above it.

| ID | Title | Status | Date | Supersedes | Superseded By | Context |
|---|---|---|---|---|---|---|
| [0001](0001-sway-as-window-management-backend.md) | Use Sway as the Window-Management Backend | Accepted | 2026-07-26 | - | - | Initial commit |
| [0002](0002-rust-tokio-daemon.md) | Implement the Daemon in Rust on Tokio with swayipc-async | Accepted | 2026-07-26 | - | - | Initial commit |
| [0003](0003-daemon-owns-agent-state.md) | Daemon Owns Agent State; Clients Route Through Its Socket | Accepted | 2026-07-29 | - | - | Wave 4 |
| [0004](0004-newline-delimited-json-ipc.md) | Newline-Delimited JSON IPC with Method-Tagged Messages | Accepted | 2026-07-28 | - | - | Waves 2-3 (`43497ae`, `1410acc`) |
| [0005](0005-layout-as-command-plans-behind-registry-trait.md) | Layout Emits Sway Command Plans Behind an AgentRegistry Trait | Accepted | 2026-07-27 | - | - | Wave 1, Wave 3 |
| [0006](0006-agent-identity-via-app-id-and-env.md) | Agent Identity via Unique app_id and Injected Environment | Accepted | 2026-08-01 | - | - | Wave 6 |
| [0007](0007-agent-hierarchy-and-sibling-focus.md) | Agent Hierarchy via parent_id and Sway Children Marks; Sibling-Cycle Focus | Accepted | 2026-07-29 | - | - | Waves 4-5 |
| [0008](0008-push-status-subscription-for-hud.md) | Push Agent Status to Subscribers over the Socket for the HUD | Accepted | 2026-08-02 | - | - | Wave 12 |
| [0009](0009-thin-socket-integrations.md) | Agent Integrations Are Thin Socket Clients, Not SDKs | Accepted | 2026-08-01 | - | - | Waves 7-11 |
| [0010](0010-toml-config-in-home-config-dir.md) | TOML Configuration at ~/.config/swayve/config.toml with Serde Defaults | Accepted | 2026-07-28 | - | - | Wave 1 (`46e5c13`) |
| [0011](0011-focus-state-from-sway-window-events.md) | Focus Cycle State Is Fed by Sway Window-Focus Events | Accepted | 2026-11-02 | - | - | Orchestration 13 |
| [0012](0012-runtime-socket-private-dir-contract.md) | Runtime Socket Lives in a Private XDG Runtime Directory; SWAYVE_SOCKET Is an Explicit Override | Accepted | 2026-11-08 | - | - | Orchestration 19 |
| [0013](0013-opt-in-diff-status-subscription.md) | Opt-In Diff Status Subscription Alongside the Full-List Stream | Accepted | 2026-11-08 | - | - | Orchestration 20 |
| [0014](0014-skip-undecodable-ipc-lines-keep-client.md) | Skip Non-UTF-8 IPC Lines and Keep the Client Connected | Accepted | 2026-11-08 | - | - | Orchestration 21 |
| [0015](0015-ipc-rate-limit-drop-and-log.md) | Per-Client IPC Rate Limit: Drop and Log | Accepted | 2026-11-08 | - | - | Orchestration 22 |

## Not recorded as ADRs

* **Design docs** (`Sway Multi Agent Architecture Design.md`, `Sway Multi-Agent Viewer Architecture.md`): their JSON-RPC 2.0 protocol and `sway-agentd` naming are superseded in practice by [0004](0004-newline-delimited-json-ipc.md) and [0002](0002-rust-tokio-daemon.md). They are proposals, not decisions, and are not superseded formally.
* **Test layout and CI**: recorded in `AGENTS.md` (Test layout and CI bullets). Not an ADR, because it is easy to change.
* **Per-task micro-plan details** (`planning/tasks/`): transient, per the heuristic filter.
