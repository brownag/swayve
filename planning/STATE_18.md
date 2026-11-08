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
