# Swayve Project State

## Baseline Checklist
- [x] `src/daemon.rs` exists
- [x] `src/sway.rs` exists
- [x] `src/types.rs` exists
- [x] `src/layout.rs` exists (Wave 2)
- [x] `src/config.rs` exists
- [x] Tests exist and pass (Wave 1 & 2)

## Wave Board

### Wave 1: Core Infrastructure & Basic Orchestration
- [x] **Task 1.1**: Define Layout Domain & Module
- [x] **Task 1.2**: Refactor Daemon Loop (Basic Setup)
- [x] **Task 1.3**: Add Unit Tests for `LayoutManager`

### Wave 2: Hierarchical Orchestration
- [x] **Task 2.1**: Implement Hierarchical Layout Logic
- [x] **Task 2.2**: Integrate Hierarchical Logic into Daemon

### Wave 3: Architectural Polish & Agent Coordination
- [ ] **Task 3.1**: Extract AgentRegistry Trait; break daemon ↔ layout cycle
- [ ] **Task 3.2**: Add Process-Tree Auditing Configuration
- [ ] **Task 3.3**: Verify End-to-End Parent Agent Support & CLI Integration

## Task Reports

### Wave 1 & 2 (Completed)
- **Task 1.1-1.3 + 2.1-2.2**: Implemented `LayoutManager` module with basic and hierarchical orchestration. ✅
- **Standards Cleanups**: Applied three cleanup recommendations (mark extraction, test helper, depth constant). ✅

### Wave 3 (Pending)
- **Task 3.1**: Pending — Extract AgentRegistry trait, impl on DaemonState, update layout.rs imports
- **Task 3.2**: Pending — Add daemon config section with max_ancestry_depth and process_tree_audit_enabled
- **Task 3.3**: Pending — Test end-to-end parent agent flow; wire CLI focus into daemon
