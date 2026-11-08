# Execution Plan 19: Runtime Socket Contract and Small Hygiene Items (G22, G24, G26, G27)

## Goal & Scope

**Goal**: From `planning/GOALS.md` (2026-11-08):
- **G22**: The daemon's runtime socket path is enforced by tests and documented as the external client contract. The Antigravity sidecar (G15) can build against it.
- **G24**: `integrations/antigravity/README.md` names the runtime socket default, not `/tmp/swayve.sock`.
- **G26**: A malformed IPC line does not drop the client; a test pins it.
- **G27**: Plans and state for waves 15–17 move to `planning/archive/`.

G23 (GOALS/STATE agreement on G17) is done in the `GOALS.md` rewrite of 2026-11-08 and is recorded in STATE_19 as a baseline item, not a task.

**In scope:**
- Tests for `ensure_private_dir` (mode 0700, owner check) and for stale-socket removal (accepts a stale socket owned by the current uid, refuses a regular file).
- Test for malformed-line tolerance in `handle_client`.
- README contract paragraph: socket path, framing, `SWAYVE_SOCKET` override.
- Antigravity README socket default.
- Archive move of 15–17 planning artifacts.

**Out of scope:**
- Any change to daemon protocol, focus behavior, or socket binding logic. The stale-socket check already exists (`src/daemon.rs:204-214`). This plan tests it, does not rewrite it.
- G9 (diff broadcast), G10 (`XDG_CONFIG_HOME`): deferred on their triggers.
- G15 (Antigravity sidecar): external, blocked.
- G11: pruned.
- Archiving Orchestrations 18 and 19 (they stay live until closed).
- Non-UTF-8 IPC lines: current behavior (client closes) is kept; not tested here (see Hazard Notes).

**Verbatim constraints** (from `planning/GOALS.md`, G22 exit criteria): "An automated test asserts that the socket directory is created with mode 0700 and owned by the current uid, and that stale-socket removal refuses a non-socket and a foreign-owned path." "`SWAYVE_SOCKET` is still honored when set." "`make all` passes."

## Decision Log

- **D1: G22 is one circled goal; G24, G26, G27 fold in as tasks** (2026-11-08, user said "yes, add other goals"). G24 is the doc half of the socket contract, so it shares a wave with G22.
- **D2: Foreign-owner path is not tested by creating a root-owned file** (2026-11-08, default). Tests run as the normal user, so a foreign-owner case is not reproducible without root. The test covers the non-socket branch and the same-owner stale-socket branch. The foreign-owner check remains code-reviewed only, and STATE_19 says so.
- **D3: The contract paragraph corrects "JSON-RPC" to newline-delimited JSON** (2026-11-08, default). `docs/decisions/0004-newline-delimited-json-ipc.md` is Accepted, and the wire form is `{"method": ...}` per line, not a JSON-RPC envelope. The Mermaid edge labels in `README.md:32-33` are changed to match. Mermaid label edits are in scope only for this contract.
- **D4: `SWAYVE_SOCKET` bypasses `ensure_private_dir`** (2026-11-08, default). It is the user's explicit path. The README contract says so. No code change.
- **D5: Worker tier is mid-tier for tasks touching `src/`, small for docs and planning moves** (2026-11-08, default). This follows the escalation ladder: a small model is retried once, then the task moves up a tier.
- **D6: Wave gates run serially** (2026-11-08, carried from D5 of Plan 18). Full `make all` runs only at wave end.

## Hazard Notes

- `src/daemon.rs:204-214`: stale-socket removal already checks `symlink_metadata`, `is_socket()`, and `uid`. Tests must not change its behavior.
- `src/cli.rs:92-106` `ensure_private_dir`: `DirBuilder::mode(0o700)` followed by an owner and mode check. Test with a temp dir under `std::env::temp_dir()` with a unique name.
- `src/daemon.rs:600-615` `test_stale_socket_removal_refuses_regular_file` already covers the regular-file case. Do not duplicate it; add only the same-owner stale-socket case.
- `src/daemon.rs:244-281` `handle_client`: a parse error logs and continues. A non-UTF-8 line makes `lines.next_line()` return `Err`, which closes the client. The malformed-line test uses valid UTF-8 only.
- `tests/integration_tests.rs` uses its own `/tmp/swayve_test_*.sock` paths. Do not touch.
- `integrations/pi/src/swayve-client.ts:18` reads `SWAYVE_SOCKET`. Do not touch.
- `planning/archive/` already holds Orchestrations 13–14 and older STATE/PLAN files. G27 adds 15–17 without renaming existing archive entries.

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **19.1.1** | `ensure_private_dir` unit tests (G22) | `src/cli.rs` (tests module only) | None |
| **Wave 1** | **19.1.2** | Stale-socket same-owner removal test and malformed-line test (G22, G26) | `src/daemon.rs` (tests module only) | None |
| **Wave 1** | **19.1.3** | Antigravity socket default (G24) | `integrations/antigravity/README.md` | None |
| **Wave 1** | **19.1.4** | README socket contract paragraph and Mermaid labels (G22) | `README.md` | None |
| **Wave 2** | **19.2.1** | Archive Orchestrations 15–17 planning files (G27) | `planning/archive/**`, `planning/EXECUTION_PLAN_1[567].md`, `planning/STATE_1[567].md`, `planning/PROMPT_TEMPLATE_1[567].md`, `planning/tasks/1[567].*` | Wave 1 end gate |

