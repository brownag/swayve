# Execution Plan: Deepen the Layout Engine (Minimalist Migration)

## Goal & Scope
Refactor the `swayve` daemon to extract layout orchestration logic from the core event loop (`src/daemon.rs`) into a dedicated `LayoutManager` module. This aims to improve testability, separation of concerns, and maintainability by decoupling visual orchestration from event handling.

**In scope:**
- Extraction of basic layout setup logic (marking windows and moving to workspaces) from `src/daemon.rs`.
- Creation of a new `src/layout.rs` module.
- Integration of `LayoutManager` into the daemon's main event loop.

**Out of scope:**
- Hierarchical/tree-based layout logic (splitting/tabbing/child-grouping).
- Refactoring of the `AgentRegistry`.
- Changing `swayipc-async` or `SwayManager`.

## Decision Log
- **D1: Interface**: Message-Passing. `LayoutManager` returns `Vec<SwayCommand>` given a `LayoutAction` and current state.
- **D2: Migration Scope**: Minimalist. Move only "Basic Setup" (marking/workspace routing) first.
- **D3: API Granularity**: Action Mapping. `Daemon` translates `SwayEvent` to `LayoutAction`.
- **D4: Execution Mode**: Isolated Worktrees.
- **D5: Acceptance Bar**: Compilation + Targeted Test.

## Hazard Notes
- Touching `src/daemon.rs` is high risk as it's the heart of the daemon.
- Mismatched `LayoutAction` mapping could break window management.

## Wave 1: Core Infrastructure & Basic Orchestration

### Task 1.1: Define Layout Domain & Module
- **Files-in-scope**: `src/layout.rs`, `src/lib.rs`
- **Depends-on**: None
- **Spec**: Create `src/layout.rs`. Define `LayoutAction` enum (e.g., `WindowCreated { con_id: i64, app_id: String, project_id: ProjectId }`) and `LayoutManager` struct. Implement `LayoutManager::compute_commands(&self, action: LayoutAction, state: &DaemonState) -> Vec<SwayCommand>`. Update `src/lib.rs` to include `pub mod layout`.
- **Constraints**: Do not implement hierarchical logic yet.
- **Grep Anchors**: `pub mod layout`, `pub struct LayoutManager`, `pub enum LayoutAction`.
- **Acceptance**: `cargo check`.
- **Targeted Test**: `cargo test --lib layout`.

### Task 1.2: Refactor Daemon Loop (Basic Setup)
- **Files-in-scope**: `src/daemon.rs`
- **Depends-on**: Task 1.1
- **Spec**: In `run_daemon_loops`, inside the `SwayEvent::WindowCreated` match arm, translate the event to a `LayoutAction` and call `LayoutManager::compute_commands`. Replace the hardcoded "Mark Window" and "Move to Workspace" logic with the result of this call.
- **Constraints**: Do not touch hierarchical split/tab logic.
- **Grep Anchors**: `LayoutManager`, `LayoutAction`, `SwayEvent::WindowCreated`.
- **Acceptance**: `cargo check`.
- **Targeted Test**: `cargo check`.

### Task 1.3: Add Unit Tests for LayoutManager
- **Files-in-scope**: `src/layout.rs`
- **Depends-on**: Task 1.1
- **Spec**: Add a `#[cfg(test)]` module in `src/layout.rs`. Test `compute_commands` for the `WindowCreated` action to ensure it produces the expected `MarkWindow` and `MoveToWorkspace` commands.
- **Constraints**: Use minimal mocks for `DaemonState` and `SwayCommand`.
- **Grep Anchors**: `#[cfg(test)]`, `test_window_created`.
- **Acceptance**: `cargo test --lib layout`.
