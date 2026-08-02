# Swayve Project State

## Baseline Checklist
- [x] `src/daemon.rs` exists
- [x] `src/sway.rs` exists
- [x] `src/types.rs` exists
- [x] `src/layout.rs` exists
- [x] `src/config.rs` exists
- [x] Tests exist and pass

## Wave Board

### Wave 1: Core Infrastructure & Basic Orchestration
- [x] **Task 1.1**: Define Layout Domain & Module
- [x] **Task 1.2**: Refactor Daemon Loop (Basic Setup)
- [x] **Task 1.3**: Add Unit Tests for `LayoutManager`

### Wave 2: Hierarchical Orchestration
- [x] **Task 2.1**: Implement Hierarchical Layout Logic
- [x] **Task 2.2**: Integrate Hierarchical Logic into Daemon

### Wave 3: Architectural Polish & Agent Coordination
- [x] **Task 3.1**: Extract AgentRegistry Trait; break daemon ↔ layout cycle
- [x] **Task 3.2**: Add Process-Tree Auditing Configuration
- [x] **Task 3.3**: Verify End-to-End Parent Agent Support & CLI Integration

### Wave 4: Hierarchical Focus & CLI-Daemon Coordination
- [x] **Task 4.1**: Implement Hierarchical Focus Algorithm
- [x] **Task 4.2**: Refactor WindowCreated Handler into Private Sub-Helpers
- [x] **Task 4.3**: Wire CLI Focus Command to Daemon via Socket IPC
- [x] **Task 4.4**: Integration Test for CLI→Daemon→Sway Focus Flow

### Wave 5: Destruction Handling & Focus Reachability
- [x] **Task 5.1**: Promote orphaned children on parent destruction
- [x] **Task 5.1b**: Advance between root-level agents in `next_focus_target`
- [x] **Task 5.2**: Re-assert the children mark when its carrier dies
- [x] **Task 5.3**: Bind focus cycling in the documented Sway config

### Wave 6: Border Fixing, Unique IDs, and Environment Injection
- [x] **Task 6.1**: Fix Sway border formatting command in `src/sway.rs`
- [x] **Task 6.2**: Plumb unique `app_id` and environment variables in `src/cli.rs`
- [x] **Task 6.3**: Update and verify integration tests

### Wave 7: Agent Integrations and Shell Abstractions
- [x] **Task 7.1**: Add `init-shell` CLI subcommand
- [x] **Task 7.2**: Build the `integrations/` directory structures

### Wave 8: Claude Code Lifecycle Hook Integration
- [x] **Task 8.1**: Create Claude hook templates and installer

### Wave 9: Pi and Claude Integration Testing Scripts
- [x] **Task 9.1**: Configure and verify Pi integration tests
- [x] **Task 9.2**: Build and verify Claude `test-hooks.sh` integration

### Wave 10: Claude Code Hooks Schema Fix
- [x] **Task 10.1**: Restructure Claude Code hooks to match the correct schema

### Wave 11: Hook System, Log Redirection, and Auto-Monitoring
- [x] **Task 11.1**: Add hook config, log redirection, and hook execution
- [x] **Task 11.2**: Auto-spawn log-tail monitoring windows for subagents
- [ ] **Task 11.3**: Integration tests for hooks and auto-monitoring

## Task Reports

### Waves 1-10 (Completed)
- All tasks in Waves 1 to 10 are fully completed, verified, and integrated.

### Wave 11 Reports
### 11.1
- **Status**: Completed
- **Files changed**:
  - `src/config.rs` (L11-19, L76, L93-139): Added `HooksConfig` struct, `hooks` config field, defaults, and unit tests.
  - `src/cli.rs` (L239-247): Created `/tmp/swayve/logs` directory and redirected spawned terminal `stdout`/`stderr` to `/tmp/swayve/logs/{agent_id}.log`.
  - `src/daemon.rs` (L42-45, L48-52, L64-88, L585-616): Added `execute_lifecycle_hook`, status conversion helper, hook triggers in `register_agent`/`update_status`, and unit tests.
- **Verification**: `cargo check` passed; `cargo test` passed (22/22 tests passed: 15 unit tests, 7 integration tests).
- **Deviations**: None
- **Gotchas for dependent tasks**: Log redirection creates files in `/tmp/swayve/logs/<agent_id>.log` which Task 11.2 can tail directly.

### 11.2
- **Status**: Completed
- **Files changed**:
  - `src/config.rs` (L40-60, L150-168): Added `auto_monitor: bool` field to `DaemonConfig` and config parsing unit test.
  - `src/daemon.rs` (L30-80, L265-310, L690-720): Added `spawn_monitor_terminal`, checked `auto_monitor` upon subagent registration & process audit, routed `agent.monitor.` windows to layout, and added unit tests.
  - `src/layout.rs` (L35-55, L385-425): Updated `LayoutManager` to group `agent.monitor.` windows side-by-side with target subagent windows, and added unit test.
- **Verification**: `cargo test` passed (25/25 tests passed: 18 unit tests, 7 integration tests).
- **Deviations**: None
- **Gotchas for dependent tasks**: Task 11.3 can test auto-monitoring by enabling `auto_monitor = true` in config and verifying monitor window layout.
