# Execution Plan 18: Per-User Runtime Paths and Small Hardening/Docs Items (G17–G21)

## Goal & Scope

**Goal**: From `planning/GOALS.md`:
- **G17**: the daemon socket and agent logs live in a per-user runtime directory, and the daemon never deletes a socket it does not own.
- **G18**: the README CLI reference lists every subcommand, including `init-shell` and `status --watch`.
- **G19**: `cargo clippy --all-targets` is clean.
- **G20**: `make test` also runs the Pi client tests and `integrations/claude/test-hooks.sh`.
- **G21**: the Sway command strings in `src/sway.rs` and the types in `src/types.rs` are pinned by unit tests.

**In scope:**
- Socket path `$XDG_RUNTIME_DIR/swayve/swayve.sock`; logs `$XDG_RUNTIME_DIR/swayve/logs/<agent>.log`; directory created with mode 0700; stale-socket removal only for a socket owned by the current user.
- README CLI reference; Makefile `test` target; clippy fixes; unit tests for pure functions in `sway.rs` and `types.rs`.

**Out of scope:**
- G10 (`XDG_CONFIG_HOME`): deferred, separate by decision.
- G9 (diff broadcast): deferred on a trigger.
- G15 (Antigravity sidecar): blocked outside this repo.
- Any change to daemon protocol or focus behavior.

**Verbatim constraints**: from G17: "The daemon socket and agent logs live in a per-user runtime directory, and the daemon never deletes a socket it does not own." Exit criteria: "the default socket path is `$XDG_RUNTIME_DIR/swayve/swayve.sock` in a directory created with mode 0700. Logs go to the same directory. Stale-socket removal happens only when the file is a socket owned by the current user. `make all` passes. `SWAYVE_SOCKET` is still honored when set."

## Decision Log

- **D1: Bundle G17–G21 as two waves** (2026-11-07, user asked for unblocked work as additional waves). Wave 1 is G17 alone. Wave 2 holds the rest, because G19's clippy fixes touch `src/cli.rs` and `src/daemon.rs`, which G17 also edits.
- **D2: Runtime dir unset: refuse to start** (2026-11-07, recommended option, taken by orchestrator while user said "yes"). The daemon and CLI exit with `XDG_RUNTIME_DIR is not set; refusing to use a shared location`. No fallback to `/tmp`.
- **D3: Logs move with the socket** (2026-11-07, default). Same per-user directory, `logs/` subfolder.
- **D4: Mid-tier worker per task** (2026-11-07, default).
- **D5: Acceptance**: targeted tests per task; `make all` serially at the end of each wave.

## Hazard Notes

- `src/cli.rs:74` hard-codes `/tmp/swayve.sock`; `src/cli.rs:242-244` creates `/tmp/swayve/logs`; `src/daemon.rs:98` builds the monitor `tail -f` path under `/tmp/swayve/logs`.
- `src/daemon.rs:198-202`: `if path.exists() { remove_file }` then `UnixListener::bind`. Replace with an ownership and file-type check before removal.
- Integration tests in `tests/integration_tests.rs` use their own `/tmp/swayve_test_*.sock` paths; they set the path explicitly and do not depend on the default. Keep them unchanged.
- `integrations/claude/test-hooks.sh` uses `mktemp` under `/tmp` for its own mock; it sets `SWAYVE_SOCKET` itself. Keep unchanged.
- The Pi client reads `SWAYVE_SOCKET` (`integrations/pi/src/swayve-client.ts:18`). Do not change it.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **18.1.1** | Runtime-dir socket and logs; ownership-checked stale removal (G17) | `src/cli.rs`, `src/daemon.rs`, `README.md` (only if it names the socket path) | None |
| **Wave 2** | **18.2.1** | README CLI reference complete (G18) | `README.md` | None (Wave 1 may also edit README; run after Wave 1) |
| **Wave 2** | **18.2.2** | Clippy clean (G19) | `src/cli.rs`, `src/daemon.rs`, `src/layout.rs`, `tests/integration_tests.rs` | 18.1.1 |
| **Wave 2** | **18.2.3** | `make test` runs Pi and hook tests (G20) | `Makefile` | None |
| **Wave 2** | **18.2.4** | Unit tests for `sway.rs` and `types.rs` pure functions (G21) | `src/sway.rs`, `src/types.rs` | None |

