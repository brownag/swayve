---
id: "0009"
title: "Agent Integrations Are Thin Socket Clients, Not SDKs"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-08-01
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Commits 78ad0d8, 7cc0634, 8be31a5, 8cc2e55, 095010e; integrations/claude/, integrations/pi/"
---

# 9. Agent Integrations Are Thin Socket Clients, Not SDKs

## Context and Problem Statement
Claude Code and Pi need to report status to swayve. Each agent runtime has its own hook or extension mechanism, and none is a shared library.

## Decision Drivers
* Integration must work without installing a language SDK into the agent.
* Claude Code runs shell hooks; Pi runs TypeScript.
* Hook installation must merge with existing user settings.

## Considered Options
1. A per-runtime SDK that wraps the protocol.
2. Shell hooks and a wrapper that send one JSON line with `nc -U` or `socat`, plus a TypeScript client for Pi.
3. Make each agent runtime talk Sway directly.

## Decision Outcome
Chosen option: "Shell hooks and a wrapper using `nc -U` or `socat`; a TypeScript client for Pi", because the protocol is one line per message ([0004](0004-newline-delimited-json-ipc.md)), so any runtime that can run a shell command can integrate.

Concrete pieces:
* `integrations/claude/install-hooks.sh` merges hooks into `~/.claude/settings.json` or `./.claude/settings.json`. It uses `jq` and falls back to `python3`.
* Claude hooks use the nested matcher schema. Commit `8be31a5` fixed the schema, and `8cc2e55` overwrites hook arrays by key to avoid `jq` merge conflicts.
* `claude-wrapper.sh` reports `thinking`, `completed`, and `failed` by exit code.
* `integrations/pi/src/swayve-client.ts` connects with `net.createConnection`, using `SWAYVE_SOCKET`.

### Consequences
* **Positive**: No runtime dependency. Hooks are inspectable shell.
* **Negative**: Shell quoting is fragile (see `8be31a5`, `8cc2e55`). Integrations depend on `nc` or `socat` and `jq` or `python3` being installed.
* **Negative**: Settings-file schema is owned by the agent runtime. A change upstream breaks the installer.

## Validation & Compliance
`integrations/claude/test-hooks.sh` starts a mock socket server and checks hook messages. `integrations/pi/tests/swayve-client.test.ts` covers the Pi client.
