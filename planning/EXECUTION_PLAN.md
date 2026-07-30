# Execution Plan: Wave 3 — Architectural Polish & Agent Coordination

## Goal & Scope

**Goal**: Refactor the swayve daemon to eliminate the circular dependency between `daemon.rs` and `layout.rs` by introducing an `AgentRegistry` trait. Add process-tree auditing configuration to `config.toml`. Verify end-to-end parent agent support (CLI → daemon → LayoutManager) via integration tests.

**In scope:**
- Extract `AgentRegistry` trait; impl on DaemonState to break daemon ↔ layout cycle
- Add `max_ancestry_depth` and `process_tree_audit_enabled` configuration knobs to config.toml
- Add integration test for parent agent support end-to-end flow (CLI --parent flag through hierarchy logic)
- Update imports: LayoutManager depends on trait, not concrete DaemonState

**Out of scope:**
- Hierarchical focus logic (focus next sibling, etc.) — deferred to Wave 4
- Dynamic parent re-assignment after agent creation — future enhancement
- Refactoring AgentRegistry; changing SwayManager or swayipc-async

## Decision Log

- **D1: Wave Sequencing**: Two-wave plan collapsed into one big wave (Wave 3) with three sequential tasks. Trait + config + CLI in one session avoids daemon.rs re-entry.
- **D2a: AgentRegistry Interface**: Minimal trait (single `agents()` method returning HashMap ref). LayoutManager does its own lookups; trait boundary stays clean.
- **D2b: Implementation**: Direct impl on DaemonState. No wrapper types.
- **D3: Config Approach**: Process-tree settings in config.toml (`[daemon]` section: `max_ancestry_depth`, `process_tree_audit_enabled`). Loaded by config.rs, passed to daemon via Config struct.
- **D4: Wave Structure**: One wave, three tasks, sequential execution. Trait isolation first, then config plumbing, then CLI test. daemon.rs edited three times (careful coordination).
- **D5: Acceptance Bar**: Hybrid. Tasks 1.1 & 1.2 gate on `cargo check`. Task 1.3 gates on `cargo test`. Full suite at wave end.
- **D6: Focus Cycling Scope**: Keep CLI focus command parallel (not routed through LayoutManager yet). Wire into daemon event handler for consistency. Hierarchical focus is Wave 4.
- **D7: Parent Support**: Already integrated (Wave 2). Wave 3 Task 1.3 verifies end-to-end via test; no new features.

## Hazard Notes

- **daemon.rs edited in all three tasks**: Trait impl, config plumbing, CLI wiring. Risk of conflicts if tasks overlap. Mitigation: explicit grep anchors per task, staged edits.
- **Circular import during refactor**: Brief moment during Task 1.1 where layout.rs still imports DaemonState but starts using AgentRegistry. Must update both in one commit.
- **Config parsing**: config.toml defaults must match const MAX_ANCESTRY_DEPTH (32). If changed later, both must stay in sync.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 3** | **3.1** | Extract AgentRegistry trait; impl on DaemonState | src/types.rs, src/daemon.rs, src/lib.rs, src/layout.rs | None |
| | **3.2** | Add process-tree config to config.rs; plumb through daemon | src/config.rs, src/daemon.rs | Task 3.1 |
| | **3.3** | Test end-to-end parent agent support; CLI integration | tests/integration_tests.rs, src/cli.rs, src/daemon.rs | Task 3.2 |

---

## Task Specifications

### Task 3.1: Extract AgentRegistry Trait

**ID**: `3.1`  
**Repo**: `swayve`  
**Files-in-scope**: `src/types.rs`, `src/daemon.rs`, `src/lib.rs`, `src/layout.rs`

**Depends-on**: None

**Spec prose**:
Define a minimal `AgentRegistry` trait in `src/types.rs`:
```rust
pub trait AgentRegistry {
    fn agents(&self) -> &HashMap<AgentId, AgentWindow>;
}
```

Implement the trait on `DaemonState` in `src/daemon.rs`:
```rust
impl AgentRegistry for DaemonState {
    fn agents(&self) -> &HashMap<AgentId, AgentWindow> {
        &self.agents
    }
}
```

Update `src/layout.rs` to import `AgentRegistry` instead of `DaemonState`:
- Change `use crate::daemon::DaemonState;` to `use crate::types::AgentRegistry;`
- Update `compute_commands` signature: `state: &dyn AgentRegistry` instead of `state: &DaemonState`
- All internal lookups (`.agents()` calls) remain unchanged

Export trait from `src/lib.rs`: add `pub use crate::types::AgentRegistry;`