Wave 2 file sets are disjoint except 18.2.1 and 18.1.1, which may both touch `README.md`. Wave 2 starts only after Wave 1 is committed; 18.2.1 runs after 18.1.1 in serial order.

---

## Task Specifications

### Task 18.1.1: Runtime-Dir Socket and Logs

**ID**: `18.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs`, `src/daemon.rs`
- may edit: `README.md` only if a line names `/tmp/swayve.sock` or `/tmp/swayve/logs`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add a pure helper in `src/cli.rs`: `fn swayve_runtime_dir() -> anyhow::Result<std::path::PathBuf>`. Reads `XDG_RUNTIME_DIR`. If unset or empty, returns an error with the message `XDG_RUNTIME_DIR is not set; refusing to use a shared location`. Otherwise returns `<XDG_RUNTIME_DIR>/swayve`.
2. Replace the hard-coded `/tmp/swayve.sock` (`src/cli.rs:74`) with `<runtime_dir>/swayve.sock` unless `SWAYVE_SOCKET` is set, in which case use it unchanged. Create the runtime directory with mode 0700 (`std::os::unix::fs::DirBuilderExt`) if missing; if it exists, check that it is owned by the current user and has no group or other permission bits, and error otherwise.
3. Replace the logs path (`src/cli.rs:242-244`) with `<runtime_dir>/logs/<agent>.log`. Create `logs/` with mode 0700.
4. In `src/daemon.rs`, replace the removal at lines 198-199 with: if the path exists, read its metadata with `symlink_metadata`; remove only if `file_type().is_socket()` and `uid` equals the current effective uid (`std::os::unix::fs::MetadataExt`). Otherwise return an error naming the path and do not remove it.
5. In `src/daemon.rs:98`, build the monitor command from the same logs directory (use the helper; fall back to the same error if unset).
6. Keep `SWAYVE_SOCKET` honored everywhere it is read today.

**Constraints (do not touch)**:
- `tests/integration_tests.rs` (its own socket paths).
- `integrations/` (all of it).
- `src/sway.rs`, `src/layout.rs`, `src/types.rs`, `src/config.rs`.
- No new dependencies. Use `std` and existing `anyhow`.

**Grep anchors**:
- `/tmp/swayve.sock` (`src/cli.rs:74`)
- `/tmp/swayve/logs` (`src/cli.rs:242-244`, `src/daemon.rs:98`)
- `path.exists()` (`src/daemon.rs:198`)
- `UnixListener::bind` (`src/daemon.rs:202`)

**Reference implementation**: the stale-file handling at `src/daemon.rs:197-202` (replace the body, keep the bind).

**Acceptance**:
- Add unit test `test_runtime_dir_requires_xdg_runtime_dir` in `src/cli.rs` `mod tests`: unsetting `XDG_RUNTIME_DIR` yields the error message; setting it yields `<dir>/swayve`. Use a helper that takes the value as an argument so the test does not mutate the process environment.
- `cargo test --lib test_runtime_dir_requires_xdg_runtime_dir` passes.
- `cargo test --test integration_tests` passes unchanged.
- `grep -rn '/tmp/swayve' src` returns no line except in test code.

**Targeted test**: `cargo test --lib test_runtime_dir_requires_xdg_runtime_dir`

---

### Task 18.2.1: README CLI Reference Complete

**ID**: `18.2.1`
**Files-in-scope**: must edit `README.md` (the "CLI Command Reference" section, around lines 103-126).
**Depends-on**: 18.1.1 (serial; both may touch README)

**Spec prose**:
1. Read the subcommands in `src/cli.rs` (`enum Commands`, around lines 25-70).
2. Add an entry for `swayve init-shell` (what it prints; use `eval "$(swayve init-shell)"` as shown in `integrations/antigravity/README.md:105`).
3. Add `swayve status [--json] [--watch]` (the current entry shows only `--json`).
4. Add `swayve focus --cycle` with a one-line description.
5. Change nothing else in the README.

**Grep anchors**: `enum Commands` (`src/cli.rs`), `CLI Command Reference` (`README.md`).

