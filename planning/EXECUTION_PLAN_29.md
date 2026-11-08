# Execution Plan 29: one argv path for all three launches (G80 residual)

Source: `planning/GOALS.md` Circled Goal G80 (residual). Reopened after Orchestration 28 closed the monitor and panel paths.
Recon base: `planning/STATE_28.md`, HEAD `2b6898f`. Status: draft for approval.

## Goal & Scope

**Goal.** The agent launch (`spawn_plan`) builds its terminal argv through the shared helper `expand_terminal_args`, the same helper the monitor and panel launches use. An argv-capture integration test covers the agent launch with a whitespace path, a quoted prompt and a shell template.

**In scope.**
- `spawn_plan` (`src/cli.rs`) shlex-splits the user command, then calls `expand_terminal_args`.
- Unit tests pin the agent argv for a quoted prompt and a shell template.
- An integration argv-capture test for the agent launch, run through the real binary.

**Out of scope.**
- Monitor and panel launch changes (done in Orchestration 28).
- Config format changes to `terminal.args` (ADR 0010).
- Lifecycle hook argv (G74, sequenced after this goal).
- Any change to `register_and_wait` or the spawn registration path.

**Verbatim constraints.**
- G80 exit criterion: "an argv-capture test covers all three launches [agent, monitor, panel], including a whitespace path, a quoted prompt and an explicit shell template."
- Constraint (G80): "preserve ADR 0006 and 0010. Keep the existing spawn-plan extraction (G60, G64)."

## Decision Log

- **D1 (2026-11-08): bare-splice rule.** A bare `{command}` entry splices argv in. `{app_id}` and `{title}` substitute in other entries. `spawn_plan` already uses exact-match `{command}`, so this matches current behavior.
- **D2 (2026-11-08): shlex failure keeps the raw string.** If `shlex::split` fails, the raw command stays one argv entry, as today. No new error path.
- **D3 (2026-11-08): two serial waves, one task each.** Wave 1 touches `src/cli.rs`. Wave 2 touches `tests/integration_tests.rs`. Serial, following the Orchestration 28 precedent, so the integration test runs against the refactored binary.
- **D4 (2026-11-08): the integration test runs the real binary.** It uses `env!("CARGO_BIN_EXE_swayve")` with a temp `XDG_CONFIG_HOME` (config sets the mock terminal) and a private `XDG_RUNTIME_DIR`. A mock emulator writes one argv entry per line.

## Hazard Notes (from recon)

- **H1.** `src/cli.rs` is 993 lines and `tests/integration_tests.rs` is 1320. Use grep anchors and offset/limit reads, never whole-file reads.
- **H2.** `spawn_plan` is at `src/cli.rs:541`. Its inline loop and `shlex::split` are at `src/cli.rs:560-570`. The `Spawn` arm that calls it and launches the terminal is at `src/cli.rs:~225-300`.
- **H3.** `expand_terminal_args` and `panel_terminal_args` are at `src/cli.rs:~533-545`. Existing `spawn_plan` tests are in the `cli.rs` test module (`test_spawn_plan_*`, 8 tests). They must keep passing.
- **H4.** The spawn arm creates the log file under `swayve_runtime_dir()/logs` with `ensure_private_dir`. The test must set `XDG_RUNTIME_DIR` to a private temp dir, not `/tmp`.
- **H5.** Test paths use `tests/common::unique_path` (G45). No `/tmp` literal.
- **H6.** `make all` is serial. Never background it.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 29.1.1 | G80 | `src/cli.rs` | — |
| 2 | 29.2.1 | G80 | `tests/integration_tests.rs` | 29.1.1 |

Wave gates: serial `make all` exit 0, zero warnings after wave 2 (closing gate). Targeted checks after wave 1.

---

## Task 29.1.1: `spawn_plan` builds argv through `expand_terminal_args` (G80)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `src/cli.rs`
- can create: none (scratch in `.development/` only)

**Depends on:** none.

