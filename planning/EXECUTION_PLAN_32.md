# Execution Plan 32: open goals in series (G83, G73, G75)

Source: `planning/GOALS.md` open backlog (Independent Backlog: G83, G73, G75). Deferred goals (G66, G77, G78, G84) are out of scope: their triggers have not fired.
Recon base: HEAD `682085c`. Status: approved by the user 2026-11-08; executed (all three wave gates green).

## Goal & Scope

**Goal.** Clear the three open backlog goals one after another, each in its own wave, each gated before the next starts:
1. **G83** `cargo audit` reports no unsound or vulnerable crate in `Cargo.lock`.
2. **G73** `npm run build` type-checks `integrations/pi/tests/`.
3. **G75** The two planning design docs carry a header that marks them historical and points to the ADRs and the current protocol.

**In scope.**
- `Cargo.lock` tokio bump (G83).
- A typecheck config and a `build` script change in `integrations/pi/` (G73).
- A header block in the two design docs in `planning/` (G75).

**Out of scope.**
- Deferred goals G66, G77, G78, G84 (triggers not met).
- Blocked goals G15, G25, G53 (external).
- Any change to the tokio API or swayve code. G83 is a lockfile bump only (the swayve source makes no direct `AsyncFd` call; see Recon).
- Fixing Pi test type errors. None exist today (verified under `tsc --strict`).
- Publishing or pushing (no remote writes).

**Verbatim constraints.**
- G83 exit criterion (GOALS.md): "`cargo audit` reports no unsound or vulnerable crate in `Cargo.lock`."
- G73 exit criterion (GOALS.md): "`npm run build` (or a `typecheck` script) type-checks `integrations/pi/tests/`."
- G75 exit criterion (GOALS.md): "The planning design docs ... carry a header that marks them historical and points to the ADRs and the current protocol."
- Repo rule (CLAUDE.md): no remote write operations.

## Recon (facts)

- **Advisory.** RUSTSEC-2026-0386 (`~/.cargo/advisory-db/crates/tokio/RUSTSEC-2026-0386.md`): affected functions are the public `tokio::io::unix::AsyncFd` constructors only. Patched: `>= 1.51.6, < 1.52.0` and `>= 1.53.3`. `Cargo.lock` pins `tokio 1.53.1`, so the fix is a bump to `1.53.3` or later.
- **Direct use.** `grep -rn AsyncFd src tests` returns nothing. swayve reaches tokio through `UnixStream`/`UnixListener`, not the affected constructors. The bump is still the clean fix; "documented as unreachable" is the fallback.
- **Cargo.toml.** `tokio = { version = "1", features = ["full"] }`. A `cargo update -p tokio --precise 1.53.3` changes only `Cargo.lock`.
- **Pi package.** `integrations/pi/tsconfig.json` has `"include": ["src/**/*"]`, `rootDir: ./src`, `strict: true`. Scripts: `build: tsc`, `test: tsx --test tests/*.test.ts`. Dev dependency `tsc` is present (`node_modules` installed).
- **Pi tests today.** `tests/swayve-client.test.ts` type-checks clean under `tsc --strict` (exit 0, verified with the real exit code). G73 is therefore a wiring change: no existing errors to fix.
- **Design docs.** `planning/Sway Multi Agent Architecture Design.md` and `planning/Sway Multi-Agent Viewer Architecture.md` have no historical header today. Their content describes `sway-agentd` and a JSON-RPC 2.0 envelope. Superseded by ADR 0004 (newline-delimited JSON IPC), ADR 0003 (daemon owns agent state), ADR 0009 (thin socket integrations). The current socket contract is in `README.md` ("Socket Contract").
- **Baseline.** Last green gate: Orchestration 31 (`682085c`): `make all` serial exit 0, zero warnings; unit 88 passed / 2 ignored; integration 14.

## Decision Log