**Acceptance**:
- `grep -c 'init-shell' README.md` returns at least 1.
- `grep -c 'status \[--json\] \[--watch\]' README.md` returns 1.
- `grep -c 'focus --cycle' README.md` returns at least 2.

**Targeted check**: the three greps above.

---

### Task 18.2.2: Clippy Clean

**ID**: `18.2.2`
**Files-in-scope**: must edit `src/cli.rs`, `src/daemon.rs`, `src/layout.rs`, `tests/integration_tests.rs`
**Depends-on**: 18.1.1 (edits the same `cli.rs` and `daemon.rs` regions)

**Spec prose**:
1. Run `cargo clippy --all-targets 2>&1 | grep -A3 '^warning'` to list the current warnings (15 lib, 6 tests at plan time; line numbers will have shifted after 18.1.1).
2. Fix each warning with the smallest change that keeps behavior: collapse nested `if`/`if let` as clippy suggests, replace manual prefix stripping with `strip_prefix`, remove the useless `format!`.
3. Add `impl Default for LayoutManager` (delegating to the existing constructor) so clippy's `new_without_default` lint passes.
4. Do not change any behavior, and do not add `#[allow]` attributes.

**Acceptance**:
- `cargo clippy --all-targets` reports zero warnings.
- `cargo test` passes with the same counts as before (26 unit, 10 integration).

**Targeted test**: `cargo clippy --all-targets` and `cargo test --lib`.

---

### Task 18.2.3: `make test` Runs Pi and Hook Tests

**ID**: `18.2.3`
**Files-in-scope**: must edit `Makefile` only.
**Depends-on**: None

**Spec prose**:
1. Change the `test` target so it runs `cargo test`, then `cd integrations/pi && npm test`, then `bash integrations/claude/test-hooks.sh`. Each step fails the target on error.
2. Keep `all` as `check test build`.
3. Update the `help` text line for `test` to say it runs Rust, Pi and hook tests.
4. Do not add CI config.

**Grep anchors**: `^test:` in `Makefile`; `help` text line `test      :`.

**Acceptance**:
- `make test` exits 0 and prints the Pi test summary (3 passing) and `All integration test assertions passed`.
- `make -n test` shows the three commands in order.

**Targeted test**: `make test`

---

### Task 18.2.4: Unit Tests for Sway Commands and Types

**ID**: `18.2.4`
**Files-in-scope**: must edit `src/sway.rs`, `src/types.rs` (test modules only, plus no logic changes).
**Depends-on**: None

**Spec prose**:
1. In `src/sway.rs`, add a `#[cfg(test)] mod tests` that pins the string each `SwayCommand` variant renders to, for every variant (grep anchors: `enum SwayCommand` and its `to_command_string`-style method; read the file first to find the exact name).
2. In `src/types.rs`, add a test module that pins serde round-trips for `AgentStatus` (all variants, including `Failed(String)`) and that `AgentId::to_mark` produces the expected string for a known id (grep `fn to_mark`).
3. Do not change production logic.

**Acceptance**:
- `cargo test --lib sway::` and `cargo test --lib types::` pass, with at least one test in each module.
- `cargo clippy --all-targets` shows no new warnings from these files.

**Targeted test**: `cargo test --lib sway:: types::`

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 18.1.1 | Socket and logs in runtime dir (G17) | Medium (path change; clients follow `SWAYVE_SOCKET`) | ~80 LOC |
| 18.2.1 | README reference complete (G18) | Low | docs |
| 18.2.2 | Clippy clean (G19) | Low | mechanical |
| 18.2.3 | Makefile test covers Pi and hooks (G20) | Low | Makefile |
| 18.2.4 | Command and type tests (G21) | Low | tests only |

## Wave End Gates

- After Wave 1: `make all` serially. Expect 27 unit tests (26 plus the new one) and 10 integration tests.
- After Wave 2: `make all` serially, and `make test` with the Pi and hook suites.

## Manual Verification (deferred)

- Live check of the runtime-dir socket on the user's session (start the daemon with `XDG_RUNTIME_DIR` set, confirm `$XDG_RUNTIME_DIR/swayve/swayve.sock` exists and mode is 0700 on the directory).
