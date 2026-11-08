# Execution Plan 27: make all runs clippy with warnings denied

Source: `planning/GOALS.md` (G72, circled goal from the 2026-11-08 Magic Wand audit).
Recon base: `planning/STATE_26.md`, HEAD `00a4675`. Makefile at repo root.
Status: approved 2026-11-08, ready for wave-orchestration.

## Goal & Scope

**Goal.** The root gate fails on any clippy warning, so lint coverage is enforced by `make all` rather than by discipline.

**In scope.**
- `Makefile`: the `check` target runs `cargo clippy --all-targets -- -D warnings` after `cargo check`.
- `Makefile`: the `help` text for `check` mentions clippy.

**Out of scope.**
- The `test`, `build`, and `daemon` targets.
- Any Rust source change. Clippy is clean at baseline, so no code fix is expected.
- Pi (`integrations/pi`) lint or type-check gaps. That is G73.
- Clippy lint-level config (`#![deny]`, `clippy.toml`). `-D warnings` on the command line is the whole change.

**Verbatim constraints.**
- G72 exit criteria: "`make all` runs `cargo clippy --all-targets` and fails on any warning."

## Decision Log

- **D1 (2026-11-08): clippy goes in `check`, not a new `lint` target.** The user chose it over a separate target. `check` is already the compile-sanity step in `all`, so the diff stays one line plus help text.
- **D2 (2026-11-08): `-D warnings` is passed on the command line.** Clippy alone exits 0 on warnings. Denying them is what makes the gate fail.

## Hazard Notes (from recon)

- **H1.** `make all` = `check test build`. `check` is the only target that runs before `test`, so a clippy failure stops the gate early.
- **H2.** Clippy was run at baseline (STATE_26, `cargo clippy --all-targets`): exit 0, zero warnings. The new gate should pass as-is.
- **H3.** `make all` is serial. The gate must run serially; do not background it.

## Wave Table

| Wave | Task | Goal | Files (disjoint within wave) | Depends on |
|---|---|---|---|---|
| 1 | 27.1.1 | G72 | `Makefile` | — |

One task, one wave, one file.

## Wave Gate

- Serial `make all` exits 0 with zero warnings.
- `make -n check` shows `cargo clippy --all-targets -- -D warnings`.
- `planning/GOALS.md`: G72 moved to Completed with the commit hash.

---

## Task 27.1.1: `make check` runs clippy with warnings denied (G72)

**Repo:** swayve. **Wave:** 1.

**Files in scope.**
- must edit: `Makefile`
- can create: `.development/` scratch only

**Depends on:** none.

**Spec.**
1. In the `check` target, keep `cargo check` and add `cargo clippy --all-targets -- -D warnings` on the next line, with the same tab indentation.
2. In the `help` block, change the `check` line to say it runs `cargo check` and clippy with warnings denied. Keep the column alignment of the other lines.
3. Change nothing else in the Makefile.

**Anchors.** `Makefile`: `check:` target (lines 29–30, `\tcargo check`), help line `@echo "  check     : ..."` (line 11). Recipe lines use a hard tab.

**Constraints.** No change to `test`, `build`, `all`, or `daemon` targets. No new target. No Rust source change. Do not add a `clippy.toml`.

**Acceptance.**
- `make -n check` prints `cargo clippy --all-targets -- -D warnings`.
- `make check` exits 0.
- `make all` exits 0 with zero warnings (serial run; the wave gate).

---

## Closing gate

After wave 1, run `make all` serially. Confirm exit 0 and zero warnings. Then move G72 to Completed in `planning/GOALS.md` with the commit hash, and promote the next Independent Backlog item only if the user asks.