**Constraints**: 
- Do not change LayoutManager logic or behavior; this is a pure refactor of imports
- Do not touch any other modules
- Ensure the import cycle is broken: layout.rs must only import from types.rs and sway.rs, never daemon.rs

**Grep Anchors**: 
- `pub trait AgentRegistry`
- `impl AgentRegistry for DaemonState`
- `use crate::types::AgentRegistry;` (in layout.rs)

**Acceptance**: `cargo check`

**Targeted Test**: `cargo test --lib layout` — unit tests should still pass with trait-based input.

---

### Task 3.2: Add Process-Tree Auditing Configuration

**ID**: `3.2`  
**Repo**: `swayve`  
**Files-in-scope**: `src/config.rs`, `src/daemon.rs`

**Depends-on**: Task 3.1

**Spec prose**:
Extend `Config` struct in `src/config.rs` to add two new fields under a `[daemon]` section:
```rust
pub struct Config {
    // ... existing fields ...
    pub daemon: DaemonConfig,
}

pub struct DaemonConfig {
    pub max_ancestry_depth: usize,        // default: 32
    pub process_tree_audit_enabled: bool, // default: true
}
```

Update the TOML parsing in `load_config()` to deserialize these fields, with sensible defaults if the `[daemon]` section is missing.

In `src/daemon.rs`, replace the hardcoded `const MAX_ANCESTRY_DEPTH = 32` with a dynamic value read from `config.daemon.max_ancestry_depth`. Use this in `find_registered_parent_ancestor()`:
- Change `while current_pid > 1 && depth < MAX_ANCESTRY_DEPTH` to `while current_pid > 1 && depth < config.daemon.max_ancestry_depth`
- Pass `config` to the function or access it from state/context as appropriate

Add a guard: if `config.daemon.process_tree_audit_enabled` is false, skip the auto-detect parent logic entirely (lines ~187–195).

**Constraints**:
- Do not change the logic of process-tree scanning; only make the depth limit and enable/disable flag configurable
- Maintain backward compatibility: if config.toml has no `[daemon]` section, use defaults

**Grep Anchors**: 
- `struct DaemonConfig`
- `max_ancestry_depth`
- `process_tree_audit_enabled`

**Acceptance**: `cargo check`

**Targeted Test**: Unit test (mock config with depth=5, verify scan stops early) in `src/daemon.rs` or new test module.

---

### Task 3.3: Verify End-to-End Parent Agent Support & CLI Integration

**ID**: `3.3`  
**Repo**: `swayve`  
**Files-in-scope**: `tests/integration_tests.rs`, `src/cli.rs`, `src/daemon.rs`

**Depends-on**: Task 3.2

**Spec prose**:
Add a new integration test `test_cli_parent_flag_to_layout` that:
1. Spawns daemon with dry-run mode
2. Uses CLI `spawn --agent parent-agent --parent <id>` (or similar existing command) to register parent
3. Uses CLI `spawn --agent child-agent --parent parent-agent` to register child with parent
4. Sends synthetic window-created events for both
5. Verifies LayoutManager produces correct hierarchy commands (split, tab, move-to-container)

This end-to-end test proves the flow: CLI --parent flag → RegisterParams.parent_agent_id → daemon.register_agent() → LayoutAction.parent_id → LayoutManager hierarchy logic → SwayCommands.

Also, wire the CLI `focus --cycle` command into the daemon event loop (currently not wired):
- Add a `Focus` variant to `DaemonEvent` enum (src/daemon.rs line 66–70)
- In `run_daemon_loops`, add a match arm for `DaemonEvent::Focus` (after SwayEvent handler, line ~226)
- Focus handler collects agent windows from state (like CLI does) and rotates focus; does NOT call LayoutManager (per D6)
- For now, focus is independent; hierarchical focus (focus next sibling) is Wave 4

**Constraints**:
- Do not add hierarchical focus logic (focus next sibling) — that's Wave 4
- Do not change LayoutManager; only test and wire existing behavior
- Focus command remains independent of layout orchestration for now

**Grep Anchors**: 
- `test_cli_parent_flag_to_layout`
- `--parent` (CLI parsing in cli.rs)
- `focus --cycle` (if wiring is needed)

**Acceptance**: `cargo test` (full suite)

**Targeted Test**: `cargo test test_cli_parent_flag_to_layout` (new test passes; existing tests still pass)

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 3.1 | Break circular import via trait | Medium (refactoring) | ~20 LOC, 15 min |
| 3.2 | Config plumbing for audit settings | Low (additive) | ~30 LOC, 15 min |
| 3.3 | End-to-end test + CLI wiring | Medium (integration) | ~50 LOC, 20 min |

**Total**: ~100 LOC, ~50 min wall-clock (sequential). Tests validate all three integrate correctly.
