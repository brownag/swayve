# Orchestration 17 State: Bundled Small Goals (G13, G14, Daemon-Down Wording Correction)

Plan: `planning/EXECUTION_PLAN_17.md`
Last Updated: 2026-11-07

## Baseline (before Wave 1)
- Orchestration 16 complete. Commits `66ef55c` (parent-ID fix) and `03a57c8` (G12 record) on top of `398fa92`.
- `make all` (last recorded at Orchestration 16): 25 unit passed, 10 integration passed, release build OK, no warnings.
- Working tree clean at plan time.

## Wave Board

### Wave 1: Flat Fallback Test, Wording Correction, Planning Archive
- [x] **Task 17.1.1**: Extract `next_flat_agent_index` in `src/cli.rs`; add `test_flat_fallback_cycles_agent_windows_in_tree_order` (G13)
- [x] **Task 17.1.2**: Correct daemon-down wording in `README.md` line 94 and `docs/decisions/0003-daemon-owns-agent-state.md` line 33
- [x] **Task 17.1.3**: Archive waves 1–14 into `planning/archive/` with `git mv`; update `AGENTS.md` pointers (G14)

### Wave 1 end gate
- [x] `make all` passes serially (check, test, build); 26 unit + 10 integration passed, release build OK, no warnings

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

## 17.1.2
- Edited `README.md` line 94: now "falls back to agent windows only, in flat order (hierarchy is lost)."
- Edited `docs/decisions/0003-daemon-owns-agent-state.md` line 33: now "flat cycle over agent windows (`app_id` starting with `agent.`) in the Sway tree".
- Verified `focus_cycle_standalone` (`src/cli.rs`) filters on `app_id.starts_with("agent.")`.
- Acceptance greps: `every Sway window` in README = 0; `agent\.` in ADR = 1; `status: "Accepted"` = one line (line 4).
- No cargo run, no commit. Note: `git diff --stat` also shows `src/cli.rs` modified; not by this task.

## 17.1.1
- Status: done, not committed.
- Edited only `src/cli.rs`: added `next_flat_agent_index` above `focus_cycle_standalone`; replaced the inline empty check and `focused_idx`/`next_idx` with a `let Some(..) else` that prints the same "No active agent windows found to focus." message and returns Ok.
- Added `test_flat_fallback_cycles_agent_windows_in_tree_order` (empty, middle, last wraps, none focused, single focused).
- `cargo test --lib test_flat_fallback_cycles_agent_windows_in_tree_order`: 1 passed. `cargo check`: clean.
- Full suite not run, per instruction.
- Note: `git diff --stat` also shows `README.md` and `docs/decisions/0003-daemon-owns-agent-state.md` modified. Not by this task (likely parallel workers).

## 17.1.3
- Created `planning/archive/` and `planning/archive/tasks/`.
- `git mv`: 9 top-level D2 files into `planning/archive/`, 22 task files (waves <= 14) into `planning/archive/tasks/`. Nothing deleted.
- Left in place: PROMPT_TEMPLATE.md, GOALS.md, *_15/_16/_17, tasks/15.*, 16.*, 17.*, both design docs.
- Edited only `AGENTS.md` lines 31, 33, 44 (diff: 3 insertions, 3 deletions).
- `ls planning/archive | wc -l` = 10 (>= 9).
- `git status --short planning` shows 31 `R` entries. Path-limited to `planning/archive` it shows `A` because the sources are outside the pathspec; the renames are intact.
- `ls planning/EXECUTION_PLAN.md planning/STATE.md` gives 2 "No such file".
- `grep -n 'STATE.md' AGENTS.md` returns nothing.
- Not committed.
