# Execution Plan 26: Pi test sockets out of cwd, spawn_plan request struct

Source: `planning/GOALS.md` (G63, G64). Audit base: `planning/ARCHITECTURE_REVIEW_2026-11-08.md` and the 2026-11-08 workspace audit. Recon base: `planning/STATE_25.md`, commit `9a91b55`.
Status: draft, awaiting approval.

## Goal & Scope

**Goal.** The Pi test suite writes no socket files into the package directory, and `spawn_plan` takes one request struct instead of nine positional arguments, so the `too_many_arguments` lint suppression can be removed.

**In scope.**
- G63: `integrations/pi/tests/swayve-client.test.ts` stops writing `test.sock` and `dummy.sock` into `process.cwd()`.
- G64: `spawn_plan` in `src/cli.rs` takes a `SpawnRequest<'_>`. The `#[allow(clippy::too_many_arguments)]` attribute goes away.

**Out of scope.**
- G65 (daemon-launched monitor terminal env). Retired: the monitor runs `tail -f` with no `SWAYVE_*` env, and nothing reads it.
- Any change to the spawn env contents, the register message, or the dry-run output.
- `src/daemon.rs` and the two ignored benchmarks (G66, Deferred).

**Verbatim constraints.**
- "also we could add 65 if possible" (later retired by the user's choice: "Retire G65")
- "write the goals as above" / "plan for 63 and 64"

## Decision Log

- **D1 (2026-11-08): G63 and G64 share one wave.** Their files are disjoint (`integrations/pi/tests/` vs `src/cli.rs`). The user asked for both.
- **D2 (2026-11-08): G64 uses one borrowed struct, `SpawnRequest<'a>`.** The user chose it over an owned struct and over two structs. It is passed by value, because `command` is an owned `Option<String>` that `spawn_plan` consumes today.
- **D3 (2026-11-08): G65 retired.** The user chose to retire it. Reason: `src/daemon.rs` (`spawn_monitor_terminal`, around line 287) runs the monitor with `.args(...)` and no env, and nothing in the repo reads `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, or `SWAYVE_PROJECT_ID` from that process.
- **D4 (default, for your approval): G63 deletes the `test.sock` path and its `cleanup` test.** Nothing in the suite creates `test.sock` now (`captureRegister` uses its own temp dir). The `cleanup` test only removes that file, so it goes too. `dummy.sock` moves into a per-run `mkdtemp` dir. The Pi test count goes from 6 to 5. Say so if you want to keep the cleanup test.
- **D5 (default, for your approval): models and gate.** Rust worker on sonnet, Pi worker on haiku. Escalation follows the wave-orchestration ladder. The wave gate is a serial `make all`, plus `cargo clippy --all-targets`.

## Hazard Notes (from recon)

- **H1.** `integrations/pi/tests/swayve-client.test.ts` uses `process.cwd()` at lines 10 and 27 only. `captureRegister` (lines 42–66) already uses `fs.mkdtempSync(path.join(os.tmpdir(), 'swayve-pi-'))` and is the pattern to copy.
- **H2.** `spawn_plan` has exactly one production caller (`src/cli.rs:240`, the `Commands::Spawn` arm) and six test calls (`src/cli.rs` around lines 745–795). All six must change.
- **H3.** `command: Option<String>` is consumed by `unwrap_or_else` inside `spawn_plan`. Passing the request by value keeps that ownership with no clone.
- **H4.** In the spawn arm, `parent.map(AgentId)` moves `parent` later in the arm. Build the request with `parent.as_deref()` before that move. Task 25.2.1 handled the same ordering.
- **H5.** `make all` runs `cargo test`, `npm test` in `integrations/pi`, and `test-hooks.sh`. Tests must leave no artifacts (plan 25, H6).

## Wave Table

| Wave | Task | Goal | Files (disjoint within wave) | Depends on |
|---|---|---|---|---|
| 1 | 26.1.1 | G63 | `integrations/pi/tests/swayve-client.test.ts` | — |
| 1 | 26.1.2 | G64 | `src/cli.rs` | — |

Both tasks are in wave 1 with disjoint files. There is one wave, so there is no wave-end sequencing beyond the gate.

## Wave Gate

- Serial `make all` with zero warnings, plus `cargo clippy --all-targets` clean.
- After the wave: `ls integrations/pi/*.sock` finds nothing, and `grep -n "allow(clippy::too_many_arguments)" src/cli.rs` finds nothing.

---

## Task 26.1.1: Pi test sockets leave the package directory (G63)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `integrations/pi/tests/swayve-client.test.ts`
- can create: `.development/` scratch only

**Depends on:** none.

**Spec.**
1. Delete the `const socketPath = path.join(process.cwd(), 'test.sock');` line (line 10) and the `cleanup` test (lines 12–17). Nothing creates `test.sock` now (D4).
2. In `handles connection errors gracefully` (lines 25–40), create a per-run directory before the dummy file: `const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'swayve-dummy-'));`. Set `dummyFile = path.join(dir, 'dummy.sock')`. In the `finally` block, replace the `unlinkSync` with `fs.rmSync(dir, { recursive: true, force: true });`.
3. Keep the other tests and their assertions exactly as they are.

**Anchors.** `integrations/pi/tests/swayve-client.test.ts`: `process.cwd()` (lines 10, 27), `cleanup` test (12–17), `handles connection errors gracefully` (25–40), `captureRegister` (42–66, the pattern to copy).

**Constraints.** Do not change `src/`, `captureRegister`, or any assertion. Do not add a dependency. Do not change `handles missing socket gracefully`.

**Acceptance.**
- `cd integrations/pi && npm test` passes with 5 tests and 0 failures.
- `cd integrations/pi && npm run build` succeeds.
- `grep -n "process.cwd" integrations/pi/tests/swayve-client.test.ts` prints nothing.
- `ls integrations/pi/*.sock` finds nothing after the run.

---

## Task 26.1.2: `spawn_plan` takes a request struct (G64)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `src/cli.rs`
- can create: `.development/` scratch only

**Depends on:** none.

**Spec.**
1. Add `pub struct SpawnRequest<'a>` with these fields, in this order: `agent: &'a str`, `wave: &'a str`, `command: Option<String>`, `prompt: Option<&'a str>`, `term_template: &'a [String]`, `socket_path: &'a str`, `project: &'a str`, `now_ms: u128`, `parent: Option<&'a str>`. Put it directly above `spawn_plan`.
2. Change `spawn_plan` to `pub fn spawn_plan(req: SpawnRequest<'_>) -> SpawnPlan`. Destructure at the top of the body: `let SpawnRequest { agent, wave, command, prompt, term_template, socket_path, project, now_ms, parent } = req;`. The body is otherwise unchanged.
3. Remove `#[allow(clippy::too_many_arguments)]` and the comment above it that says "fixes these 9 parameters".
4. In the `Commands::Spawn` arm (line ~240), replace the nine-argument call with a `SpawnRequest { ... }` literal built from the same values. Use `prompt.as_deref()`, `&config.terminal.args`, `socket_path`, `&project`, `now`, and `parent.as_deref()`. Build it before the `parent.map(AgentId)` move (H4).
5. Update the six test calls (`test_spawn_plan_*` and `test_spawn_plan_env_carries_socket_agent_project`, `..._sets_parent_env_when_given`, `..._omits_parent_env_when_absent`) to build a `SpawnRequest`. A small local helper is allowed if it keeps the diff smaller. Keep every assertion unchanged.

**Anchors.** `src/cli.rs`: `Commands::Spawn` arm, `let plan = spawn_plan(` (~line 240), `pub fn spawn_plan(` (~line 510) with its allow attribute, `struct SpawnPlan` (above it), `mod tests` `test_spawn_plan_*` (~lines 745–795).

**Constraints.** The spawn env contents, the register message, `register_and_wait`, and the dry-run text do not change. Do not change `SpawnPlan`'s fields. Do not add a dependency. Do not touch `src/daemon.rs`.

**Acceptance.**
- `cargo test spawn_plan` passes with 7 tests and 0 failures.
- `cargo check` shows zero warnings.
- `cargo clippy --all-targets` shows zero warnings.
- `grep -n "too_many_arguments" src/cli.rs` finds nothing.

---

## Closing gate

After wave 1, run `make all` serially with zero warnings, plus `cargo clippy --all-targets` clean. Then update `planning/GOALS.md`: move G63 and G64 to Completed with commit hashes, and move G65 to Blocked / Pruned with the reason from D3.