**Spec.**
1. Write failing unit tests in the `cli.rs` test module: (a) a quoted prompt with a shell template produces the expected argv (for example `["-e","bash","-c","pi \"it's\""]` with the existing `escapes_single_quote` input shape); (b) a command with a space inside a quoted argument stays one entry after the shlex split. These pin current behavior, so they should pass before the refactor (record this).
2. In `spawn_plan` (`src/cli.rs:541`), replace the inline `for arg in term_template` loop with: `let split = shlex::split(&actual_command).unwrap_or_else(|| vec![actual_command.clone()]);` then `expand_terminal_args(term_template, &app_id, &title, &split)`. This keeps the D2 fallback.
3. Remove the now-unused code. Keep `shlex` imported (still used).
4. Do not change `SpawnPlan`, `SpawnRequest`, the env vector, or the `agent_id`/`app_id`/`title` derivation.

**Anchors.** `spawn_plan` at `src/cli.rs:541`. `expand_terminal_args` at `src/cli.rs:~533`. Shlex loop at `src/cli.rs:560-570`. Existing tests: `test_spawn_plan_escapes_single_quote_in_prompt`, `test_spawn_plan_substitutes_app_id_and_title` (the 8 `test_spawn_plan_*` tests).

**Reference implementation.** `panel_terminal_args` (`src/cli.rs:~545`) as the pattern for calling the helper.

**Constraints.** Do not touch `terminal.args` config (ADR 0010). Keep `spawn_plan` extraction (G60, G64). Do not change the `Spawn` arm (only its helper callee). Preserve D2.

**Acceptance.**
- `cargo test --lib test_spawn_plan` passes (8 existing tests plus the new ones).
- `cargo test --lib test_monitor_argv` and `cargo test --lib test_panel_argv` still pass.
- `make check` exits 0 with zero warnings.

---

## Task 29.2.1: argv-capture integration test for the agent launch (G80)

**Repo:** swayve. **Wave:** 2.

**Files in scope.**
- must edit: `tests/integration_tests.rs`
- can create: none (scratch in `.development/` only)

**Depends on:** 29.1.1.

**Spec.**
1. Write `test_spawn_argv_capture` in `tests/integration_tests.rs`. Create a mock emulator script under `common::unique_path` that writes each argument on its own line (`printf '%s\n' "$@" > <flag>`), the same pattern as the monitor test at `tests/integration_tests.rs:898-903`.
2. Write a temp `XDG_CONFIG_HOME` with `config.toml` setting `[terminal] default_emulator` to the mock and `args = ["--app-id", "{app_id}", "--title", "{title}", "-e", "{command}"]`. Set `XDG_RUNTIME_DIR` to a private temp dir (mode 0700).
3. Run `env!("CARGO_BIN_EXE_swayve")` with `spawn --wave w1 --agent pi --project p --command "<shell template>" --prompt "it's"`, using `std::process::Command` with the env vars set. Use a whitespace path in the command argument (for example `--command 'tool --path "/run/my dir/x"'`). Poll for the flag file (same pattern as the monitor test).
4. Assert the captured argv exactly. The suffix is clock-based, so read the `--app-id` line from the capture, assert it matches `agent.pi-w1-<digits>`, then build the expected array with that captured `app_id` and the title `Agent [pi]: w1`. Compare the full array with `assert_eq!`. The expected array is `["--app-id", <app_id>, "--title", "Agent [pi]: w1", "-e", "bash", "-c", <script>]` for the prompt case, where the script and the whitespace path come from the test inputs.
5. Keep the existing monitor test and all other integration tests unchanged.

**Anchors.** Monitor mock and assertion pattern: `tests/integration_tests.rs:~898-903` and `~986-1003`. `common::unique_path`: `tests/common/mod.rs:5`. Spawn args: `Commands::Spawn` in `src/cli.rs` (fields `wave`, `agent`, `project`, `command`, `prompt`, `parent`).

**Reference implementation.** `test_hooks_and_monitoring` (`tests/integration_tests.rs:871`) for the mock-emulator and poll pattern.

**Constraints.** Do not add `/tmp` literals (G45). Do not change `tests/common` beyond what the test needs. Do not change the existing monitor assertions. The test must not require a running Sway or daemon: the spawn arm warns and launches when the socket is absent.

**Acceptance.**
- `cargo test --test integration_tests test_spawn_argv_capture` passes.
- `cargo test --test integration_tests` passes with 13 tests (12 existing plus this one).
- `make all` serial exit 0 with zero warnings (closing gate).

---

## Closing gate

After wave 2, run serial `make all`. Confirm exit 0 and zero warnings. Then move G80 to Completed in `planning/GOALS.md` with its commit hashes. G74 becomes the next Sequenced goal.
