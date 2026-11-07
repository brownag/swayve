# Execution Plan 14: Record Focus-State Decision and Correct Focus-Order ADR (Goal G2)

## Goal & Scope

**Goal**: Record the focus-state decision made in Orchestration 13 as ADR 0011, and correct ADR 0007, whose focus-order text is stale. ADR 0007 still describes children-only sibling cycling, but `next_focus_target` (`src/daemon.rs`) now steps in pre-order over roots and their children, wrapping at the end.

**In scope:**
- New ADR `docs/decisions/0011-focus-state-from-sway-window-events.md`: `last_focused` is the source of truth for focus cycling, fed by Sway window-focus events for agent windows.
- Amend ADR 0007 in place: fix the focus-order text and note that Wave 1.5 changed it (D1).
- Add the 0011 row to the table in `docs/decisions/README.md`.

**Out of scope:**
- Any code change. The behavior is already implemented in Orchestration 13.
- A new superseding ADR for 0007 (D1: amend in place).
- Daemon-down fallback (ADR 0003 negative; G6).
- Live-Sway verification (deferred from Orchestration 13).

## Decision Log

- **D1: Amend ADR 0007 in place** (2026-11-02). The correction clarifies a rule changed in Wave 1.5; no decision is reversed, so no superseding record is needed.
- **D2: One task** (2026-11-02). One worker edits 0007, writes 0011, and updates the README. The work is docs-only and small.
- **D3: Mid-tier worker** (2026-11-02, recommended default, not separately confirmed). Matches Orchestration 13.
- **D4: Acceptance** (2026-11-02, recommended default). Mechanical grep checks plus a link check on the README table. No cargo gate, since no code changes. `make check` runs as a sanity check at the end.
- **D5: 0011 status Accepted** (2026-11-02, recommended default). The behavior is shipped in commit `436b190`.

## Hazard Notes

- **ADR 0007 "Validation & Compliance" section** cites Wave 4 tests for sibling traversal. Keep that citation; only the rule text changes.
- **ADR 0003 "Consequences"** says the CLI fallback is flat and loses the hierarchy. That stays true and is not part of this task.
- **README table order**: the README says ADRs are ordered by dependency. 0011 depends on 0003 and 0007, so the row goes last.
- **Commit `436b190`** is the implementation of the decision in 0011. Cite it in `source_context`, as other ADRs do.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **14.1.1** | Amend ADR 0007; write ADR 0011; add README row | `docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`, `docs/decisions/0011-focus-state-from-sway-window-events.md` (create), `docs/decisions/README.md` | None |

---

## Task Specifications

### Task 14.1.1: Amend ADR 0007, Write ADR 0011, Update README

**ID**: `14.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`, `docs/decisions/README.md`
- can create: `docs/decisions/0011-focus-state-from-sway-window-events.md`
- scratch: `.development/` only
**Depends-on**: None

**Spec prose**:

1. **Amend ADR 0007** (`docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`):
   - Replace the "Rules" bullet that says "Focus on a parent cycles its children. Focus on a child cycles its siblings, then the parent (Wave 4 D1)." with the current rule: focus steps through the hierarchy in pre-order. Each root is followed by its children (ordered by `con_id`), then the next root, wrapping at the end. Agents with no window yet are skipped. Children whose parent has no window are appended at the end of the cycle.
   - Add one sentence noting that Wave 1.5 changed the Wave 4 sibling rule to pre-order (cite "Wave 1.5, task 5.1b" as named in planning/STATE.md). Keep the Wave 4 D1 reference as history.
   - Update the `Decision Outcome` sentence that says "Focus on a parent cycles children" only if it contradicts the rule above.
   - Do not change the `Validation & Compliance` section, the `source_context` line, or the `status`.
   - Anchors: `Focus on a parent cycles its children` (the Rules section), `Multi-level focus` bullet (keep: multi-level traversal is still out of scope), `fn next_focus_target` in `src/daemon.rs` (reference only, do not edit).

