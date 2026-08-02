#!/usr/bin/env bash
# Swayve Claude Code Lifecycle Hook Installer
# Installs or merges Swayve lifecycle hooks into Claude Code settings.json (~/.claude/settings.json or .claude/settings.json)

set -euo pipefail

TARGET_FILE=""

# Parse flags/arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -g|--global)
            TARGET_FILE="$HOME/.claude/settings.json"
            shift
            ;;
        -l|--local)
            TARGET_FILE="./.claude/settings.json"
            shift
            ;;
        -f|--file)
            TARGET_FILE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--global | --local | --file <path>]"
            echo "  --global, -g : Target ~/.claude/settings.json"
            echo "  --local, -l  : Target ./.claude/settings.json"
            echo "  --file, -f   : Target specific file path"
            exit 0
            ;;
        *)
            echo "Error: Unknown argument '$1'" >&2
            exit 1
            ;;
    esac
done

# Auto-detect target file if not specified
if [ -z "$TARGET_FILE" ]; then
    if [ -f "./.claude/settings.json" ]; then
        TARGET_FILE="./.claude/settings.json"
    elif [ -f "$HOME/.claude/settings.json" ]; then
        TARGET_FILE="$HOME/.claude/settings.json"
    else
        TARGET_FILE="$HOME/.claude/settings.json"
    fi
fi

# Target directory
TARGET_DIR="$(dirname "$TARGET_FILE")"
mkdir -p "$TARGET_DIR"

# Swayve hook commands
HOOK_CMD_THINKING='if [ -n "$SWAYVE_SOCKET" ] && [ -n "$SWAYVE_AGENT_ID" ] && [ -S "$SWAYVE_SOCKET" ]; then printf '\''{"method":"agent.update_status","params":{"agent_id":"%s","status":"thinking"}}\n'\'' "$SWAYVE_AGENT_ID" | nc -U "$SWAYVE_SOCKET" 2>/dev/null || true; fi'
HOOK_CMD_TOOL='if [ -n "$SWAYVE_SOCKET" ] && [ -n "$SWAYVE_AGENT_ID" ] && [ -S "$SWAYVE_SOCKET" ]; then printf '\''{"method":"agent.update_status","params":{"agent_id":"%s","status":"executing_tool"}}\n'\'' "$SWAYVE_AGENT_ID" | nc -U "$SWAYVE_SOCKET" 2>/dev/null || true; fi'
HOOK_CMD_IDLE='if [ -n "$SWAYVE_SOCKET" ] && [ -n "$SWAYVE_AGENT_ID" ] && [ -S "$SWAYVE_SOCKET" ]; then printf '\''{"method":"agent.update_status","params":{"agent_id":"%s","status":"idle"}}\n'\'' "$SWAYVE_AGENT_ID" | nc -U "$SWAYVE_SOCKET" 2>/dev/null || true; fi'

echo "[swayve-hooks] Installing Claude Code lifecycle hooks to: $TARGET_FILE"

if command -v jq >/dev/null 2>&1; then
    if [ -f "$TARGET_FILE" ] && [ -s "$TARGET_FILE" ]; then
        jq --arg thinking "$HOOK_CMD_THINKING" \
           --arg tool "$HOOK_CMD_TOOL" \
           --arg idle "$HOOK_CMD_IDLE" \
           '.hooks.SessionStart = [{matcher: "", hooks: [{type: "command", command: $thinking}]}] |
            .hooks.PreToolUse = [{matcher: "", hooks: [{type: "command", command: $tool}]}] |
            .hooks.PostToolUse = [{matcher: "", hooks: [{type: "command", command: $thinking}]}] |
            .hooks.Stop = [{matcher: "", hooks: [{type: "command", command: $idle}]}]' "$TARGET_FILE" > "$TARGET_FILE.tmp" && mv "$TARGET_FILE.tmp" "$TARGET_FILE"
    else
        jq -n \
           --arg thinking "$HOOK_CMD_THINKING" \
           --arg tool "$HOOK_CMD_TOOL" \
           --arg idle "$HOOK_CMD_IDLE" \
           '{
             hooks: {
               SessionStart: [{matcher: "", hooks: [{type: "command", command: $thinking}]}],
               PreToolUse: [{matcher: "", hooks: [{type: "command", command: $tool}]}],
               PostToolUse: [{matcher: "", hooks: [{type: "command", command: $thinking}]}],
               Stop: [{matcher: "", hooks: [{type: "command", command: $idle}]}]
             }
           }' > "$TARGET_FILE"
    fi
elif command -v python3 >/dev/null 2>&1; then
    python3 - "$TARGET_FILE" "$HOOK_CMD_THINKING" "$HOOK_CMD_TOOL" "$HOOK_CMD_IDLE" <<'PYEOF'
import sys, json, os

target_file = sys.argv[1]
thinking_cmd = sys.argv[2]
tool_cmd = sys.argv[3]
idle_cmd = sys.argv[4]

hooks_data = {
    "SessionStart": [{"matcher": "", "hooks": [{"type": "command", "command": thinking_cmd}]}],
    "PreToolUse": [{"matcher": "", "hooks": [{"type": "command", "command": tool_cmd}]}],
    "PostToolUse": [{"matcher": "", "hooks": [{"type": "command", "command": thinking_cmd}]}],
    "Stop": [{"matcher": "", "hooks": [{"type": "command", "command": idle_cmd}]}]
}

data = {}
if os.path.exists(target_file) and os.path.getsize(target_file) > 0:
    try:
        with open(target_file, 'r') as f:
            data = json.load(f)
    except Exception as e:
        sys.stderr.write(f"Warning: Failed to parse existing JSON: {e}\n")
        data = {}

if "hooks" not in data or not isinstance(data["hooks"], dict):
    data["hooks"] = {}

for event, hook_list in hooks_data.items():
    data["hooks"][event] = hook_list

with open(target_file, 'w') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
PYEOF
else
    echo "[swayve-hooks] Warning: Neither 'jq' nor 'python3' available." >&2
    echo "[swayve-hooks] Unable to safely merge JSON. Please install jq or python3." >&2
    exit 1
fi

echo "[swayve-hooks] Hooks successfully installed."
