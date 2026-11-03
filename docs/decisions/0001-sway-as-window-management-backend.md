---
id: "0001"
title: "Use Sway as the Window-Management Backend"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-26
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Initial commit 6c3585c; README.md 'The Tiling Compositor Solution'; Sway Multi-Agent Viewer Architecture.md"
---

# 1. Use Sway as the Window-Management Backend

## Context and Problem Statement
Agent sessions need one terminal window each, visible state (focus, attention needed), and keyboard navigation. Nested multiplexers (`tmux`, `zellij`) share one server process and clash with editor shortcuts. Electron grid viewers are heavy. Terminal borders cannot signal state in either.

## Decision Drivers
* Each agent must run in an OS-isolated process with its own window.
* Ambient state signals (border colour) must be drawn by something outside the terminal.
* Keyboard focus navigation must be global, not per-application.

## Considered Options
1. Nested terminal multiplexer (tmux/zellij).
2. Electron or web grid viewer.
3. Sway (i3-compatible Wayland compositor) driven over its IPC socket.

## Decision Outcome
Chosen option: "Sway driven over its IPC socket", because it provides per-window isolation, border styling, `con_mark` addressing, and global bindsyms without a custom renderer.

### Consequences
* **Positive**: No custom rendering. Hierarchy and focus reuse Sway's container tree and marks (see [0007](0007-agent-hierarchy-and-sibling-focus.md)).
* **Negative**: Requires a running Sway session with `SWAYSOCK` set. Live mode cannot run elsewhere. Sway criteria syntax is a dependency that has broken before (commit `8c7d9fb`).

## Validation & Compliance
`src/sway.rs` builds all compositor actions as `SwayCommand` variants (`MarkWindow`, `SetBorderColor`, `FocusWindow`, `SplitHorizontal`, `MoveToContainer`, `SetLayoutTabbed`) and renders them as `[con_mark=...]` / `[con_id=...]` criteria. Dry-run mode skips the connection.
