# Swayve Project State

## Baseline Checklist
- [x] `src/daemon.rs` exists
- [x] `src/sway.rs` exists
- [x] `src/types.rs` exists

## Wave Board

### Wave 1: Core Infrastructure & Basic Orchestration
- [x] **Task 1.1**: Define Layout Domain & Module (`src/layout.rs`)
- [x] **Task 1.2**: Refactor Daemon Loop (Basic Setup) (`src/daemon.rs`)
- [x] **Task 1.3**: Add Unit Tests for `LayoutManager` (`src/layout.rs`)

### Wave 2: Hierarchical Orchestration
- [ ] **Task 2.1**: Implement Hierarchical Layout Logic in `LayoutManager`
- [ ] **Task 2.2**: Integrate Hierarchical Logic into `DaemonState`

## Task Reports
- **Task 1.1**: Implemented `LayoutAction`, `LayoutManager`, and unit tests.
- **Task 1.2**: Refactored `run_daemon_loops` to delegate basic setup to `LayoutManager`.
- **Task 1.3**: Verified unit tests in `src/layout.rs`.