- **D1 (2026-11-08): series, one wave per goal.** The user asked for series. Each goal gets its own wave, gated before the next wave starts. Order: G83 first (it changes the lockfile that every later `make all` gate runs against, so the gates after it test the bumped tokio). G73 second (Pi only). G75 last (docs only, no gate impact).
- **D2 (2026-11-08): G83 bumps to `1.53.3`, the smallest patched version.** Recommended over `1.54.x`: it moves the least. Judgment call, flagged for approval.
- **D3 (2026-11-08): G73 uses a separate `tsconfig.test.json` and a `typecheck` script chained into `build`.** `tsconfig.json` has `rootDir: ./src`, so adding `tests/` to it would break `tsc` output. The separate config keeps `dist/` unchanged. Judgment call, flagged for approval.
- **D4 (2026-11-08): G75 header is a blockquote at the top of each doc, with links to ADR 0003, 0004, 0009 and the README Socket Contract.** Only added lines; no body text is changed. Judgment call on wording, flagged for approval.
- **D5 (2026-11-08): no direct use of `AsyncFd` in swayve is recorded as a fact, not as a fix.** The fix is the bump; the unreachability note is the fallback only if the bump is blocked.

## Hazard Notes

- **H1.** The Pi `build` script runs `tsc`; the new `typecheck` step must not write to `dist/` (use `--noEmit`).
- **H2.** `tsx --test tests/*.test.ts` is the Pi test entry. Do not change its glob.
- **H3.** The design docs are long. Edit only the first lines (grep `^# ` for the title line). No whole-file rewrite.
- **H4.** `make all` is serial. Never background it.
- **H5.** Do not run `cargo update` without `-p tokio --precise`; a bare `cargo update` moves every crate.

## Wave Table

| Wave | Task | Goal | Files | Depends on |
|---|---|---|---|---|
| 1 | 32.1.1 | G83 | `Cargo.lock` | — |
| 2 | 32.2.1 | G73 | `integrations/pi/package.json`, `integrations/pi/tsconfig.test.json` (create) | 32.1.1 (serial by D1) |
| 3 | 32.3.1 | G75 | `planning/Sway Multi Agent Architecture Design.md`, `planning/Sway Multi-Agent Viewer Architecture.md` | 32.2.1 (serial by D1) |

Each wave has one task, so there are no same-wave file collisions.

---

## Task 32.1.1: bump tokio past RUSTSEC-2026-0386 (G83)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `Cargo.lock`
- can create: none (scratch in `.development/` only)

**Depends on:** none.

**Spec.**
1. Red: `cargo audit` prints the tokio warning for RUSTSEC-2026-0386 at version `1.53.1`. Record the output (one line).
2. `cargo update -p tokio --precise 1.53.3`. Confirm the resolved version in `Cargo.lock` (`grep -A1 'name = "tokio"' Cargo.lock`).
3. Green: `cargo audit` prints no tokio warning. Record the output.
4. Gate: `make all` exit 0, zero warnings.

**Anchors.** `Cargo.lock` tokio entry (`name = "tokio"`). `Cargo.toml:7` requirement is `"1"` and needs no change.

**Reference implementation.** None needed; this is a lockfile change.

**Constraints.** Only `-p tokio --precise`. No `Cargo.toml` change. No other crate version may change: check with `git diff --stat Cargo.lock` (only the tokio block and any tokio-internal crates it pulls should change; report any other crate in the diff).

**Acceptance.**
- `cargo audit` shows no tokio warning.
- `make all` exit 0, zero warnings.
- `git diff Cargo.lock` changes the tokio version and nothing unrelated.

---

## Task 32.2.1: type-check the Pi tests in the build (G73)

**Repo:** swayve. **Wave:** 2.

**Files in scope.**
- must edit: `integrations/pi/package.json`
- can create: `integrations/pi/tsconfig.test.json`
- can create (scratch, red check only, deleted before the report): `integrations/pi/tests/_typecheck_probe.test.ts` (the probe must be deleted before the commit; the orchestrator checks)

**Depends on:** 32.1.1.

