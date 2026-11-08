# Orchestration 19 State: Runtime Socket Contract and Small Hygiene Items (G22, G24, G26, G27)

Plan: `planning/EXECUTION_PLAN_19.md`
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Orchestration 18 complete and committed (`01f2da4` is HEAD at plan time). Working tree clean.
- Recorded in Orchestration 18: `cargo test` 31 unit + 10 integration; `cargo clippy --all-targets` clean; `make test` runs Rust, Pi and hook suites.
- G23 (GOALS/STATE agreement on G17): done in the `planning/GOALS.md` rewrite of 2026-11-08. Not a task.
- Live socket check for G17 passed 2026-11-08 (`planning/STATE_18.md`). Kept on record; not re-run.
- Pre-wave 1 (run 2026-11-08, HEAD `01f2da4`): `make all` serially passed. Unit 31 / integration 10 / result ok. Pi suite 3/3, hook test passed.

## Wave Board

### Wave 1: Socket Contract Tests and Docs
- [x] **Task 19.1.1**: `ensure_private_dir` unit tests (G22) — commit `cc3c7b3`
- [x] **Task 19.1.2**: Stale-socket same-owner removal and malformed-line tests (G22, G26) — commit `30f576e`
- [x] **Task 19.1.3**: Antigravity socket default (G24) — commit `161c0de`
- [x] **Task 19.1.4**: README socket contract (G22) — commit `40c2777`; acceptance amended, see its report (line 54 correction and pre-existing line 178 remain)

### Wave 1 end gate
- [x] `make all` passes serially. Actual: 35 unit / 10 integration / pi 3/3 / hook test passed. Recorded by orchestrator.

### Wave 2: Archive Hygiene
- [x] **Task 19.2.1**: Archive Orchestrations 15–17 (G27) — 15 files `git mv`'d, uncommitted at time of gate (see note below). GOALS.md G12 path repointed.

### Wave 2 end gate
- [x] `make all` passes serially: 35 unit / 10 integration / ok. `make test` (Rust, Pi, hook) run as part of `make all`: all green.

### Orchestration close notes
- The GOALS.md edits (the user's uncommitted rewrite, plus the one-line G12 path change) are not committed by the orchestrator. Committing them is the user's call.
- Pre-existing stale `JSON-RPC` mention at `README.md:178` (src/types.rs link description) is out of scope for 19.1.4; follow-up.

### Deferred (manual, not gated)
- None. The G17 live check is recorded in STATE_18.

## Known limits (recorded at plan time)
- Foreign-owner socket and directory cases are not reproducible without root. They are covered by code review only (decision D2 in `EXECUTION_PLAN_19.md`).
- A non-UTF-8 IPC line closes that client (`src/daemon.rs` `handle_client`). Kept as-is; not tested in this orchestration.

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

## 19.1.1
- Status: done.
- Files changed: src/cli.rs lines 633-656 (two tests in `mod tests`, after test_runtime_dir_requires_xdg_runtime_dir); planning/tasks/19.1.1.todo.md.
- Command: `cargo test --lib test_ensure_private_dir` -> 2 passed, 0 failed.
- Deviations: none (Q1 approved: set_permissions(0o755) added to loose-mode test).
- Gotchas: DirBuilderExt/MetadataExt/PermissionsExt imported locally per test; cleanup runs before asserts so failures don't leak dirs.

## 19.1.3
- Status: done.
- Files changed: integrations/antigravity/README.md line 17 only (`SWAYVE_SOCKET` row: example now `$XDG_RUNTIME_DIR/swayve/swayve.sock`; description now the approved default/override sentence); planning/tasks/19.1.3.todo.md (boxes ticked).
- Commands and results: `grep -c '/tmp/swayve.sock' integrations/antigravity/README.md` -> 0; `grep -c 'XDG_RUNTIME_DIR/swayve/swayve.sock' integrations/antigravity/README.md` -> 1.
- Deviations: none.
- Gotchas: the `$` characters are literal inside the backtick-wrapped example cell; the description cell is plain text with `$XDG_RUNTIME_DIR` (no backticks, as specified). No `|` in the new text.

## 19.1.2
- Status: done.
- Files changed: src/daemon.rs (mod tests, new tests at ~614-652, after test_stale_socket_removal_refuses_regular_file); planning/tasks/19.1.2.todo.md.
- `cargo test --lib test_stale_socket_same_owner_is_replaced`: 1 passed, 0 failed.
- `cargo test --lib test_malformed_ipc_line_keeps_client`: 1 passed, 0 failed.
- Full `cargo test --lib` not run (orchestrator runs make all at wave end).
- Deviations: poll also requires a successful UnixStream::connect (approved, Q1). Two targeted commands run separately because the combined acceptance command is invalid cargo syntax.
- Gotchas: DaemonEvent lacks Debug/PartialEq, so matches! is used.

## 19.1.4
- Status: done (amended acceptance approved by coordinator).
- Files changed: README.md lines 32-33 (Mermaid labels now `JSON (newline-delimited, method field)`), lines 50-56 (new `### Socket Contract` with 4 bullets); planning/tasks/19.1.4.todo.md.
- Commands and results: `grep -n 'JSON-RPC' README.md` -> lines 54 and 178 only; `grep -c '### Socket Contract' README.md` -> 1; `grep -c 'newline-delimited' README.md` -> 3; `git diff --stat -- README.md` -> 9 insertions, 2 deletions.
- Deviations: original acceptance `grep -c 'JSON-RPC'` = 0 was not met. Line 54 is the required bullet text ("It is not a JSON-RPC 2.0 envelope."); line 178 is pre-existing and out of scope, left untouched. Amended acceptance approved by coordinator.
- Gotchas: the spec's acceptance and its bullet 3 text conflicted; resolved by amending acceptance, not by editing the bullet.

## 19.2.1
- Status: done (not committed).
- Files moved (git mv, 15 total): planning/EXECUTION_PLAN_15/16/17.md, PROMPT_TEMPLATE_15/16/17.md, STATE_15/16/17.md -> planning/archive/; planning/tasks/15.1.1, 15.1.2, 16.1.1, 17.1.1, 17.1.2, 17.1.3 todo.md -> planning/archive/tasks/.
- Files edited: planning/GOALS.md line 65 only (G12 path `planning/STATE_16.md` -> `planning/archive/STATE_16.md`); planning/tasks/19.2.1.todo.md (boxes ticked).
- Acceptance: `ls planning/EXECUTION_PLAN_1[567].md planning/STATE_1[567].md 2>/dev/null | wc -l` -> 0. `ls planning/archive/EXECUTION_PLAN_1[567].md planning/archive/STATE_1[567].md | wc -l` -> 6. `ls planning/tasks/1[567].* 2>/dev/null | wc -l` -> 0.
- Deviations: none.
- Gotchas: under zsh, an unmatched glob errors before `2>/dev/null` applies ("no matches found"), still printing 0; use `setopt nullglob` or bash to avoid the noise. GOALS.md line 36 (G27 range prose) left unchanged per coordinator Q1. Wave 1 gate recorded by orchestrator per Q2. No commit made.
