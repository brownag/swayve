# TODO

## Goal
Fix the two defects that agent-window destruction leaves behind — wedged focus cycling and a dangling children-container mark — and make focus cycling reachable from a Sway keybinding.

## Tasks

### 1. Promote orphaned children on parent destruction
- [x] Add private `promote_orphans(state: &mut DaemonState, destroyed: &AgentId)` in `src/daemon.rs`, clearing `parent_id` on every agent that pointed at `destroyed`
- [x] Call it from the `SwayEvent::WindowDestroyed` arm, after the agent is removed from `state.agents`
- [x] Add integration test `test_focus_survives_parent_destruction` in `tests/integration_tests.rs`: register parent + 2 children, give each a window, destroy the parent, then send two `daemon.focus` messages and assert both emit a `FocusWindow` for a surviving child
- [x] Verify: `cargo test -- --test-threads=1`
- [x] Added `test_promote_orphans` unit test — see correction below

### 2. Re-assert the children mark when its carrier dies
- [x] Add `LayoutAction::WindowDestroyed { agent_id, parent_id }` variant in `src/layout.rs`
- [x] Handle it in `compute_commands`: when the destroyed agent had a parent and at least one sibling with a `container_id` survives, emit `MarkWindow { con_id: <lowest-con_id survivor>, mark: parent.to_children_mark() }`
- [x] Add unit test in `src/layout.rs` covering both branches: surviving sibling re-marked; no survivors emits nothing
- [x] Call the layout from the `WindowDestroyed` arm (order relative to promotion is immaterial — see corrected note)
- [x] Verify: `cargo test -- --test-threads=1`

### 3. Bind focus cycling in the documented Sway config
- [x] Add `bindsym $mod+a exec swayve focus --cycle` to the Navigation Bindings block in `README.md` (currently lines 89–94), with a one-line comment
- [x] Verify: `README.md` renders and the binding sits alongside the existing `$mod+Shift+a` and `$mod+Shift+o` entries

### 4. Close out the wave
- [x] Run the full suite serially: `cargo test -- --test-threads=1`
- [x] Confirm `cargo check` is warning-free
- [x] One conventional commit per task group (1, 2, 3)
- [x] Record Wave 5 board and reports in `planning/STATE.md`

## Discovered Tasks

### 1b. Advance between root-level agents in `next_focus_target`
- [x] In `src/daemon.rs`, change the root branch of `next_focus_target`: when a root agent has no children, advance to the next root by `con_id` (wrapping) instead of returning `None`
- [x] Keep the existing descent: a root **with** children still enters its first child
- [x] Add unit cases to `test_hierarchical_focus`: two unrelated roots cycle into each other; a lone root still returns `None`
- [x] Verify: `cargo test -- --test-threads=1`

**Why this is required.** Task 1 as written cannot pass its own test. Probing `next_focus_target` directly:

```
two promoted roots:  from None -> Some(child_a),  from child_a -> None
flat, no hierarchy:  from solo_1 -> None
```

Promotion moves the children to roots, but the root branch is `children_of(current).first()` — a root with no children yields `None`, so focus stops after one press. The orphan fix alone converts "focus returns a dead id" into "focus returns nothing"; the user-visible symptom, cycling stops working, is unchanged.

The second line is the more serious finding and is not about orphans at all: **several unrelated agents with no parent/child links cannot cycle at all.** That is the ordinary case, and Wave 4 shipped it broken. My Wave 4 integration test only ever built one parent with two children, so it passed while the common path was dead.

## Notes

**CORRECTED — tasks 1 + 2 do not collide.** I claimed task 1 clears the `parent_id` that task 2 needs. That was wrong. Task 2 reads the *destroyed agent's own* `parent_id`; `promote_orphans` clears the `parent_id` of the destroyed agent's *children*. Those are different records — a leaf child being destroyed has no children to promote, and a parent being destroyed is not what task 2 keys off. Either order works.

**CORRECTED — task 1's integration test does not isolate task 1.** `test_focus_survives_parent_destruction` passes with `promote_orphans` commented out, because 1b's dangling-children fallback already re-admits children of a departed parent into the cycle. The integration test is still worth keeping — it pins the user-visible guarantee end to end — but `promote_orphans` is justified on different grounds: it stops the layout path from reasoning about a dead parent (see task 2), and it is covered directly by the `test_promote_orphans` unit test instead.

**Why task 2 is a real defect, not speculative.** `compute_commands` applies the children mark to the *first* child's `con_id` (`src/layout.rs:61-64`). Marks live on containers, so when that first child's window dies the mark dies with it. A later sibling then takes the `has_siblings` branch and emits `MoveToContainer { target_mark: mark_children_<parent> }` against a mark that no longer exists — the move silently fails and the window lands in the wrong place. Re-asserting the mark on a survivor keeps the target alive.

**Decisions from the question round (2026-07-30).** Orphaned children are promoted to roots rather than re-parented to a grandparent. The daemon keeps its loop-local `last_focused` cursor rather than querying the Sway tree, so focusing by mouse or a plain sway binding still desyncs the cursor until the next cycle — accepted, unchanged from Wave 4.

**Dropped from the Wave 4 plan.** That plan listed "refactoring the overstuffed UpdateStatus handler" for Wave 5. On inspection (`src/daemon.rs:158-180`) it is ~22 lines doing one coherent job — status to border colour, plus focus on `AwaitingHumanInput`. It is not overstuffed and no longer appears here.

**Needs live-Sway confirmation.** Task 2 assumes `mark --add` moves a mark when applied to a different container. The unit test pins the emitted command, not Sway's reaction to it; confirm on a real compositor before relying on the fix.

**Not in scope.** Multi-level nesting focus (grandchildren), dynamic parent re-assignment, and reading focus state from the Sway tree.
