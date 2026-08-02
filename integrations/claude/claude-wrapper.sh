#!/usr/bin/env bash
# Swayve Claude CLI Wrapper
# Wraps the `claude` CLI command to report startup ('thinking') and exit ('completed' / 'failed') status to Swayve daemon.

set -u

# Function to send JSON payload over Swayve Unix domain socket
_send_swayve_status() {
    local status_json="$1"
    if [ -n "${SWAYVE_SOCKET:-}" ] && [ -n "${SWAYVE_AGENT_ID:-}" ] && [ -S "$SWAYVE_SOCKET" ]; then
        local payload="{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"$SWAYVE_AGENT_ID\",\"status\":$status_json}}"
        if command -v nc >/dev/null 2>&1; then
            printf '%s\n' "$payload" | nc -U "$SWAYVE_SOCKET" 2>/dev/null || true
        elif command -v socat >/dev/null 2>&1; then
            printf '%s\n' "$payload" | socat - UNIX-CONNECT:"$SWAYVE_SOCKET" 2>/dev/null || true
        fi
    fi
}

# Report startup status
_send_swayve_status '"thinking"'

# Run claude CLI with passed arguments
EXIT_CODE=0
if command -v claude >/dev/null 2>&1; then
    claude "$@" || EXIT_CODE=$?
else
    echo "[claude-wrapper] Error: 'claude' command not found in PATH." >&2
    EXIT_CODE=127
fi

# Report exit status
if [ "$EXIT_CODE" -eq 0 ]; then
    _send_swayve_status '"completed"'
else
    _send_swayve_status "{\"failed\":\"Process exited with status code $EXIT_CODE\"}"
fi

exit "$EXIT_CODE"
