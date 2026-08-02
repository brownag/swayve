# Claude Code Integration for Swayve

This integration enables **Claude Code** to dynamically report session and tool execution states back to the **Swayve** daemon using Claude Code's native lifecycle hooks and shell wrapper scripts.

---

## Overview

Swayve tracks agent windows, status indicators (e.g. `thinking`, `executing_tool`, `idle`, `completed`, `failed`), and window focus order in the Sway tiling window manager.

Claude Code exposes lifecycle hooks (`SessionStart`, `PreToolUse`, `PostToolUse`, `Stop`) in `.claude/settings.json` (or `~/.claude/settings.json`). By attaching status reporting commands to these lifecycle hooks, Claude Code automatically notifies Swayve over Unix socket (`$SWAYVE_SOCKET`) whenever it changes state.

---

## Lifecycle Hooks & Status Mapping

| Hook Event | Trigger Condition | Swayve Agent Status | IPC Method |
| :--- | :--- | :--- | :--- |
| `SessionStart` | Claude Code session starts | `thinking` | `agent.update_status` |
| `PreToolUse` | Claude begins executing a tool | `executing_tool` | `agent.update_status` |
| `PostToolUse` | Claude finishes tool execution | `thinking` | `agent.update_status` |
| `Stop` | Claude finishes turn / goes idle | `idle` | `agent.update_status` |

---

## Installing Hooks (`install-hooks.sh`)

The `install-hooks.sh` script installs or merges Swayve lifecycle hooks into your Claude settings file.

### Usage

```bash
# Auto-detect settings file (checks ./.claude/settings.json, then ~/.claude/settings.json)
./integrations/claude/install-hooks.sh

# Target global configuration (~/.claude/settings.json)
./integrations/claude/install-hooks.sh --global

# Target local project configuration (./.claude/settings.json)
./integrations/claude/install-hooks.sh --local

# Target a specific configuration file
./integrations/claude/install-hooks.sh --file /path/to/settings.json
```

---

## Generated Configuration (`settings.json`)

When installed, your `.claude/settings.json` will contain hooks configured like so:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "command": "if [ -n \"$SWAYVE_SOCKET\" ] && [ -n \"$SWAYVE_AGENT_ID\" ] && [ -S \"$SWAYVE_SOCKET\" ]; then printf '{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"%s\",\"status\":\"thinking\"}}\\n' \"$SWAYVE_AGENT_ID\" | nc -U \"$SWAYVE_SOCKET\" 2>/dev/null || true; fi"
      }
    ],
    "PreToolUse": [
      {
        "command": "if [ -n \"$SWAYVE_SOCKET\" ] && [ -n \"$SWAYVE_AGENT_ID\" ] && [ -S \"$SWAYVE_SOCKET\" ]; then printf '{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"%s\",\"status\":\"executing_tool\"}}\\n' \"$SWAYVE_AGENT_ID\" | nc -U \"$SWAYVE_SOCKET\" 2>/dev/null || true; fi"
      }
    ],
    "PostToolUse": [
      {
        "command": "if [ -n \"$SWAYVE_SOCKET\" ] && [ -n \"$SWAYVE_AGENT_ID\" ] && [ -S \"$SWAYVE_SOCKET\" ]; then printf '{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"%s\",\"status\":\"thinking\"}}\\n' \"$SWAYVE_AGENT_ID\" | nc -U \"$SWAYVE_SOCKET\" 2>/dev/null || true; fi"
      }
    ],
    "Stop": [
      {
        "command": "if [ -n \"$SWAYVE_SOCKET\" ] && [ -n \"$SWAYVE_AGENT_ID\" ] && [ -S \"$SWAYVE_SOCKET\" ]; then printf '{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"%s\",\"status\":\"idle\"}}\\n' \"$SWAYVE_AGENT_ID\" | nc -U \"$SWAYVE_SOCKET\" 2>/dev/null || true; fi"
      }
    ]
  }
}
```

---

## CLI Wrapper Script (`claude-wrapper.sh`)

In addition to lifecycle hooks, you can launch Claude Code via `claude-wrapper.sh` to report binary startup (`thinking`) and exit status (`completed` or `failed`):

```bash
# Run wrapper script directly
./integrations/claude/claude-wrapper.sh --dangerously-skip-permissions
```
