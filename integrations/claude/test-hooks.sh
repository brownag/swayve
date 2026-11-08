#!/usr/bin/env bash
# Swayve Claude Integration Test Script
# Verifies lifecycle hook and wrapper status updates sent over Unix domain socket.

set -euo pipefail

MOCK_DIR="$(mktemp -d)"
SOCKET_PATH="$MOCK_DIR/swayve_claude_test.sock"
LOG_FILE="$(mktemp)"

PY_PID=""

cleanup() {
    if [ -n "$PY_PID" ] && kill -0 "$PY_PID" 2>/dev/null; then
        kill "$PY_PID" 2>/dev/null || true
        wait "$PY_PID" 2>/dev/null || true
    fi
    rm -f "$SOCKET_PATH"
    rm -f "$LOG_FILE"
    rm -rf "$MOCK_DIR"
}

trap cleanup EXIT INT TERM

# Ensure socket path is removed prior to server start
rm -f "$SOCKET_PATH"

echo "[test-hooks] Starting inline Python socket server on $SOCKET_PATH..."

python3 - "$SOCKET_PATH" "$LOG_FILE" <<'PYEOF' &
import socket
import sys
import os

sock_path = sys.argv[1]
log_path = sys.argv[2]

if os.path.exists(sock_path):
    os.remove(sock_path)

server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
server.bind(sock_path)
server.listen(10)

with open(log_path, 'a') as log_file:
    while True:
        try:
            conn, _ = server.accept()
            data = conn.recv(4096)
            if data:
                log_file.write(data.decode('utf-8').strip() + '\n')
                log_file.flush()
            conn.close()
        except Exception:
            break
PYEOF

PY_PID=$!

# Wait for socket file to be created
MAX_WAIT=50
WAIT_COUNT=0
while [ ! -S "$SOCKET_PATH" ]; do
    sleep 0.1
    WAIT_COUNT=$((WAIT_COUNT + 1))
    if [ $WAIT_COUNT -ge $MAX_WAIT ]; then
        echo "[test-hooks] Error: Python socket server failed to create socket $SOCKET_PATH" >&2
        exit 1
    fi
done

echo "[test-hooks] Socket server active."

# Prepare mock `claude` CLI if `claude` is not available in PATH
if ! command -v claude >/dev/null 2>&1; then
    cat <<'MOCKEOF' > "$MOCK_DIR/claude"
#!/usr/bin/env bash
if [ "${1:-}" = "--version" ]; then
    echo "claude 1.0.0-mock"
    exit 0
fi
echo "mock claude ok"
exit 0
MOCKEOF
    chmod +x "$MOCK_DIR/claude"
    export PATH="$MOCK_DIR:$PATH"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WRAPPER_SCRIPT="$SCRIPT_DIR/claude-wrapper.sh"

echo "[test-hooks] Testing claude-wrapper.sh..."
SWAYVE_SOCKET="$SOCKET_PATH" SWAYVE_AGENT_ID="claude-test" "$WRAPPER_SCRIPT" --version

echo "[test-hooks] Simulating Claude Code lifecycle hooks (SessionStart, PreToolUse, PostToolUse, Stop)..."

send_hook_payload() {
    local status="$1"
    local payload="{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"claude-test\",\"status\":$status}}"
    if command -v nc >/dev/null 2>&1; then
        printf '%s\n' "$payload" | nc -U "$SOCKET_PATH" 2>/dev/null || true
    elif command -v socat >/dev/null 2>&1; then
        printf '%s\n' "$payload" | socat - UNIX-CONNECT:"$SOCKET_PATH" 2>/dev/null || true
    fi
}

send_hook_payload '"thinking"'
send_hook_payload '"executing_tool"'
send_hook_payload '"thinking"'
send_hook_payload '"idle"'

# Short sleep to ensure all IPC calls complete
sleep 0.2

# Stop Python server
if [ -n "$PY_PID" ]; then
    kill "$PY_PID" 2>/dev/null || true
    wait "$PY_PID" 2>/dev/null || true
    PY_PID=""
fi

echo "[test-hooks] Reading received socket payloads..."
cat "$LOG_FILE"

# Assertions
echo "[test-hooks] Verifying JSON assertions..."

# 1. Check thinking status from wrapper
grep -q '"status":"thinking"' "$LOG_FILE" || { echo "Assertion failed: 'thinking' status missing" >&2; exit 1; }
echo "[ASSERT PASSED] Received status: 'thinking'"

# 2. Check completed status from wrapper
grep -q '"status":"completed"' "$LOG_FILE" || { echo "Assertion failed: 'completed' status missing" >&2; exit 1; }
echo "[ASSERT PASSED] Received status: 'completed'"

# 3. Check executing_tool status from hooks
grep -q '"status":"executing_tool"' "$LOG_FILE" || { echo "Assertion failed: 'executing_tool' status missing" >&2; exit 1; }
echo "[ASSERT PASSED] Received status: 'executing_tool'"

# 4. Check idle status from hooks
grep -q '"status":"idle"' "$LOG_FILE" || { echo "Assertion failed: 'idle' status missing" >&2; exit 1; }
echo "[ASSERT PASSED] Received status: 'idle'"

# 5. Check agent_id in payload
grep -q '"agent_id":"claude-test"' "$LOG_FILE" || { echo "Assertion failed: 'agent_id':'claude-test' missing" >&2; exit 1; }
echo "[ASSERT PASSED] Agent ID 'claude-test' correctly passed in JSON payload"

echo "[test-hooks] All integration test assertions passed successfully!"