**Spec.**
1. Red (probe): create `tests/_typecheck_probe.test.ts` containing `const x: number = "not a number";`. Run `npx tsc -p tsconfig.test.json` once the config exists (step 2): it must fail. Before step 2, the config is absent, so the red is "config missing" (record it). Delete the probe after the red step.
2. Create `integrations/pi/tsconfig.test.json`: `{"extends": "./tsconfig.json", "compilerOptions": {"noEmit": true, "rootDir": "."}, "include": ["src/**/*", "tests/**/*"]}`.
3. `package.json` scripts: add `"typecheck": "tsc -p tsconfig.test.json"`, and change `"build"` from `"tsc"` to `"tsc && npm run typecheck"`. Keep `"test"` unchanged (H2).
4. Green: `npm run build` exit 0 (it writes `dist/` as today); `npm run typecheck` exit 0; `npm test` passes 8/8.
5. Confirm the probe is gone (`ls tests/`).
6. Gate: `make all` exit 0, zero warnings (the root gate runs `make test`, which runs `npm test`).

**Anchors.** `integrations/pi/tsconfig.json` (`include`, `rootDir`, `strict`). `integrations/pi/package.json` `"scripts"` block (`build`, `dev:run-agent`, `test`).

**Reference implementation.** `integrations/pi/tsconfig.json` (the base config the new file extends).

**Constraints.** Do not change `tsconfig.json`. Do not change the `test` script (H2). Do not add a dependency. Do not write any file under `tests/` except the scratch probe, which is deleted.

**Acceptance.**
- `npm run typecheck` exits 0 with the real tests included.
- The red probe fails `npm run typecheck` (recorded), and is deleted afterwards.
- `npm run build` exits 0; `npm test` 8/8.
- `make all` exit 0, zero warnings.

---

## Task 32.3.1: mark the two design docs historical (G75)

**Repo:** swayve. **Wave:** 3.

**Files in scope.**
- must edit: `planning/Sway Multi Agent Architecture Design.md`, `planning/Sway Multi-Agent Viewer Architecture.md`
- can create: none (scratch in `.development/` only)

**Depends on:** 32.2.1.

**Spec.**
1. Add a blockquote header at the top of each file (before the first line), identical in both except the title sentence:
   `> **Historical design document.** This describes an early design (`sway-agentd`, JSON-RPC 2.0). The shipped daemon is `swayve`. The current protocol is newline-delimited JSON (see [ADR 0004](../docs/decisions/0004-newline-delimited-json-ipc.md)); the daemon owns agent state ([ADR 0003](../docs/decisions/0003-daemon-owns-agent-state.md)); integrations are thin socket clients ([ADR 0009](../docs/decisions/0009-thin-socket-integrations.md)). The current socket contract is in the [README](../README.md#socket-contract).`
   Check the README anchor (`grep -n "^## .*Socket" README.md`) and fix the anchor text to match the real heading slug before writing.
2. Do not change any other line.
3. Verify: `git diff --stat` shows only added lines in the two files (`git diff --numstat` shows `N 0`).
4. Verify the relative links resolve: `ls ../docs/decisions/0003-daemon-owns-agent-state.md` etc. from `planning/`.
5. Gate: `make all` exit 0, zero warnings (docs-only; confirms nothing else moved).

**Anchors.** Line 1 of each file is the title (`# **Sway Multi ...`). Viewer doc line 1 is `# **Sway Multi-Agent Viewer Architecture: ...`.

**Reference implementation.** `planning/GOALS.md` header style (a status line at top).

**Constraints.** Only added lines. No edits to the body. No rename of either file.

**Acceptance.**
- Both files start with the blockquote header.
- `git diff --numstat -- planning/` shows `N 0` for both design docs.
- Each link target exists on disk.
- `make all` exit 0, zero warnings.

---

## Closing gate

After wave 3, confirm: `cargo audit` clean for tokio; `npm run build` green; both design docs marked. Then move G83, G73, G75 to Completed in `planning/GOALS.md` with their commit hashes. Backlog is then empty of open items; deferred and blocked goals remain as recorded.
