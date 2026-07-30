# Execution Plan: Deepen the Layout Engine (Minimalist Migration)

## Goal & Scope
Refactor the `swayve` daemon to extract layout orchestration logic from the core event loop (`src/daemon.rs`) into a dedicated `LayoutManager` module. This aims to improve testability, separation of concerns, and maintainability by decoupling visual orchestration from event handling.

**In scope:**
- Extraction of basic layout setup logic (marking windows and moving to workspaces) from `src/daemon.rs`.
- Creation of a new `src/layout.rs` module.
- Integration of `LayoutManager` into the daemon's main event loop.
- Implementation of hierarchical orchestration (splitting/tabbing).

**Out of scope:**
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

## Wave 1: Core Infrastructure & Basic Orchestration (COMPLETED)

## Wave 2: Hierarchical Orchestration (COMPLETED)