Wave 1 file sets are pairwise disjoint: `src/cli.rs`, `src/daemon.rs`, `integrations/antigravity/README.md`, `README.md`. Wave 2 is a single task.

---

## Task Specifications

### Task 19.1.1: `ensure_private_dir` Unit Tests

**ID**: `19.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/cli.rs` (inside `mod tests` only, at the bottom of the existing tests)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `test_ensure_private_dir_creates_0700` in `mod tests`. Use a unique path `std::env::temp_dir().join(format!("swayve_epd_{}", std::process::id()))` and remove it first if present. Call `ensure_private_dir`. Assert `Ok`, and assert `metadata.mode() & 0o777 == 0o700`.
2. Add `test_ensure_private_dir_rejects_loose_mode`. Create the directory with `std::fs::DirBuilder::new().mode(0o755)`. Assert `ensure_private_dir` returns `Err`. Remove the directory afterward.
3. Do not change the function body in `src/cli.rs`. If a test fails because the function is wrong, stop and report.

**Constraints (do not touch)**:
- Any non-test code in `src/cli.rs`.
- `src/daemon.rs`, `README.md`, `integrations/`.

**Grep anchors**:
- `pub fn ensure_private_dir` (`src/cli.rs:92`)
- `fn test_runtime_dir_requires_xdg_runtime_dir` (`src/cli.rs:623`, add new tests after it)

**Reference implementation**: `test_runtime_dir_requires_xdg_runtime_dir` in the same module (naming and style).

**Acceptance**:
- `cargo test --lib test_ensure_private_dir` runs 2 tests, both pass.
- `cargo test --lib` passes.

**Targeted test**: `cargo test --lib test_ensure_private_dir`

---

### Task 19.1.2: Stale-Socket Same-Owner Removal and Malformed-Line Tolerance

**ID**: `19.1.2`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` (inside `mod tests` only)
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:
1. Add `test_stale_socket_same_owner_is_replaced`. Create a `std::os::unix::net::UnixListener::bind` at a unique temp path, then drop it (leaves a socket file owned by the current uid). Spawn `start_daemon_ipc(path, tx)` with `tokio::spawn`. Poll `symlink_metadata` every 20 ms, up to 2 s, until the path `is_socket()`. Assert it did. Abort the spawned task and remove the file afterward.
2. Add `test_malformed_ipc_line_keeps_client`. Use `tokio::net::UnixStream::pair()`. Spawn `handle_client(server_end, event_tx)`. Write to the client end: a line `not json\n` then `{"method":"daemon.focus"}\n`. Assert the next `DaemonEvent` received on `event_rx` is `DaemonEvent::Focus`. Use a 2 s timeout so a regression fails instead of hanging.
3. Do not change any non-test code. If `handle_client` does not match the Hazard Notes, stop and report.

**Constraints (do not touch)**:
- Non-test code in `src/daemon.rs`.
- `test_stale_socket_removal_refuses_regular_file` (already present; do not edit).
- `src/cli.rs`, `README.md`, `integrations/`.

**Grep anchors**:
- `async fn start_daemon_ipc` (`src/daemon.rs:199`)
- `async fn handle_client` (`src/daemon.rs:240`)
- `async fn test_stale_socket_removal_refuses_regular_file` (`src/daemon.rs:604`, add new tests after it)
- `enum DaemonEvent` (`src/daemon.rs`, the `Focus` variant)

**Reference implementation**: `test_stale_socket_removal_refuses_regular_file` (`src/daemon.rs:604`) for temp path naming and the `#[tokio::test]` shape.

**Acceptance**:
- `cargo test --lib test_stale_socket_same_owner test_malformed_ipc_line` runs 2 tests, both pass.
- `cargo test --lib` passes.

**Targeted test**: `cargo test --lib test_malformed_ipc_line_keeps_client` and `cargo test --lib test_stale_socket_same_owner_is_replaced`

---

### Task 19.1.3: Antigravity Socket Default

**ID**: `19.1.3`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `integrations/antigravity/README.md` (line 17 only)

**Depends-on**: None

**Spec prose**:
1. In the environment variable table, change the `SWAYVE_SOCKET` example from `/tmp/swayve.sock` to `$XDG_RUNTIME_DIR/swayve/swayve.sock`.
2. Change its description to: `Path to the active Swayve IPC Unix domain socket. Default: $XDG_RUNTIME_DIR/swayve/swayve.sock. Set SWAYVE_SOCKET to override.`
3. Change nothing else in the file.

