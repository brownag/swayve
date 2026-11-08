# Execution Plan 25: Spawn env in the plan, parent identity to Pi, temp-path sweep

Source: `planning/GOALS.md` (G59 Circled; G60, G61, G62). Recon base: `planning/STATE_24.md`, commit `b839190`.
Status: draft, awaiting approval.

## Goal & Scope

**Goal.** A spawned agent learns its parent. `swayve spawn --parent X` puts `SWAYVE_PARENT_ID=X` in the child env, the Pi client sends that value as `parent_agent_id` on register, and the spawn env is testable without launching a process. Integration tests stop using `/tmp` literals.

**In scope.**
- G60: the child env (`SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, `SWAYVE_PROJECT_ID`) is built in `spawn_plan`, and the spawn arm executes the plan.
- G59: `SWAYVE_PARENT_ID` is added to that env when `--parent` is given.
- G61: the Pi client reads `SWAYVE_PARENT_ID` and sends `parent_agent_id` (`null` when unset). Pi is the preferred integration.
- G62: per-run temp paths in `integrations/claude/test-hooks.sh` and `integrations/pi/tests/swayve-client.test.ts`.

**Out of scope.**
- The external Antigravity sidecar (G25, G15) and external HUD (G53). Blocked, external watch.
- Daemon-launched monitor terminals (`src/daemon.rs` spawn env). Not requested.
- Wire protocol changes. `agent.register` already accepts `parent_agent_id`.
- Architecture review items 1–4, already landed (`55489c7`, `8460157`, `9843d6f`, `1012fd5`).

**Verbatim constraints.**
- "write GOALS and include 59 and any other goals you can in the plan wave"
- "we do prefer pi in general"

## Decision Log

- **D1 (2026-11-08): order G60 → G59 → G61.** G59 and G60 both edit the `Commands::Spawn` arm in `src/cli.rs`, so they cannot share a wave. G60 moves env into the plan first, so G59 is a one-line addition with a pure test.
- **D2 (2026-11-08): G62 includes the Pi test.** `integrations/pi/tests/swayve-client.test.ts` also uses `/tmp/non-existent.sock`.
- **D3 (2026-11-08): G61 is wave 3, after G59.** It follows the declared dependency. Pi work starts after the env path is in place.
- **D4 (defaults, not asked):** the wave gate is serial `make all` at wave end. Workers use the cheapest capable model, escalating per the wave-orchestration ladder. No ADR is superseded. ADR 0006 (identity via env) covers `SWAYVE_PARENT_ID`.

## Hazard Notes (from recon)

- **H1. The arch review item 5 is stale.** `spawn_plan` already exists and is pure (`src/cli.rs:507`). What remains inline is the env (`src/cli.rs` `Command::new(...).env(...)`, around line 285). G60 targets that.
- **H2.** `spawn_plan` has three existing tests (`test_spawn_plan_*`, around `src/cli.rs:731`). Any signature change must update them.
- **H3.** Pi's `register()` (`integrations/pi/src/swayve-client.ts:56`) omits `parent_agent_id` entirely. G61 adds the key, with `null` when unset.
- **H4.** The Pi test writes `test.sock` and `dummy.sock` into `cwd`. Keep that behavior when changing paths.
- **H5.** `integrations/claude/test-hooks.sh` uses `SOCKET_PATH="/tmp/..."` and `mktemp /tmp/...`. Unix socket paths are capped near 108 bytes, so keep names short.
- **H6.** `make all` runs `cargo test`, `npm test` in `integrations/pi`, and `test-hooks.sh`. Tests must leave no artifacts.

## Wave Table

| Wave | Task | Goal | Files (disjoint within wave) | Depends on |
|---|---|---|---|---|
| 1 | 25.1.1 | G60 | `src/cli.rs` | — |
| 1 | 25.1.2 | G62 | `integrations/claude/test-hooks.sh`, `integrations/pi/tests/swayve-client.test.ts` | — |
| 2 | 25.2.1 | G59 | `src/cli.rs` | 25.1.1 |
| 3 | 25.3.1 | G61 | `integrations/pi/src/swayve-client.ts`, `integrations/pi/tests/swayve-client.test.ts`, `integrations/pi/README.md` | 25.2.1 |

Wave 1 has disjoint files. Wave 2 and wave 3 are single tasks. Wave 3 touches the Pi test file, which 25.1.2 edits in wave 1, so the sequencing keeps them apart.

## Wave Gate

- Each wave: `make all` run serially, with zero warnings, plus `cargo clippy --all-targets` clean.
- After wave 3: `cargo test` for the spawn tests, `npm test`, and a live check. Start `make daemon-dry`, then run `swayve spawn --parent <id>` against a dry-run emulator and confirm the env. A dry run prints the plan, so the live check is confirmed by the unit tests, not by a process.

---

## Task 25.1.1: Spawn child env moves into `spawn_plan` (G60)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `src/cli.rs`
- can create: `planning/tasks/25.1.1.todo.md`, `.development/` scratch only

**Depends on:** none.

**Spec.**
1. Add `pub env: Vec<(String, String)>` to `SpawnPlan` (`src/cli.rs:498`).
2. `spawn_plan` (`src/cli.rs:507`) gains `socket_path: &str` and `project: &str` parameters. It fills `env` with `("SWAYVE_SOCKET", socket_path)`, `("SWAYVE_AGENT_ID", agent_id)`, `("SWAYVE_PROJECT_ID", project)`. Parent is not added here; G59 does that.
3. In the `Commands::Spawn` arm (around line 240 and 285), pass `socket_path` and `&project`, and replace the three `.env(...)` calls with `for (k, v) in &plan.env { cmd.env(k, v); }`.
4. Update the three existing `spawn_plan` tests to the new signature, with the same assertions.
5. Add one test: `test_spawn_plan_env_carries_socket_agent_project`.

**Anchors.** `src/cli.rs`: `struct SpawnPlan` (~498), `fn spawn_plan` (~507), `Commands::Spawn` arm (~228–300), the `Command::new(&config.terminal.default_emulator)` block (~285), `mod tests` for `test_spawn_plan_*` (~731–750).

**Constraints.** Do not change `register_and_wait`, the register message, or the dry-run output text. Do not add a dependency.

**Acceptance.**
- `cargo test spawn_plan` passes, including the new test.
- `cargo check` shows zero warnings.

**Reference.** `spawn_plan` already follows the pure-with-tests pattern. Match it.

---

## Task 25.1.2: Per-run temp paths in integration tests (G62)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `integrations/claude/test-hooks.sh`, `integrations/pi/tests/swayve-client.test.ts`
- can create: `.development/` scratch only

**Depends on:** none.

**Spec.**
1. `test-hooks.sh`: create `MOCK_DIR="$(mktemp -d)"` first, then `SOCKET_PATH="$MOCK_DIR/swayve_claude_test.sock"`, and `LOG_FILE="$(mktemp)"`. No `/tmp` literal anywhere. `mktemp` honors `TMPDIR` and falls back to the system default. The socket lives in `MOCK_DIR` so it is removed with it, and its path stays short.
3. Pi test: replace `'/tmp/non-existent.sock'` with `path.join(os.tmpdir(), 'swayve-missing-' + process.pid + '.sock')`. Use `import os from 'node:os'`.
4. Confirm the Pi test cleanup still removes `test.sock` and `dummy.sock`.

**Anchors.** `integrations/claude/test-hooks.sh` lines 7–9. `integrations/pi/tests/swayve-client.test.ts`: `handles missing socket gracefully` (~line 19).

**Constraints.** Keep the test count and assertions. Do not change `src/`. Socket paths must stay short (under 108 bytes); `mktemp -u` in a standard temp dir satisfies this.

**Acceptance.**
- `bash integrations/claude/test-hooks.sh` passes.
- `cd integrations/pi && npm test` passes.
- `grep -n '/tmp' integrations/claude/test-hooks.sh integrations/pi/tests/swayve-client.test.ts` prints nothing.

---

## Task 25.2.1: `SWAYVE_PARENT_ID` in the spawn env (G59)

**Repo:** swayve. **Wave:** 2.

**Files in scope.**
- must edit: `src/cli.rs`
- can create: `.development/` scratch only

**Depends on:** 25.1.1.

**Spec.**
1. Add `parent: Option<&str>` as the last parameter of `spawn_plan`. When `Some(p)`, push `("SWAYVE_PARENT_ID", p)` onto `env`. When `None`, push nothing.
2. The arm passes `parent.as_deref()`. Check the `Commands::Spawn` `parent` binding: it is `Option<String>` and is moved into `parent.map(AgentId)` at around line 256. Clone it or borrow it before that move.
3. Update the existing `spawn_plan` tests and the 25.1.1 test to pass `None`.
4. Add two tests: `test_spawn_plan_sets_parent_env_when_given` (asserts the env contains `SWAYVE_PARENT_ID` with the value) and `test_spawn_plan_omits_parent_env_when_absent` (asserts no `SWAYVE_PARENT_ID` key).

**Anchors.** `src/cli.rs`: `spawn_plan` (~507), register block using `parent.map(AgentId)` (~256), spawn `Command::new` (~285), spawn tests (~731+).

**Constraints.** The register message is unchanged; the daemon already gets `parent_agent_id`. Only the child env gains the variable. Do not change the daemon.

**Acceptance.**
- `cargo test spawn_plan` passes, including the two new tests.
- `cargo check` shows zero warnings.

**Reference.** `ADR 0006` (identity via env) is the design basis. The env name matches the name G25 expects.

---

## Task 25.3.1: Pi client sends `parent_agent_id` from `SWAYVE_PARENT_ID` (G61)

**Repo:** swayve. **Wave:** 3.

**Files in scope.**
- must edit: `integrations/pi/src/swayve-client.ts`, `integrations/pi/tests/swayve-client.test.ts`, `integrations/pi/README.md`
- can create: `.development/` scratch only

**Depends on:** 25.2.1.

**Spec.**
1. `SwayveClientConfig` (`swayve-client.ts:8`) gains `parentAgentId?: string`.
2. The constructor stores `this.parentAgentId = config.parentAgentId ?? process.env.SWAYVE_PARENT_ID ?? null`.
3. `register()` (around line 56) adds `parent_agent_id: this.parentAgentId` to `params`. It sends `null` when unset.
4. Tests: start a `net.createServer` on a temp socket path and capture the first line. Assert `parent_agent_id` equals the configured value when set, and `null` when unset. Clean up the socket file afterwards.
5. README: add `SWAYVE_PARENT_ID` to the environment list (around line 30), with a one-line description (the spawning agent's ID, or unset for a root agent).

**Anchors.** `integrations/pi/src/swayve-client.ts`: `SwayveClientConfig` (~8), constructor (~18), `register()` (~56). `integrations/pi/README.md` env list (~30).

**Constraints.** Do not change `sendStatus`, `requestFocus`, or `withSwayve`. Keep the existing `_send` behavior. Do not add a dependency.

**Acceptance.**
- `cd integrations/pi && npm test` passes, with the new register tests.
- `npm run build` in `integrations/pi` succeeds.
- The register payload has `parent_agent_id` in both cases.

**Reference.** `integrations/pi/src/swayve-client.ts` (`register()`) is the template. The test pattern is the existing `swayve-client.test.ts`.

---

## Closing gate

After wave 3, run `make all` serially and confirm zero warnings. Then update `planning/GOALS.md`: move G59, G60, G61, G62 to Completed with their commit hashes.
