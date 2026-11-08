# Orchestration 29 State: one argv path for all three launches (G80 residual)

Plan: `planning/EXECUTION_PLAN_29.md` (approved by the user with `/wave-orchestration run plan 29`)
Last Updated: 2026-11-08

## Baseline (before Wave 1)
- Starting commit: `201f8ca` (main; planning commit on top of `2b6898f`).
- Pre-wave-1 green: Orchestration 28 closed at `b37b0e2`: unit 83 passed / 2 ignored; integration 12; Pi `npm test` 8/8; zero warnings. Working tree clean.

## Wave Board

### Wave 1: spawn_plan through expand_terminal_args (G80)
- [x] **Task 29.1.1**: `spawn_plan` builds argv through `expand_terminal_args`; D2 fallback kept (`bcbe4a0`)

### Wave 2: agent-launch argv-capture test (G80)
- [x] **Task 29.2.1**: `test_spawn_argv_capture` runs the real binary and asserts exact argv (`c2ce284`)

### Wave gates
- [x] After wave 2: serial `make all` exit 0, zero warnings. G80 exit criteria checked.

### Orchestrator steps
- [x] `git status` clean at each wave end (planning files excepted). Nothing pushed.

## Deviations
- Workers were not spawned. The orchestrator ran both tasks inline, because each is a single small edit plus its tests. The plan gate (micro-TODO approval) and the per-task commits were kept in the orchestrator's record only; no `planning/tasks/29.*.todo.md` files were written.
- Test-only correction in 29.2.1: `Config` must be a full serialized config (`toml::to_string(&Config::default())`). A partial TOML file fails to parse, and swayve silently falls back to `foot`. The spec's hand-written config was not valid.
- Clippy `collapsible_if` in the poll loop of 29.2.1 was fixed with `.filter` before the gate passed.

## Task Reports

### 29.1.1 (done)
- Status: done. `src/cli.rs` `spawn_plan` (~541) now calls `shlex::split(...).unwrap_or_else(|| vec![actual_command])` (D2), then `expand_terminal_args` (~562). Inline loop removed. `SpawnPlan`, env and id derivation unchanged.
- Red-first pins: `test_spawn_plan_shell_template_quoted_prompt_argv` and `test_spawn_plan_quoted_whitespace_arg_stays_one_entry` passed on the old code, as the spec predicted.
- Gate: `test_spawn_plan` 10 passed; `test_monitor_argv` 2 passed; `test_panel_argv` 1 passed; `make check` exit 0, zero warnings.

### 29.2.1 (done)
- Status: done. `tests/integration_tests.rs` `test_spawn_argv_capture` (~1040). Runs `env!("CARGO_BIN_EXE_swayve") spawn` with a full config (mock emulator, `--app-id {app_id} --title {title} -e {command}`), a private `XDG_RUNTIME_DIR` (0700), and a nonexistent `SWAYVE_SOCKET`.
- Captured argv: `--app-id agent.pi-w1-<digits> --title "Agent [pi]: w1" -e bash -c 'tool --path "/run/my dir/x" "it'"'"'s"'`, pinned with one exact `assert_eq!`.
- Gate: `cargo test --test integration_tests` 13 passed; closing `make all` serial exit 0, zero warnings. Unit 85 passed / 2 ignored; integration 13.
- Deviations: see the Deviations section.

### Closing gate
- `make all` serial exit 0, zero warnings. G80 exit criteria met: agent, monitor and panel launches all route through `expand_terminal_args`; argv-capture coverage for the agent launch with a whitespace path, a quoted prompt and a shell template.
- `planning/GOALS.md`: G80 moved to Completed (`bcbe4a0`, `c2ce284`). Circled Goal is empty. G74 is next.
- Discovered: none beyond the plan.
