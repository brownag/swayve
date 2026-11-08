# Orchestration 18 State: Per-User Runtime Paths and Small Hardening/Docs Items (G17–G21)

Plan: `planning/EXECUTION_PLAN_18.md`
Last Updated: 2026-11-07

## Baseline (before Wave 1)
- Orchestration 17 complete. Commits `a265a3e`, `17c5880`, `24783dd` on top of `03a57c8`.
- `make all` (Orchestration 17 gate): 26 unit passed, 10 integration passed, release build OK, no warnings from `cargo check`.
- `cargo clippy --all-targets`: 15 warnings in lib, 6 in integration tests (not yet fixed; G19).
- Working tree clean at plan time.

## Wave Board

### Wave 1: Runtime-Dir Socket and Logs
- [ ] **Task 18.1.1**: Runtime-dir socket and logs; ownership-checked stale removal (G17)

### Wave 1 end gate
- [ ] `make all` passes serially; expect 27 unit + 10 integration

### Wave 2: Docs, Clippy, Test Target, Unit Tests
- [ ] **Task 18.2.1**: README CLI reference complete (G18)
- [ ] **Task 18.2.2**: Clippy clean (G19) (after 18.1.1)
- [ ] **Task 18.2.3**: `make test` runs Pi and hook tests (G20)
- [ ] **Task 18.2.4**: Unit tests for `sway.rs` and `types.rs` (G21)

### Wave 2 end gate
- [ ] `make all` passes serially; `make test` runs Rust, Pi and hook suites

### Deferred (manual, not gated)
- [ ] Live check: daemon started with `XDG_RUNTIME_DIR` set creates `$XDG_RUNTIME_DIR/swayve/swayve.sock`; directory mode 0700.

## Task Reports

_Workers append here. Orchestrator distills each report to 15 lines or fewer._

## 18.1.1
- Done in `src/cli.rs`, `src/daemon.rs`, `README.md:32`; no commit.
- cli.rs: added `runtime_dir_from`, `swayve_runtime_dir`, `current_uid` (/proc/self), `ensure_private_dir` (0700, owner and mode check).
- Socket: `SWAYVE_SOCKET` if set, else `$XDG_RUNTIME_DIR/swayve/swayve.sock`; resolved lazily (skipped for init-shell and panel). Logs go to `<runtime>/swayve/logs/`.
- daemon.rs: the monitor `tail -f` uses the runtime logs dir (logs and skips on error). Stale removal uses `symlink_metadata`, and only removes a socket owned by the current uid; it errors otherwise.
- Tests added: `test_runtime_dir_requires_xdg_runtime_dir` (cli), `test_stale_socket_removal_refuses_regular_file` (daemon). Both pass; `cargo check` is clean; lib tests 28/28 pass.
- FAIL: `cargo test` has one failure, `tests/integration_tests.rs::test_hooks_and_monitoring` (line ~950). It asserts the old `/tmp/swayve/logs/...` tail path, and the monitor command now gives `/run/user/1000/swayve/logs/...`. It passes on the stashed baseline.
- That file is out of scope (constraint). The orchestrator must decide: update the assertion (a one-line path change) or re-scope.
- Also changed one assertion in tests/integration_tests.rs (~line 951) to build the expected tail path from XDG_RUNTIME_DIR (approved).

## 18.2.1
- Edited `README.md` only (CLI Command Reference section); nothing committed.
- Added `init-shell` entry (`eval "$(swayve init-shell)"`, prints bash/zsh prompt hook config).
- Changed status entry to `swayve status [--json] [--watch]` with a note that `--watch` (`-w`) refreshes live (verified against `src/cli.rs` Status/`run_status_watch`).
- Added `focus --cycle` entry: cycles focus through agent windows (daemon-assisted, falls back to a direct cycle if the daemon is down, per `src/cli.rs:431-450`).
- Acceptance greps: `init-shell` = 1 (>=1 OK); `status \[--json\] \[--watch\]` = 1 (OK); `focus --cycle` = 2 (>=2 OK).
- No other tests run (docs-only change).

## 18.2.3
- Edited `Makefile` only: `test:` now runs `cargo test`, `cd integrations/pi && npm test`, `bash integrations/claude/test-hooks.sh`. Help line now reads "Run Rust, Pi and hook tests". `all` unchanged (`check test build`).
- `make -n test` shows the three commands in order.
- `make test` exits 0: cargo 28 + 10 tests pass, Pi `# pass 3` / `# fail 0`, hooks "All integration test assertions passed successfully!".
- Working directory: `test-hooks.sh` uses `BASH_SOURCE`, so exit 0 from repo root and from `integrations/claude`. `npm test` works from `integrations/pi`.
- Not committed.

## 18.2.4
- Status: done, not committed.
- Extracted the inline match in `SwayManager::run_executor` into `impl SwayCommand { pub fn to_command_string(&self) -> String }` (src/sway.rs); strings unchanged, `run_executor` now calls it.
- Added `sway::tests::renders_every_variant` (all 8 variants pinned).
- Added `types::tests::agent_status_roundtrip` (all 6 variants incl. `Failed`, JSON pinned) and `types::tests::agent_id_marks` (`to_mark`, `to_children_mark`).
- `cargo test --lib sway::` 1 passed; `cargo test --lib types::` 2 passed; `cargo check` clean.
- Files touched: src/sway.rs, src/types.rs, planning/tasks/18.2.4.todo.md, this file. clippy not run.

## 18.2.2
- Status: done, not committed.
- Ran `cargo clippy --fix` (edition 2024 let-chains) on `src/cli.rs` (4), `src/daemon.rs` (9), `src/layout.rs` (2), `tests/integration_tests.rs` (5), then reviewed the diff by hand.
- `strip_prefix("agent.monitor.")` replaces manual slicing in daemon.rs and layout.rs; `format!` in cli.rs replaced; `impl Default for LayoutManager` added.
- daemon.rs 371-377 audit chain collapsed to let-chains; checked it has no else branches (the outer `else if` was the last arm), so fallthrough and side effects are identical.
- Test match arms folded into guards; the `_ => {}` arm absorbs non-matching marks as before.
- `cargo clippy --all-targets`: zero warnings. No `#[allow]` added.
- `cargo test`: 31 lib + 10 integration pass.
- Files touched: the four in-scope files and this file. `planning/tasks/18.2.2.todo.md` was created in the plan phase.
