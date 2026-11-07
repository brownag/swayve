# Execution Plan: Wave 12 — Interactive HUD Dashboard

## Goal & Scope

**Goal**: Implement the Interactive HUD Dashboard overlay (`swayve panel`). Support a live-streaming daemon status subscription protocol and develop an interactive, real-time updating terminal UI (TUI) showing the running tree of subagents, their current states, and their log streams in the workspace.

**In scope:**
- Exposing a live status update event-stream loop in `src/daemon.rs` for connected HUD panel socket clients.
- Refactoring `swayve status --watch` or the TUI dashboard loop in `src/cli.rs` to render a tree view with dynamic ANSI refresh animations.
- Ensuring the dashboard remains open in a floating scratchpad window when triggered via `swayve panel toggle`.

**Out of scope:**
- Rendering graphical dashboards outside standard terminal shell interfaces.

## Decision Log

- **D1: Socket Status Stream**: The daemon socket will support a new subscription variant (e.g. `daemon.subscribe_status`) that broadcasts agent tree state updates to connected clients in real-time.
- **D2: ANSI Tree Representation**: Render parent-child hierarchies using ANSI box-drawing characters (e.g., `├─`, `└─`) to keep resource consumption minimal.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 12** | **12.1** | Add daemon status subscription broadcast | `src/types.rs`, `src/daemon.rs` | None |
| | **12.2** | Build interactive HUD dashboard CLI client | `src/cli.rs` | Task 12.1 |
| | **12.3** | Verify dashboard CLI stream assertions | `tests/integration_tests.rs` | Task 12.2 |

---

## Task Specifications

### Task 12.1: Add Daemon Status Subscription Broadcast

**ID**: `12.1`  
**Repo**: `swayve`  
**Files-in-scope**: `src/types.rs`, `src/daemon.rs`  
**Depends-on**: None  

**Spec prose**:
1. In `src/types.rs` (`AgentIPCMessage`), add a subscription variant:
   ```rust
   #[serde(rename = "daemon.subscribe")]
   SubscribeStatus,
   ```
2. In `src/daemon.rs`, maintain a registry of active subscriber broadcast channels (`mpsc::Sender<String>`).
3. Whenever an agent registers or changes status, serialize the full active agent list/tree and broadcast it to all active subscribers.

**Acceptance**: `cargo check` compiles without warnings.

---

### Task 12.2: Build Interactive HUD Dashboard CLI Client

**ID**: `12.2`  
**Repo**: `swayve`  
**Files-in-scope**: `src/cli.rs`  
**Depends-on**: Task 12.1  

**Spec prose**:
1. Implement the dashboard renderer in `src/cli.rs` under `Commands::Status` when `watch` flag or a custom panel interactive command is run.
2. The client connects to `/tmp/swayve.sock` and sends `{"method":"daemon.subscribe"}`.
3. In a loop, read JSON status broadcasts, clear the terminal screen (`\x1B[2J\x1B[H`), and print a beautiful, real-time agent hierarchy tree:
   - Tree nodes: Display Agent ID, Status, Project, and app_id.
   - Status color-coding: Use standard terminal ANSI escape codes to match borders (e.g. orange text for `Thinking`, flashing red/bold for `AwaitingInput`, green for `Completed`).
4. Update `Commands::Panel` to spawn `swayve status --watch` instead of the static `swayve status`.

**Acceptance**: `cargo test` compiles successfully.

---

### Task 12.3: Verify Dashboard CLI Stream Assertions

**ID**: `12.3`  
**Repo**: `swayve`  
**Files-in-scope**: `tests/integration_tests.rs`  
**Depends-on**: Task 12.2  

**Spec prose**:
Add integration test `test_dashboard_subscription_stream` in `tests/integration_tests.rs` to verify that when a client connects and sends `daemon.subscribe`, the daemon correctly streams the serialized active agent tree state and updates are triggered upon subsequent agent status changes.

**Acceptance**: All integration tests pass.
