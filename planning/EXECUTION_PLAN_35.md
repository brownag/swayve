# Execution Plan 35: External Swayve HUD Diff Consumer (G53)

Source: `planning/external/hud-diff-consumer.prompt.md`
Target repo: `/home/andrew/workspace/swayve-hud` (migrated to `crates/swayve-hud`)
Recon base: initial empty repository (`master`, untracked planning files).

## Goal & Scope

**Goal.**
Create the standalone `swayve-hud` Rust repository at `~/workspace/swayve-hud` (now integrated at `crates/swayve-hud`) that consumes Swayve's `daemon.subscribe_diff` stream over a Unix domain socket, maintaining a folded live agent map and rendering it to the terminal with in-place ANSI updates, recovering automatically across socket disconnects and daemon restarts.

**In scope.**
- Rust binary crate scaffold (Edition 2024, `serde_json` dependency, `Cargo.toml`, `.gitignore`, `Makefile`, `README.md`).
- Data structures (`DiffFrame`, `AgentWindow`, `AgentStatus`) and fold engine (`fold_diff_line`) with unit tests covering snapshots, upserts, removes, and ignoring non-JSON lines.
- Socket connection, subscription handshake (`{"method":"daemon.subscribe_diff"}\n`), and resilient reconnect loop with bounded exponential backoff (0.5s to 5.0s cap).
- In-place ANSI terminal renderer (`\x1b[2J\x1b[H`) showing active agent status, id, mark, and project hierarchy.
- Integration tests in `tests/` using a mock Unix socket to verify handshake, frame consumption, and reconnect on disconnect.
- Automated gate validation (`make all` running `check`, `test`, and `build` cleanly).

**Out of scope.**
- Any edits to the `~/workspace/swayve` repository during external orchestration.
- Adding git remotes or remote-write git commands (local commits only).
- Using `daemon.subscribe` (the full-list stream); HUD strictly consumes `daemon.subscribe_diff`.
- Async runtimes (e.g. `tokio`); using standard library blocking `UnixStream`.
- Complex TUI frameworks (e.g. `ratatui`); stdout rendering via ANSI escape sequences.

**Verbatim constraints.**
- "You are creating the HUD repository, which is outside the swayve repo. Its location is `~/workspace/swayve-hud`. It does not exist yet. Do not edit anything in the swayve repo. Do not add a git remote, push, or run any remote-write git command. Local git init and commits are allowed."
- "The HUD shows the live agent tree from Swayve, using `daemon.subscribe_diff` and not the full-list stream. Done means the HUD matches the daemon's state after a daemon restart, and every item in 'Acceptance' passes."
- "Dependencies: serde_json only. No async runtime needed: a blocking std::os::unix::net::UnixStream is enough for one subscriber."
- "Socket path: $SWAYVE_SOCKET if set; otherwise $XDG_RUNTIME_DIR/swayve/swayve.sock."
- "Send one line: `{\"method\":\"daemon.subscribe_diff\"}\\n`. Read newline-delimited JSON."
- "Skip any line that is not JSON. Change nothing and do not re-render."
- "On EOF or a read error, reconnect with a bounded backoff (for example 0.5s doubling to 5s, then capped). Do not spin."

## Recon (facts)

- **Socket path convention:** `$SWAYVE_SOCKET` takes precedence if non-empty; falls back to `$XDG_RUNTIME_DIR/swayve/swayve.sock` (from swayve `src/cli.rs:91-98` and ADR 0012).
- **Wire protocol:**
  - Method: `{"method":"daemon.subscribe_diff"}\n`
  - Diff frames: `{"snapshot":bool,"upsert":[AgentWindow...],"remove":["agent_id",...]}`.
  - Key order in wire frames is alphabetical (`remove`, `snapshot`, `upsert`), parsers must not depend on key order.
  - First frame is always a snapshot (`snapshot: true`, `remove: []`, `upsert: [...]`).
  - Lags/drops cause the daemon to set `needs_snapshot` and emit a snapshot on the next state broadcast (ADR 0008 / ADR 0013).
- **Fold behavior:** Reference implementation in swayve `src/cli.rs:488-494` (`fold_diff_line`) and `src/types.rs:133-143` (`DiffFrame::fold_into`). Snapshot clears map; upsert replaces or inserts by id; remove deletes by id.
- **Environment & toolchain:** Rust edition 2024, Cargo available, local git repository initialized.

## Decision Log

