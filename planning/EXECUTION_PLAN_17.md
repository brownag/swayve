# Execution Plan 17: Bundled Small Goals (G13, G14, README Correction)

## Goal & Scope

**Goal**: Three small, file-disjoint items from `planning/GOALS.md` in one wave:
- **G13**: the daemon-down focus fallback has a test that pins its flat behavior (ADR 0003 known gap).
- **G14**: planning waves 1–14 are archived into `planning/archive/`, and `AGENTS.md` points at the live files.
- **Correction**: the daemon-down note in `README.md` and ADR 0003 say the fallback cycles "every Sway window". The code (`focus_cycle_standalone`, `src/cli.rs`) cycles only windows whose `app_id` starts with `agent.`. The docs must match the code.

**In scope:**
- `src/cli.rs`: extract the flat selection step into a pure function; unit-test it.
- `README.md` line 94 and `docs/decisions/0003-daemon-owns-agent-state.md` line 33: state that the fallback cycles agent windows only.
- `planning/archive/`: move the legacy wave 1–14 files (listed in the Wave Table notes). `AGENTS.md`: update the planning pointers.

**Out of scope:**
- Changing the fallback behavior. The fix is a test and docs, not a change of rules.
- Deleting any file. Archive is `git mv`, so history is kept.
- `PROMPT_TEMPLATE.md` (generic template, kept in place), wave 15–16 files, `planning/GOALS.md`, design docs.
- Starting the daemon from the CLI (G6 decided: keep flat).

**Verbatim constraints**: from `planning/GOALS.md` G13: "The daemon-down focus fallback has a test that pins its flat behavior." From G14: "Planning waves 1–14 are archived out of `planning/` into `planning/archive/`. `AGENTS.md`'s 'Find things' section is updated to match."

## Decision Log

- **D1: Bundle G13, G14, and the README/ADR correction in one wave** (2026-11-07, user asked to bundle small goals). The three tasks touch disjoint files.
- **D2: Archive scope** (2026-11-07, default chosen by orchestrator, user said proceed). Moved: `EXECUTION_PLAN.md`, `EXECUTION_PLAN_WAVE4.md`, `EXECUTION_PLAN_13.md`, `EXECUTION_PLAN_14.md`, `PROMPT_TEMPLATE_13.md`, `PROMPT_TEMPLATE_14.md`, `STATE.md`, `STATE_13.md`, `STATE_14.md`, and every `planning/tasks/*.todo.md` with wave number ≤ 14. Kept in place: `PROMPT_TEMPLATE.md`, all `*_15`/`*_16` files, `tasks/15.*` and `tasks/16.*`, `GOALS.md`, design docs.
- **D3: Test extracts the selection, not the Sway call** (2026-11-07, default). `focus_cycle_standalone` needs a live Sway connection, so the pure choice of next index is what gets tested.
- **D4: Mid-tier worker per task** (2026-11-07, default carried from earlier plans).
- **D5: Acceptance bar**: targeted test per task; `make all` serially at wave end.

## Hazard Notes

- `focus_cycle_standalone` (`src/cli.rs`, around line 410) builds `agents: Vec<(i64, bool)>` from the Sway tree, filtered by `app_id.starts_with("agent.")`, in tree order. The selection is: next index after the focused one, wrapping; first index when none focused; `No active agent windows found` when empty.
- `git mv` for archive so history follows the file. Do not use plain `mv` followed by `git add`.
- `AGENTS.md` line 31 says `EXECUTION_PLAN*.md`; line 33 says `STATE.md`; line 44 says "`STATE.md` before changing code". After archive, the live state file is `planning/STATE_N.md` with the highest N.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **17.1.1** | Extract flat selection in `src/cli.rs`; unit test (G13) | `src/cli.rs` | None |
| **Wave 1** | **17.1.2** | Correct the daemon-down wording in README and ADR 0003 | `README.md`, `docs/decisions/0003-daemon-owns-agent-state.md` | None |
| **Wave 1** | **17.1.3** | Archive waves 1–14; update `AGENTS.md` pointers (G14) | `planning/` (moves), `planning/archive/` (new), `AGENTS.md` | None |

Pairwise disjoint: no two tasks edit the same file.

---

## Task Specifications

### Task 17.1.1: Flat Selection Extracted and Tested

**ID**: `17.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add a private pure function in `src/cli.rs`: `fn next_flat_agent_index(agents: &[(i64, bool)]) -> Option<usize>`.
   - Returns `None` when `agents` is empty.
   - Returns `Some(idx + 1) % len` when one entry has `focused == true` at `idx`.
   - Returns `Some(0)` when none is focused.
2. In `focus_cycle_standalone`, replace the inline `focused_idx`/`next_idx` computation with a call to this function. Keep the empty-list message and the `conn.run_command` behavior unchanged. Handle `None` by printing the same "No active agent windows found to focus." message and returning `Ok(())`.
3. Add a `#[cfg(test)] mod tests` test named `test_flat_fallback_cycles_agent_windows_in_tree_order` covering: empty → `None`; focused middle → next; focused last → wraps to 0; none focused → 0; a single agent focused → 0.
4. Do not change the Sway calls, the `app_id` filter, or `collect_agents_focus`.

