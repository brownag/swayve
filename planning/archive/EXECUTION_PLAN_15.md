# Execution Plan 15: Config Parse Errors Are Reported; Children-Mark Rehome Covers Multiple Survivors (Goals G3, G4)

## Goal & Scope

**Goal**: Two independent gaps from `planning/GOALS.md`:
- **G3**: a malformed `~/.config/swayve/config.toml` prints a clear error to stderr and the daemon continues with defaults. Today `load_config` (`src/config.rs:97`) returns `Config::default()` silently on a parse failure (ADR 0010 negative).
- **G4**: the children-mark rehome rule is covered when more than one sibling survives. Today `test_window_destroyed_rehomes_children_mark` (`src/layout.rs:338`) has only one survivor, so the "lowest con_id" choice is untested (ADR 0007 negative; commit `08ad0b6`).

**In scope:**
- `src/config.rs`: report a parse failure on stderr with the file path and error; keep returning defaults. A missing file stays silent.
- `src/config.rs`: a unit test for the parse-failure path.
- `src/layout.rs`: one added unit test with three children that checks the mark moves to the lowest surviving `con_id`.

**Out of scope:**
- Refusing to start on a malformed config (Decision 2 chose print-and-continue).
- Honoring `XDG_CONFIG_HOME` (G10, deferred).
- Changing the children-mark rehome logic. The existing behavior is correct; the gap is only test coverage.
- Live-Sway verification (deferred from plans 13 and 14).

**Verbatim constraints**: none beyond the goal text in `planning/GOALS.md`.

## Decision Log

- **D1: G4 is one added test, not a closed goal** (2026-11-02). The existing test covers the bug; the three-child case covers the lowest-`con_id` choice.
- **D2: Malformed config prints an error and continues with defaults** (2026-11-02). A missing file stays silent. Refusing to start would make one typo break every command that loads config.
- **D3: Two tasks, one wave, run sequentially** (2026-11-02, recommended default). The tasks touch disjoint files, so they could run in parallel, but sequential runs avoid worktree setup. Confirm if you want them parallel.
- **D4: Mid-tier worker for both tasks** (2026-11-02, recommended default). Matches plans 13 and 14.
- **D5: Acceptance bar** (2026-11-02, recommended default). Each task runs its targeted test. At wave end, `make all` runs serially.

## Hazard Notes

- **`load_config` has one caller** (`src/cli.rs:72`). Keep its signature `pub fn load_config() -> Config` so the caller does not change.
- **Stderr output in tests**: the test must not depend on captured stderr. Test a helper that returns the error as a value; `load_config` only prints it.
- **Shared temp paths**: the config test writes a temp file. Use a unique name, and remove it at the end.
- **Children-mark test shape**: `state_with_children` (`src/layout.rs:317`) builds the parent and children. The new test reuses it; it does not change the helper.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **15.1.1** | Report malformed config on stderr; add parse-failure unit test | `src/config.rs` | None |
| **Wave 1** | **15.1.2** | Add three-child rehome test to layout | `src/layout.rs` | None |

The tasks are file-disjoint and could run in parallel. They run sequentially by D3.

---

## Task Specifications

### Task 15.1.1: Report Malformed Config on Stderr

**ID**: `15.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/config.rs`
- can create: none
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add a helper in `src/config.rs`: `pub fn parse_config_file(path: &std::path::Path) -> Result<Option<Config>, String>`.
   - Returns `Ok(None)` when the file does not exist.
   - Returns `Ok(Some(config))` when it reads and parses.
   - Returns `Err(message)` when it exists but cannot be read or parsed. The message names the path and the underlying error.
2. Rewrite `load_config()` (`src/config.rs:97`) to use the helper:
   - Resolve the path from `HOME` exactly as today: `$HOME/.config/swayve/config.toml`. If `HOME` is unset, return `Config::default()`.
   - `Ok(Some(c))` → return `c`. `Ok(None)` → return `Config::default()` silently.
   - `Err(msg)` → `eprintln!("[config] {}; using defaults", msg)` and return `Config::default()`.
3. Add a unit test in the existing `#[cfg(test)] mod tests` of `src/config.rs`: write a malformed TOML string (for example `daemon = [`) to a unique temp file, call `parse_config_file`, assert it returns `Err` and the message contains the file path. Remove the temp file.
4. Do not change `Config`, `HooksConfig`, or any default values.

**Constraints (do not touch)**:
- `src/cli.rs` (the single caller of `load_config`); `load_config`'s signature stays `pub fn load_config() -> Config`.
- Any other file.
- No new dependencies.

**Grep anchors**:
- `pub fn load_config` (`src/config.rs:97`)
- `fn test_hooks_config_default` (`src/config.rs`, the existing test module to add the new test to)
- `load_config` (`src/cli.rs:72`, reference only)

**Reference implementation**: the existing `load_config` body in `src/config.rs`, for the path resolution; the existing `test_hooks_config_default` for test style.

**Acceptance**: `cargo test --lib config::` passes, including the new test.
**Targeted test**: `cargo test --lib config::`

---

### Task 15.1.2: Three-Child Children-Mark Rehome Test

**ID**: `15.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/layout.rs`
- can create: none
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add a unit test next to `test_window_destroyed_rehomes_children_mark` (`src/layout.rs:338`), named `test_window_destroyed_rehomes_to_lowest_survivor`.
2. Build children with `state_with_children(&[("first", 100), ("second", 300), ("third", 200)])`. Note the registration order differs from `con_id` order, so the test checks `con_id`, not insertion order.
3. Send `LayoutAction::WindowDestroyed { agent_id: AgentId("first".to_string()), parent_id: Some(parent_id) }`, as the existing test does.
4. Assert exactly one command: `SwayCommand::MarkWindow { con_id: 200, mark }` with `mark == "mark_children_parent"`. Survivors are `second` (300) and `third` (200), and 200 is the lowest.
5. Do not change `state_with_children`, `compute_commands`, or the existing test.

**Constraints (do not touch)**:
- `src/daemon.rs` and `src/sway.rs`.
- Any non-test code in `src/layout.rs`.

**Grep anchors**:
- `fn test_window_destroyed_rehomes_children_mark` (`src/layout.rs:338`, the test to model on)
- `fn state_with_children` (`src/layout.rs:317`)
- `LayoutAction::WindowDestroyed` (`src/layout.rs`, `compute_commands`, reference only)

**Reference implementation**: `test_window_destroyed_rehomes_children_mark` in the same file.

**Acceptance**: `cargo test --lib test_window_destroyed_rehomes_to_lowest_survivor` passes.
**Targeted test**: `cargo test --lib layout::`

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 15.1.1 | Malformed config reported on stderr; defaults kept | Low | ~30 LOC |
| 15.1.2 | Lowest-survivor rehome covered | Low (test only) | ~25 LOC |

## Manual Verification (deferred)

- Malformed config: write a bad `~/.config/swayve/config.toml`, run `swayve status`, and confirm the stderr message appears and the command still runs.
- Live-Sway focus check from plan 13 remains outstanding.
