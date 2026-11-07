# Execution Plan: Wave 4 — Hierarchical Focus & CLI-Daemon Coordination

## Goal & Scope

**Goal**: Implement hierarchical focus logic that cycles through sibling agent windows in a parent-child tree. Wire the CLI focus command to coordinate with the daemon via Unix socket for consistent state management. Refactor the daemon's overstuffed WindowCreated event handler into smaller, focused private helpers.

**In scope:**
- Implement hierarchical focus algorithm: given current focused window, determine if it's a parent or child, and rotate to next sibling (or parent if at end of siblings)
- Wire CLI `focus --cycle` command to send a focus request to daemon via Unix socket IPC
- Extract daemon WindowCreated handler into private sub-functions (register_agent_window, apply_process_tree_auditing, delegate_to_layout)
- Add unit tests for hierarchical focus logic (mock agent state, verify sibling traversal)
- Add integration test for full CLI→daemon→Sway focus flow

**Out of scope:**
- Multi-level nesting focus (focus grandchildren of grandparent); focus next parent sibling at root level
- Keybinding integration (focus from sway config) — future Wave
- Refactoring UpdateStatus handler — Wave 5

## Decision Log

- **D1: Hierarchical Focus Behavior** (2026-07-29): Sibling cycling mode. When focused on a window, pressing focus cycles to the next sibling. If in a parent, cycles children. If in a child, cycles to next sibling or parent. (vs. simple flat rotation)
- **D2: CLI-Daemon Coordination** (2026-07-29): CLI sends focus request to daemon via socket IPC. Daemon maintains focus state and determines sibling order. CLI does not execute focus directly. (vs. CLI independent)
- **D3: Handler Refactoring Scope** (2026-07-29): Extract WindowCreated into private sub-helpers (register_agent_window, apply_process_tree_auditing, delegate_to_layout). Pure refactoring, no behavior change. (vs. defer to Wave 5)
- **D4: Testing Acceptance** (2026-07-29): Unit tests for focus algorithm + integration test for CLI→daemon→Sway flow. Full suite must pass at wave end.
- **D5: Refactoring Visibility** (2026-07-29): Private internal helpers only, used within daemon.rs. Do not expose as public methods. (vs. public methods on DaemonState)

## Hazard Notes

- **daemon.rs edited in three tasks**: 4.1 (add Focus logic), 4.2 (refactor WindowCreated), 4.3 (CLI IPC wiring). Risk of merge conflicts. Mitigation: sequential waves, explicit grep anchors, micro-plans approved before edits.
- **Focus state tracking**: Daemon must track "currently focused window" across events. Can be derived from agent state (process of elimination); no new fields needed if carefully designed.
- **IPC message design**: Adding Focus variant to AgentIPCMessage. New variant is ~1 line; serde tag handling is automatic. No new dependencies.
- **Refactoring must preserve behavior**: WindowCreated splitting must not change LayoutManager invocation or process-tree audit logic. Each helper must be unit-testable in isolation.
- **CLI fallback**: If daemon socket unreachable, CLI can still focus directly via Sway (graceful degradation).

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 4.1** | **4.1** | Implement hierarchical focus algorithm in daemon Focus handler | src/daemon.rs | Wave 3 complete |
| **Wave 4.2** | **4.2** | Refactor WindowCreated handler into private sub-helpers | src/daemon.rs | Task 4.1 |
| **Wave 4.3** | **4.3** | Wire CLI focus command to daemon via socket IPC | src/cli.rs, src/daemon.rs, src/types.rs | Task 4.2 |
| **Wave 4.4** | **4.4** | Integration test for CLI→daemon→Sway focus flow | tests/integration_tests.rs | Task 4.3 |

---

## Task Specifications

### Task 4.1: Implement Hierarchical Focus Algorithm

**ID**: `4.1`  
**Repo**: `swayve`  
**Files-in-scope**: `src/daemon.rs`

**Depends-on**: Wave 3 complete

**Spec prose**:

Replace the Focus handler in `src/daemon.rs` (lines 243–265) with hierarchical sibling-cycling logic:

**Current behavior** (naive): focuses the first agent window unconditionally.

**New behavior**: 
1. Collect all agent windows currently active (those with container_id set)
2. Determine the "currently focused" window from state (can derive: find the one with highest focus hint or query Sway if needed)
3. Apply sibling cycling:
   - If focused window is a **parent**: cycle through its children (agents with `parent_id == focused.id`), then back to parent
   - If focused window is a **child**: cycle to next sibling (sort children by con_id), or if at end, back to parent
   - If focused window is an **orphan** (no parent, no children): no focus change
   - If no focused window (or only one agent total): focus the first agent
4. Send `SwayCommand::FocusWindow { mark }` for the target to Sway