**Constraints (do not touch)**: `src/daemon.rs`, `src/sway.rs`, any other file.

**Grep anchors**:
- `async fn focus_cycle_standalone` (`src/cli.rs`, around line 410)
- `fn collect_agents_focus` (nested inside `focus_cycle_standalone`)
- `focused_idx` (`src/cli.rs`, the inline selection to replace)

**Reference implementation**: the `#[cfg(test)]` modules already in `src/config.rs` and `src/layout.rs` for style.

**Acceptance**:
- `cargo test --lib test_flat_fallback_cycles_agent_windows_in_tree_order` passes.
- `cargo check` shows no new warnings.

**Targeted test**: `cargo test --lib test_flat_fallback_cycles_agent_windows_in_tree_order`

---

### Task 17.1.2: Correct Daemon-Down Wording

**ID**: `17.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `README.md`, `docs/decisions/0003-daemon-owns-agent-state.md`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. `README.md` line 94: the comment currently reads "Requires the daemon. Without it, cycling falls back to every Sway window in flat order." Change it to: "Requires the daemon. Without it, cycling falls back to agent windows only, in flat order (hierarchy is lost)."
2. `docs/decisions/0003-daemon-owns-agent-state.md` line 33: change "falls back to a **flat** cycle over the Sway tree" to "falls back to a **flat** cycle over agent windows (`app_id` starting with `agent.`) in the Sway tree". Change nothing else in the ADR; status stays `Accepted`.

**Constraints (do not touch)**: any other line of either file; any other file.

**Grep anchors**:
- `Requires the daemon` (`README.md`, line 94)
- `flat** cycle over the Sway tree` (`docs/decisions/0003-daemon-owns-agent-state.md`, line 33)

**Acceptance**:
- `grep -c "every Sway window" README.md` returns `0`.
- `grep -c 'agent\.' docs/decisions/0003-daemon-owns-agent-state.md` returns at least `1`.
- `grep -n 'status: "Accepted"' docs/decisions/0003-daemon-owns-agent-state.md` returns one line.

**Targeted check**: the three greps above.

---

### Task 17.1.3: Archive Waves 1–14 and Update AGENTS.md

**ID**: `17.1.3`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `AGENTS.md`
- moves (`git mv`) within `planning/`: the files in Decision D2
- can create: `planning/archive/`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. `mkdir -p planning/archive/tasks`.
2. `git mv` each file in D2 into `planning/archive/` (tasks into `planning/archive/tasks/`). Use `git mv` only.
3. Confirm no file in D2 is left in `planning/`, and that `planning/PROMPT_TEMPLATE.md`, `planning/GOALS.md`, `planning/*_15.md`, `planning/*_16.md`, `planning/tasks/15.*`, `planning/tasks/16.*` are still in place.
4. `AGENTS.md`:
   - Line 31 `EXECUTION_PLAN*.md — wave plans`: change to `EXECUTION_PLAN_N.md — wave plans (current: highest N; older waves in planning/archive/)`.
   - Line 33 `STATE.md — execution state`: change to `STATE_N.md — execution state (current: highest N)`.
   - Line 44: change "and `STATE.md` before changing code" to "and the highest-numbered `STATE_N.md` before changing code".
   - Change nothing else in `AGENTS.md`.

**Constraints (do not touch)**: any file in `src/`, `tests/`, `integrations/`, `docs/`, `README.md`; any file not named above; do not delete anything (use `git mv`).

**Grep anchors**: `AGENTS.md` lines 30–44 (the "Find Things" and "Implementation planning" bullets).

**Acceptance**:
- `ls planning/archive | wc -l` ≥ 9 (files moved).
- `git status --short planning/archive` shows only renames (`R`).
- `ls planning/EXECUTION_PLAN.md planning/STATE.md 2>&1 | grep -c 'No such file'` returns `2`.
- `grep -n 'STATE.md' AGENTS.md` returns no line that says the file is current.

**Targeted check**: the four checks above.

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 17.1.1 | Test pins flat fallback (G13) | Low (pure function extraction) | ~30 LOC |
| 17.1.2 | Docs match code | Low | 2 lines |
| 17.1.3 | Archive waves 1–14 (G14) | Low (git renames, reversible) | Moves + 3 lines |

## Wave 1 End Gate

- `make all` run serially (check, test, build).
- Expected: 26 unit tests (25 plus the new one) and 10 integration tests pass. Release build succeeds with no warnings.
- `git status` shows only renames under `planning/` and edits to the files named above.

## Manual Verification (deferred)

- None.