**Constraints (do not touch)**: everything except line 17. Do not edit `integrations/pi/` or `integrations/claude/`.

**Grep anchors**: `SWAYVE_SOCKET` (`integrations/antigravity/README.md:17`).

**Acceptance**:
- `grep -c '/tmp/swayve.sock' integrations/antigravity/README.md` returns 0.
- `grep -c 'XDG_RUNTIME_DIR/swayve/swayve.sock' integrations/antigravity/README.md` returns at least 1.

**Targeted test**: the two greps above.

---

### Task 19.1.4: README Socket Contract

**ID**: `19.1.4`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `README.md` (the Mermaid edges at lines 32-33, and one new short subsection placed directly after the system topology diagram)

**Depends-on**: None

**Spec prose**:
1. Change the Mermaid edge labels `JSON-RPC (agent.register)` and `JSON-RPC (agent.update_status)` to `JSON (newline-delimited, method field)`. Keep the arrow and node structure.
2. Add a subsection `### Socket Contract` after the topology diagram block, with 4 bullets:
   - Default socket: `$XDG_RUNTIME_DIR/swayve/swayve.sock`. Requires `XDG_RUNTIME_DIR`; the daemon refuses to start without it.
   - The directory is created with mode 0700 and must be owned by the current user.
   - Framing: one JSON object per line, with a `method` field (see `docs/decisions/0004-newline-delimited-json-ipc.md`). It is not a JSON-RPC 2.0 envelope.
   - `SWAYVE_SOCKET` overrides the path and skips the directory check.
3. Change nothing else in the README.

**Constraints (do not touch)**: all other README sections; `integrations/`; `docs/decisions/`.

**Grep anchors**:
- `System Topology` (`README.md:~26`)
- `JSON-RPC (agent.register)` (`README.md:32`)
- `JSON-RPC (agent.update_status)` (`README.md:33`)

**Acceptance**:
- `grep -c 'JSON-RPC' README.md` returns 0.
- `grep -c '### Socket Contract' README.md` returns 1.
- `grep -c 'newline-delimited' README.md` returns at least 2 (ADR link line and the bullet).

**Targeted test**: the three greps above.

---

### Task 19.2.1: Archive Orchestrations 15–17

**ID**: `19.2.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `planning/archive/` (add files only); `planning/GOALS.md` (update archive references only)
- must move (`git mv`): `planning/EXECUTION_PLAN_15.md`, `planning/EXECUTION_PLAN_16.md`, `planning/EXECUTION_PLAN_17.md`, `planning/PROMPT_TEMPLATE_15.md`, `planning/PROMPT_TEMPLATE_16.md`, `planning/PROMPT_TEMPLATE_17.md`, `planning/STATE_15.md`, `planning/STATE_16.md`, `planning/STATE_17.md`, `planning/tasks/15.*.todo.md`, `planning/tasks/16.*.todo.md`, `planning/tasks/17.*.todo.md`

**Depends-on**: Wave 1 end gate (19.1.1 to 19.1.4 committed)

**Spec prose**:
1. Confirm that no file in the move list is referenced by `planning/GOALS.md` as an active artifact. Historical references (for example, G12 citing `planning/STATE_16.md`) are allowed; those links must be updated to `planning/archive/STATE_16.md`.
2. `git mv` each file in the move list into `planning/archive/`, or `planning/archive/tasks/` for the task files. Do not rename.
3. Update the `planning/GOALS.md` references that name `planning/STATE_16.md` or `planning/STATE_17.md` to the archive path.
4. Do not move any Orchestration 18 or 19 file. Do not touch `planning/archive/` existing files.

**Constraints (do not touch)**: any file for Orchestrations 18 or 19; `planning/archive/` existing entries; source code.

**Grep anchors**: `planning/STATE_16.md` and `planning/STATE_17.md` in `planning/GOALS.md`.

**Acceptance**:
- `ls planning/EXECUTION_PLAN_1[567].md planning/STATE_1[567].md 2>/dev/null | wc -l` returns 0.
- `ls planning/archive/EXECUTION_PLAN_1[567].md planning/archive/STATE_1[567].md | wc -l` returns 6.
- `ls planning/tasks/1[567].* 2>/dev/null | wc -l` returns 0.

**Targeted test**: the three `ls` counts above.

---

## Wave Gates

- **Pre-wave 1 (baseline)**: run `make all` serially. Record counts in STATE_19. Expected from STATE_18: 31 unit, 10 integration.
- **Wave 1 end gate**: `make all` serially. Expect 35 unit (31 + 2 from 19.1.1 + 2 from 19.1.2) and 10 integration (unchanged). Record actual counts in STATE_19.
- **Wave 2 end gate**: `make all` serially. `make test` runs Rust, Pi and hook suites.
- **G22 closure**: manual confirmation is not needed; the automated tests cover the exit criteria. The live socket check (STATE_18) remains on record.
