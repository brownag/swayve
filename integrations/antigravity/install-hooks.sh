#!/usr/bin/env bash
# Swayve Antigravity Lifecycle Hook Installer
# Merges Swayve sidecar hooks into Antigravity's hooks.json (~/.gemini/config/hooks.json by default).

set -euo pipefail

TARGET_FILE="$HOME/.gemini/config/hooks.json"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -f|--file)
            TARGET_FILE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--file <path>]"
            echo "  --file, -f : Target hooks.json (default ~/.gemini/config/hooks.json)"
            exit 0
            ;;
        *)
            echo "Error: Unknown argument '$1'" >&2
            exit 1
            ;;
    esac
done

SIDECAR="$(cd "$(dirname "$0")" && pwd)/sidecar.mjs"
mkdir -p "$(dirname "$TARGET_FILE")"

# Replace only the "swayve-sidecar" key; keep every other hook the user has.
python3 - "$TARGET_FILE" "$SIDECAR" <<'PY'
import json, os, sys

path, sidecar = sys.argv[1], sys.argv[2]
doc = {}
if os.path.exists(path) and os.path.getsize(path) > 0:
    with open(path) as f:
        doc = json.load(f)

def cmd(event):
    return f"exec node {sidecar} hook {event}"

doc["swayve-sidecar"] = {
    "SessionStart": [{"type": "command", "command": cmd("SessionStart")}],
    "PreToolUse": [{"matcher": "*", "hooks": [{"type": "command", "command": cmd("PreToolUse")}]}],
    "PostToolUse": [{"matcher": "*", "hooks": [{"type": "command", "command": cmd("PostToolUse")}]}],
    "Stop": [{"type": "command", "command": cmd("Stop")}],
}

with open(path, "w") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print(f"[swayve-hooks] Antigravity hooks written to: {path}")
PY