2. **Create ADR 0011** (`docs/decisions/0011-focus-state-from-sway-window-events.md`), following the frontmatter and section layout of ADR 0007:
   - Frontmatter: `id: "0011"`, `title: "Focus Cycle State Is Fed by Sway Window-Focus Events"`, `status: "Accepted"`, `date: 2026-11-02`, `authors: ["Andrew G. Brown"]`, `supersedes: null`, `superseded_by: null`, `source_context: "EXECUTION_PLAN_13.md D1; Orchestration 13 tasks 13.1.1 and 13.1.2; commit 436b190; src/daemon.rs last_focused, next_focus_target; src/sway.rs SwayEvent::WindowFocused"`.
   - Context and Problem Statement: `focus --cycle` continues from the last agent the daemon focused. Focus changes made outside the daemon (mouse click, Sway binding) were not seen, so the next cycle started from a stale window.
   - Decision Drivers: cycling must resume from the window the user is on; no per-cycle Sway query; no new daemon fields unless needed.
   - Considered Options: (1) query Sway's tree on each `Focus` event, as the CLI fallback does; (2) a last-focused tracker updated from Sway focus events (chosen); (3) leave `last_focused` as only the daemon's own last focus (rejected, it is the bug).
   - Decision Outcome: `last_focused` is the cycle's source of truth. The daemon's `run_daemon_loops` sets it when a Sway window-focus event names a registered agent (`SwayEvent::WindowFocused`, mapped through `con_to_agent`). Focus on a non-agent window is ignored, so cycling resumes from the last agent (Orchestration 13 D1). When the focused agent is destroyed, `next_focus_target` falls back to the top of the cycle.
   - Consequences: Positive: no Sway round-trip per cycle; manual focus is respected. Negative: `last_focused` is loop-local in `run_daemon_loops`, so only that loop's arms may write it; the window-focus subscription must stay active (it shares the `EventType::Window` stream). Negative: a daemon-down CLI fallback still cycles flat (see ADR 0003). Daemon-initiated focus also produces a focus event; it names the same agent, so it is harmless.
   - Validation & Compliance: integration test `test_focus_follows_sway_focus` in `tests/integration_tests.rs`; unit tests `test_hierarchical_focus` and `test_focus_cycles_between_roots` in `src/daemon.rs`.
   - Length target: 40 to 60 lines, matching ADR 0007.

3. **Update the README table** (`docs/decisions/README.md`): add one row after the 0010 row: `| [0011](0011-focus-state-from-sway-window-events.md) | Focus Cycle State Is Fed by Sway Window-Focus Events | Accepted | 2026-11-02 | - | - | Orchestration 13 |`. Do not change other rows.

**Constraints (do not touch)**:
- No edits to any file under `src/` or `tests/`.
- Do not change ADR 0003 or any other ADR.
- Do not change the README's introductory text or "Not recorded as ADRs" section.
- Do not add an ADR for the CLI fallback or the mark behavior (out of scope).

**Grep anchors**:
- `Focus on a parent cycles its children` (`docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`)
- `| [0010]` (`docs/decisions/README.md`, insert after this row)
- `fn next_focus_target` and `last_focused` (`src/daemon.rs`, reference only)

**Reference implementation**: `docs/decisions/0007-agent-hierarchy-and-sibling-focus.md` for the 0011 layout; `docs/decisions/0010-toml-config-in-home-config-dir.md` for a short Accepted record.

**Acceptance** (mechanical, all must pass):
- `test -f docs/decisions/0011-focus-state-from-sway-window-events.md`
- `grep -q '^id: "0011"' docs/decisions/0011-focus-state-from-sway-window-events.md`
- `grep -q '0011-focus-state-from-sway-window-events.md' docs/decisions/README.md`
- `! grep -q 'cycles its siblings, then the parent' docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`
- `grep -q 'pre-order' docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`
- `grep -q 'Wave 1.5' docs/decisions/0007-agent-hierarchy-and-sibling-focus.md`
- `cargo check` passes (sanity; no code changed)

**Targeted test**: the acceptance commands above. No cargo test needed.

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 14.1.1 | Record focus-state decision; correct focus-order text | Low (docs only) | ~80 lines of Markdown |

## Manual Verification

None new. The live-Sway check from Orchestration 13 is still outstanding.
