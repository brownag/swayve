---
id: "0007"
title: "Agent Hierarchy via parent_id and Sway Children Marks; Sibling-Cycle Focus"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-07-29
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_WAVE4.md D1, D3, D5 (2026-07-29); commits 43497ae, 1410acc, 08ad0b6, e93f559"
---

# 7. Agent Hierarchy via parent_id and Sway Children Marks; Sibling-Cycle Focus

## Context and Problem Statement
Agents spawn sub-agents. The user needs to move focus among a parent and its children, and the children should sit together in one tabbed container. Flat focus cycling loses the parent/child relation.

## Decision Drivers
* Focus traversal must follow the parent/child tree.
* Children should group visually under the parent.
* The mapping must survive a child window closing.

## Considered Options
1. Flat rotation over all agent windows.
2. Daemon tracks `parent_id`; focus cycles siblings; children grouped by a Sway mark.
3. Put hierarchy in Sway container nesting only, with no daemon record.

## Decision Outcome
Chosen option: "daemon tracks `parent_id`; sibling cycling; a children mark for grouping", because the daemon needs the tree for focus order, and Sway needs a mark to find the container.

Rules:
* `AgentWindow` carries an optional `parent_id` (`src/types.rs`, `43497ae`).
* Focus steps through the hierarchy in pre-order: each root is followed by its children (ordered by `con_id`), then the next root, wrapping at the end. Agents with no window yet are skipped. Children whose parent has no window are appended at the end of the cycle (`next_focus_target`).
* Wave 4 D1 first defined focus as a walk over a parent's children and siblings; Wave 1.5 (task 5.1b in `planning/STATE.md`) changed that sibling rule to pre-order.
* Multi-level focus (grandchildren of a grandparent) is out of scope. Root-level traversal is in Wave 1.5 (`next_focus_target`).
* The children mark sits on the first child's window. When that window closes, the mark moves to the surviving sibling with the lowest con id (`08ad0b6`).

### Consequences
* **Positive**: Focus order follows the tree. Grouping survives child close.
* **Negative**: The mark lives on a real child window, so its lifetime is tied to that window. `08ad0b6` fixed a case where it vanished. Any new path that closes windows must reapply it.
* **Negative**: Depends on [0003](0003-daemon-owns-agent-state.md). The CLI's direct fallback is flat, so hierarchy is lost when the daemon is down.

## Validation & Compliance
Wave 4 added unit tests for sibling traversal and an end-to-end test for the focus flow. Full suite passed at wave end.