**Algorithm pseudocode**:
```
if all_agents.empty() => return
if no_agent_currently_focused => focus all_agents[0]

current = get_focused_agent()
if current.parent_id == None and current has children => 
  focus first child
else if current.parent_id == Some(pid) =>
  siblings = filter_agents(agent.parent_id == Some(pid))
  current_idx = siblings.find_index(current)
  if current_idx < siblings.len() - 1 =>
    focus siblings[current_idx + 1]
  else =>
    focus parent (pid)
else =>
  focus current (no-op, already focused)
```

Implementation location: Replace the match arm `DaemonEvent::Focus => { ... }` in `run_daemon_loops()` (lines 243–265).

**Constraints**:
- Do not change WindowCreated or WindowDestroyed handlers
- Do not call LayoutManager; focus is orthogonal to layout
- Do not add new fields to DaemonState; derive focus state from existing relationships
- Must handle edge cases: empty workspace, single agent, parent with no children yet

**Grep Anchors**: 
- `DaemonEvent::Focus =>`
- `// Focus event handler` (existing comment)

**Acceptance**: `cargo check`

**Targeted Test**: `cargo test --lib daemon::tests::test_hierarchical_focus` — unit test with mock agent state, verify sibling cycling logic.

---

### Task 4.2: Refactor WindowCreated Handler into Private Sub-Helpers

**ID**: `4.2`  
**Repo**: `swayve`  
**Files-in-scope**: `src/daemon.rs`

**Depends-on**: Task 4.1

**Spec prose**:

The current WindowCreated handler (lines 180–232) handles four concerns in one match arm:
1. Find agent by app_id and set container_id
2. Update con_to_agent mapping
3. Perform process-tree auditing
4. Delegate to LayoutManager

Extract into three **private** helper functions to clarify responsibilities and enable unit testing:

**Helper 1: `register_agent_window()`**
```rust
fn register_agent_window(
    app_id: &str,
    con_id: i64,
    state: &mut DaemonState,
) -> Option<(AgentId, String, ProjectId, Option<AgentId>)> {
    // Returns: (agent_id, mark, project_id, parent_id) if agent found and registered
    // Updates: agent.container_id, con_to_agent map
}
```

**Helper 2: `apply_process_tree_auditing()`**
```rust
fn apply_process_tree_auditing(
    agent: &mut AgentWindow,
    con_pid: u32,
    pid_to_agent: &HashMap<u32, AgentId>,
    max_depth: usize,
) -> bool {
    // Auto-detect parent if missing; return true if detected, false otherwise
    // Uses existing find_registered_parent_ancestor() logic
    // Only runs if agent.parent_id.is_none() and config.process_tree_audit_enabled
}
```

**Helper 3: `delegate_to_layout()`**
```rust
async fn delegate_to_layout(
    con_id: i64,
    app_id: String,
    project_id: ProjectId,
    parent_id: Option<AgentId>,
    layout_manager: &LayoutManager,
    state: &DaemonState,
    sway_cmd_tx: &mpsc::Sender<SwayCommand>,
) {
    // Create LayoutAction and send commands to Sway
    let action = LayoutAction::WindowCreated {
        con_id,
        app_id,
        project_id,
        parent_id,
    };
    for cmd in layout_manager.compute_commands(action, state) {
        let _ = sway_cmd_tx.send(cmd).await;
    }
}
```

**Refactored handler**:
```rust
SwayEvent::WindowCreated { con_id, app_id, pid } => {
    if app_id.starts_with("agent.") {
        if let Some((agent_id, _mark, project_id, parent_id)) = 
            register_agent_window(&app_id, con_id, &mut state) 
        {
            if let Some(con_pid) = pid {
                if let Some(agent) = state.agents.get_mut(&agent_id) {
                    let _ = apply_process_tree_auditing(
                        agent,
                        con_pid,
                        &state.pid_to_agent,
                        state.config.daemon.max_ancestry_depth,
                    );
                }
            }
            delegate_to_layout(
                con_id,
                app_id,
                project_id,
                parent_id,
                &layout_manager,
                &state,
                &sway_cmd_tx,
            )
            .await;
        }
    }
}
```

**Constraints**:
- Refactoring must not change behavior; all three functions are translates of existing code
- All helpers remain **private** (no pub); only internal to daemon.rs
- No new dependencies or external crates
- Each helper must be testable via unit test with mock state

**Grep Anchors**: 
- `SwayEvent::WindowCreated { con_id, app_id, pid } =>`
- `fn register_agent_window`
- `fn apply_process_tree_auditing`
- `fn delegate_to_layout`

**Acceptance**: `cargo check`

**Targeted Test**: `cargo test --lib daemon::tests` — unit tests for each helper in isolation (mock state, verify behavior preservation).

---

### Task 4.3: Wire CLI Focus Command to Daemon via Socket IPC

**ID**: `4.3`  
**Repo**: `swayve`  
**Files-in-scope**: `src/cli.rs`, `src/daemon.rs`, `src/types.rs`

**Depends-on**: Task 4.2

**Spec prose**:

Currently, CLI `focus --cycle` (lines 303–346 in cli.rs) connects directly to Sway and rotates focus independently. Change this to route through daemon for consistent hierarchical focus tracking.