- **D1 (2026-11-08): ANSI in-place update.** The HUD clears screen (`\x1b[2J\x1b[H`) and renders the formatted agent list/tree on each change. Non-JSON or undecodable lines trigger no clear or re-render.
- **D2 (2026-11-08): Direct `serde_json::Value` parsing (no `serde` derive dependency).** Because the dependency constraint strictly allows `serde_json` only (without `serde`'s derive macros), `DiffFrame`, `AgentWindow`, and `AgentStatus` are parsed directly from `serde_json::Value` via `from_value` helper functions. `AgentStatus` is modeled as a typed enum with known variants (`Idle`, `Thinking`, `ExecutingTool`, `AwaitingHumanInput`, `Completed`, `Failed(String)`) and a fallback variant (`Other(serde_json::Value)`) to prevent future daemon status additions from invalidating diff frames while rejecting malformed non-string/non-object statuses.
- **D3 (2026-11-08): Mock Unix socket integration test.** In addition to unit tests in `src/fold.rs`, a lightweight mock Unix socket test is added to `tests/integration_test.rs` to verify connection handshake, subscription payload, and reconnect on disconnect.
- **D4 (2026-11-08): Sequential wave execution.** 1 worker per wave across 4 sequential waves following the dependency chain (Scaffold -> Fold Engine -> Socket Loop & Renderer -> Integration Test & Gate).

## Hazard Notes

- **H1. External repo isolation:** Strictly no modifications to `/home/andrew/workspace/swayve`. All file modifications and created artifacts must stay within `/home/andrew/workspace/swayve-hud`.
- **H2. No git remote operations:** Never run `git push`, `git remote add`, or any remote git command.
- **H3. Socket path emptiness:** If `SWAYVE_SOCKET` is set to an empty string (`""`), treat it as unset and fall back to `$XDG_RUNTIME_DIR/swayve/swayve.sock`.
- **H4. Line parsing safety:** Use `BufRead::lines` over `UnixStream`. Malformed or non-JSON lines must return `false` without panicking or modifying the state map.
- **H5. Non-blocking backoff:** Do not spin in a tight loop when connection drops. Sleep with exponential backoff: 500ms, doubling up to 5000ms maximum. Reset backoff upon successful frame receipt.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 35.1.1 | Scaffold crate, build harness, README & git | `Cargo.toml`, `.gitignore`, `Makefile`, `README.md`, `src/main.rs` | — |
| 2 | 35.2.1 | Implement types, fold engine & unit tests | `src/types.rs`, `src/fold.rs`, `src/main.rs` | 35.1.1 |
| 3 | 35.3.1 | Implement socket client, backoff loop & ANSI renderer | `src/lib.rs`, `src/main.rs`, `src/client.rs` | 35.2.1 |
| 4 | 35.4.1 | Mock Unix socket integration test & full gate validation | `tests/integration_test.rs`, `Makefile` | 35.3.1 |

---

## Task 35.1.1: Scaffold crate, build harness, README & git

**Repo:** swayve-hud. **Wave:** 1.

**Files in scope.**
- can create: `Cargo.toml`, `.gitignore`, `Makefile`, `README.md`, `src/main.rs`

**Depends on:** none.

**Spec.**
1. Initialize cargo binary crate in `/home/andrew/workspace/swayve-hud` with Edition 2024 and dependency `serde_json = "1.0"`.
2. Add `.gitignore` ignoring `/target` and `/.development`.
3. Create `Makefile` with targets:
   - `check`: `cargo check`
   - `test`: `cargo test`
   - `build`: `cargo build --release`
   - `all`: `check test build`
   - `clean`: `cargo clean`
4. Create `README.md` covering:
   - What `swayve-hud` does.
   - Socket path resolution rules (`$SWAYVE_SOCKET` or `$XDG_RUNTIME_DIR/swayve/swayve.sock`).
   - How to build and run (`make all`, `cargo run`).
5. Create minimal placeholder `src/main.rs` that compiles.
6. Verify with `make all`.

**Anchors.** Standard Rust 2024 binary crate structure.

**Reference implementation.** swayve `Makefile` and `Cargo.toml`.

**Constraints.** No extra dependencies beyond `serde_json` and standard library. No git remote operations.

**Acceptance.**
- `make all` exits 0 with zero warnings.
- `Cargo.toml` specifies `edition = "2024"` and `serde_json`.
- `README.md` and `Makefile` exist.

---

## Task 35.2.1: Implement types, fold engine & unit tests

**Repo:** swayve-hud. **Wave:** 2.

**Files in scope.**
- can create: `src/types.rs`, `src/fold.rs`
- must edit: `src/main.rs` (module declarations)

**Depends on:** 35.1.1.

**Spec.**
1. In `src/types.rs` (using `serde_json::Value` only, no `serde` crate dependency):
   - Define `AgentStatus` enum (`#[derive(Debug, Clone, PartialEq)]`): `Idle`, `Thinking`, `ExecutingTool`, `AwaitingHumanInput`, `Completed`, `Failed(String)` (matching `{"failed": "<msg>"}`), and `Other(String)` for unknown string statuses. Implement `AgentStatus::from_value(v: &serde_json::Value) -> Option<Self>` and `Display`.
   - Define `AgentWindow` struct (`#[derive(Debug, Clone, PartialEq)]`) with `id: String`, `project: String`, `parent_id: Option<String>`, `container_id: Option<i64>`, `app_id: String`, `mark: String`, `status: AgentStatus`. Implement `AgentWindow::from_value(v: &serde_json::Value) -> Option<Self>` (requiring `id`, `project`, `app_id`, `mark`, `status`; optional/nullable `parent_id` and `container_id`; ignoring extra keys).
   - Define `DiffFrame` struct (`#[derive(Debug, Clone, PartialEq)]`): `snapshot: bool`, `upsert: Vec<AgentWindow>`, `remove: Vec<String>`. Implement `DiffFrame::from_str(s: &str) -> Option<Self>` requiring `snapshot` (bool), `upsert` (array of valid `AgentWindow`), and `remove` (array of strings).
2. In `src/fold.rs`:
   - Implement `fold_diff_line(map: &mut HashMap<String, AgentWindow>, line: &str) -> bool`:
     - Parses line as `DiffFrame`. Returns `false` on any parse error.
     - If `snapshot` is `true`, clears `map`.
     - Iterates `upsert`, inserting each `AgentWindow` keyed by `id`.
     - Iterates `remove`, removing each agent id from `map`.
     - Returns `true`.
3. Add unit tests in `src/fold.rs` under `#[cfg(test)] mod tests`:
   - Sequence test: feed a snapshot line, an upsert line, and a remove line; verify map at each step.
   - Non-JSON / invalid line test: feed invalid JSON; verify returns `false` and map is unmodified.
   - Snapshot clearing test: verify snapshot purges pre-existing entries before applying upserts.
4. Verify with `cargo test`.

**Anchors.** swayve `src/types.rs:30-40, 117-143`, `src/cli.rs:488-494, 863-890`.

**Reference implementation.** `fold_diff_line` in swayve `src/cli.rs:488-494`.

**Constraints.** Pure logic; no network calls; zero warnings.

**Acceptance.**
- `cargo test` passes all fold unit tests.
- Snapshot clearing, upsert, remove, and invalid JSON lines behave according to spec.
- `make check` exits 0 with zero warnings.

---

## Task 35.3.1: Implement socket client, backoff loop & ANSI renderer

**Repo:** swayve-hud. **Wave:** 3.

**Files in scope.**
- can create: `src/client.rs`, `src/lib.rs`
- must edit: `src/main.rs`

**Depends on:** 35.2.1.

**Spec.**
1. Socket path resolver:
   - Read `$SWAYVE_SOCKET` environment variable; if set and non-empty, use it.
   - Otherwise, read `$XDG_RUNTIME_DIR`; if set and non-empty, use `$XDG_RUNTIME_DIR/swayve/swayve.sock`.
   - Return clean error if socket path cannot be determined.
2. Connection & Subscription:
   - Connect using `std::os::unix::net::UnixStream`.
   - Send `{"method":"daemon.subscribe_diff"}\n` and flush.
3. Resilient reconnect loop:
   - Loop indefinitely: attempt connection; on error or EOF, sleep using bounded exponential backoff (starting at 500ms, doubling up to 5000ms max).
   - Upon successful connection and receiving frames, reset backoff to 500ms.
4. Line reading & ANSI rendering:
   - Read lines with `BufReader::new(stream).lines()`.
   - On each line, call `fold_diff_line(&mut map, &line)`.
   - If `fold_diff_line` returns `false`, skip rendering and do not modify display.
   - If `true`, render the folded agent map to stdout using ANSI clear `\x1b[2J\x1b[H`:
     - Display title/header: `=== Swayve HUD (Live Agents: <count>) ===`
     - Display table or hierarchy of agents sorted by `id`: status, id, app_id, project, mark.
5. Verify with `make check` and `make build`.

**Anchors.** `UnixStream`, `BufReader`, ANSI escape sequences `\x1b[2J\x1b[H`.

**Reference implementation.** swayve `src/cli.rs:619-668` (`run_status_watch`).

**Constraints.** Blocking UnixStream; no async runtime; zero warnings; handles EOF/errors without panic.

**Acceptance.**
- `make build` builds release binary `swayve-hud` cleanly.
- Binary responds to `--help` or runs connection loop gracefully.
- Socket path resolution conforms to `$SWAYVE_SOCKET` and `$XDG_RUNTIME_DIR`.

---

## Task 35.4.1: Mock Unix socket integration test & full gate validation

**Repo:** swayve-hud. **Wave:** 4.

**Files in scope.**
- can create: `tests/integration_test.rs`
- must edit: `Makefile` (ensure integration tests run in `test` target)

**Depends on:** 35.3.1.

**Spec.**
1. Create `tests/integration_test.rs`:
   - Bind a temporary `std::os::unix::net::UnixListener` on a unique path in `target/tmp/`.
   - Implement a mock daemon thread that accepts a client connection, verifies `{"method":"daemon.subscribe_diff"}\n` is received, sends a snapshot frame, sends an upsert frame, and closes the connection.
   - Client thread / helper connects, reads frames, verifies map folding, and detects EOF.
   - Test disconnect & recovery: mock listener sends EOF, pauses, accepts a second connection, and verifies the client successfully connects and receives a fresh snapshot.
2. Ensure `make all` executes unit tests, integration tests, clippy check, and release build.
3. Run `make all` and record full pass count.

**Anchors.** `std::os::unix::net::{UnixListener, UnixStream}`.

**Reference implementation.** swayve `tests/integration_tests.rs`.

**Constraints.** Zero compiler and clippy warnings. Clean temporary socket cleanup.

**Acceptance.**
- `make all` exits 0 with zero warnings.
- All unit and integration tests pass cleanly.
- Verified recovery after socket disconnect.
