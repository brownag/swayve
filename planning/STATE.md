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

## Task Reports

### Waves 1-6 (Completed)
- All tasks in Waves 1 to 6 are fully implemented, verified, and integrated. All 18 tests pass successfully.

### Wave 7 Reports

#### Task 7.1: Add init-shell CLI Subcommand
- **Status**: Completed
- **Files changed**: `src/cli.rs` (L27-L28, L77-L128, L432-L442)
- **Tests**: `cargo test` (19 passed, 0 failed: 12 unit tests, 7 integration tests)
- **Manual Verification**: `cargo run -- init-shell` successfully outputs POSIX-compliant prompt hook snippet for bash and zsh
- **Deviations**: None
- **Gotchas for dependent tasks**: Shell scripts requiring prompt integration can source output of `swayve init-shell` directly in subshell / agent terminals.

#### Task 7.2: Build the integrations/ Directory Structure
- **Status**: Completed
- **Files created**: `integrations/pi/README.md` (L1-L157), `integrations/claude/claude-wrapper.sh` (L1-L39), `integrations/antigravity/README.md` (L1-L107)
- **Tests**: `bash -n integrations/claude/claude-wrapper.sh` (0 errors), `./integrations/claude/claude-wrapper.sh --version` (pass), `cargo test` (19 passed, 0 failed)
- **Deviations**: None
- **Gotchas for dependent tasks**: `claude-wrapper.sh` requires `chmod +x` (applied) and routes status updates through `$SWAYVE_SOCKET` using `nc -U` or `socat`.