**In src/types.rs**:
Add a new Focus variant to `AgentIPCMessage` enum (after UpdateStatus, line ~59):
```rust
#[serde(rename = "daemon.focus")]
Focus,  // Signal daemon to execute hierarchical focus rotation
```

**In src/daemon.rs**:
Update `handle_client()` (lines 107–127) to accept the new Focus variant:
- Add match arm for `method = "daemon.focus"`: deserialize to Focus variant, send to `event_tx` as `DaemonEvent::Focus`
- No other changes needed; Focus handler already exists from Task 4.1

**In src/cli.rs**:
Replace `Commands::Focus { cycle }` handler (lines 303–346) to use socket IPC:

```rust
Commands::Focus { cycle } => {
    if cycle {
        if dry_run {
            println!("[focus] [DRY-RUN] Would send focus request to daemon.");
            return Ok(());
        }
        
        // Try to send focus request to daemon via socket
        match UnixStream::connect(socket_path).await {
            Ok(mut stream) => {
                let focus_msg = AgentIPCMessage::Focus;
                let payload = serde_json::to_string(&focus_msg)? + "\n";
                let _ = stream.write_all(payload.as_bytes()).await;
                println!("[focus] Sent focus request to daemon");
            }
            Err(_) => {
                // Fallback: if daemon unreachable, do standalone focus
                println!("[focus] Warning: Daemon unreachable. Falling back to standalone focus.");
                // (existing standalone focus logic here)
            }
        }
    }
}
```

**Constraints**:
- Do not change Focus handler logic in daemon.rs (that's Task 4.1)
- CLI must gracefully handle daemon socket unreachable (fallback to direct Sway connection)
- IPC message format must follow existing AgentIPCMessage pattern

**Grep Anchors**: 
- `Commands::Focus { cycle }` (in cli.rs)
- `AgentIPCMessage::Focus` (in types.rs)
- `"daemon.focus"` (serde tag)

**Acceptance**: `cargo check`

**Targeted Test**: `cargo test --lib` — verify serialization of Focus IPC message; verify daemon can deserialize and forward to event loop.

---

### Task 4.4: Integration Test for CLI→Daemon→Sway Focus Flow

**ID**: `4.4`  
**Repo**: `swayve`  
**Files-in-scope**: `tests/integration_tests.rs`

**Depends-on**: Task 4.3

**Spec prose**:

Add a new integration test `test_cli_focus_with_hierarchy()` that validates the end-to-end focus flow with parent-child hierarchy.

**Test setup** (follow `test_cli_parent_flag_to_layout` pattern, lines 348–462):
1. Allocate unique socket path (e.g., `/tmp/swayve_test_focus_hierarchy.sock`)
2. Spawn daemon listener and tasks
3. Set up event and command channels

**Test sequence**:
1. Register parent agent via socket (use existing recv_command pattern for IPC)
2. Register child agent with parent_agent_id set to parent
3. Send WindowCreated events for both (synthetic SwayEvents)
4. Verify layout commands (SplitHorizontal, MarkWindow, SetLayoutTabbed) — ensures hierarchy is set up
5. **Send Focus IPC message** via socket (simulating CLI `focus --cycle`)
6. Verify Focus handler produces correct FocusWindow command:
   - If child is "focused": next focus should be parent (or cycle back to child if only one)
   - If parent is "focused": next focus should be first child
7. Cleanup (abort handles, remove socket file)

**Expected behavior**:
- LayoutManager produces layout setup commands (same as Wave 3 test)
- Focus handler produces FocusWindow with correct mark for sibling cycling
- Commands arrive in order: layout setup → focus commands

**Reference implementation**: 
- Layout setup from `test_cli_parent_flag_to_layout()` (lines 348–462)
- Socket wiring from `test_daemon_registration_and_status()` (lines 18–104)
- recv_command helper for timeout/receive (already available in test suite)

**Constraints**:
- Do not test LayoutManager behavior (already covered in Wave 3); only focus flow
- Test must run with dry-run mode (no actual Sway needed)
- Focus state can be mocked via test fixtures (no real Sway tree traversal)

**Grep Anchors**: 
- `#[tokio::test]` (test marker)
- `test_cli_focus_with_hierarchy`
- `recv_command` helper

**Acceptance**: `cargo test test_cli_focus_with_hierarchy -- --nocapture`

**Targeted Test**: Full suite: `cargo test` — all integration tests pass, all unit tests pass, no regressions.

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 4.1 | Hierarchical focus algorithm | High (new logic) | ~40 LOC, 20 min |
| 4.2 | WindowCreated handler refactoring | Medium (refactoring) | ~40 LOC, 15 min |
| 4.3 | CLI-daemon socket wiring | Low (additive) | ~20 LOC, 15 min |
| 4.4 | End-to-end integration test | Medium (test design) | ~80 LOC, 25 min |

**Total**: ~180 LOC, ~75 min wall-clock (4 sequential waves). Each task is ≤12 steps, single file per wave (daemon.rs edited in tasks 4.1–4.3, but sequential).
