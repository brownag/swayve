# Orchestration 34: Address PR #1 code review comments (Antigravity sidecar)

Status: approved by the user (2026-11-08).
Base commit: `773f529` (branch `feature/antigravity-sidecar1`).
Source: https://github.com/brownag/swayve/pull/1, CodeRabbit inline review comments (4508469234, 4508469235, 4508469236, 4508469237, 4508469244).

## Goal & Scope

**Goal.** Fix the five review findings on PR #1 that were verified against the current code, locally, in the four files that carry them. Done means the acceptance checks in Task 34.1.1 pass and one local commit exists on `feature/antigravity-sidecar1`.

**In scope.**
- `AGENTS.md` line 46: the test-layout sentence says the root gate does not run the Antigravity tests. `Makefile:37` does run them.
- `integrations/antigravity/README.md` line 21: the `SWAYVE_AGENT_PID` row says the launcher supplies it. `swayve spawn` does not (`src/cli.rs:564-570`).
- `integrations/antigravity/README.md` around line 108: the test command leaves the reader's shell inside `integrations/antigravity`, so the registration example below it breaks.
- `integrations/antigravity/sidecar.mjs` `send()`: no socket timeout, so a stalled daemon can hang the agent hook.
- `integrations/antigravity/sidecar.test.mjs` focus test: the promise never rejects, and the temp directory is not removed on failure. This is the Major comment, and the "trivial" summary item (same cleanup) is folded into it.

**Out of scope (deferred or refused, not silently dropped).**
- Replying to or resolving the review threads on GitHub. Refused: the user's global rule says "No remote write operations on GitHub or any git remote are allowed." The user resolves the threads by hand.
- The pre-merge "Docstring Coverage" warning. It is a CodeRabbit check, not a review comment.
- Any change to `src/`, the daemon, or the hook configuration (`.agents/hooks.json`, `~/.gemini/config/hooks.json`). Deferred as in orchestration 33.
- Any push or remote-write git command. Local commit only.

**Verbatim constraints.**
- User input: "https://github.com/brownag/swayve/pull/1 code review comments".
- User global rule: "No remote write operations on GitHub or any git remote are allowed."
- Wave-33 constraint still in force: "Do not create any `.js` file" (`.gitignore` ignores `*.js`). Use `.mjs`.
- Wave-33 constraint still in force: "Use Node.js with the built-in `node:test` runner and no dependencies."

## Decision Log

- **D1 (2026-11-08, user answered).** Socket timeout is a 2000 ms idle timeout. On expiry, `send()` logs one stderr line and calls `socket.destroy()`. Rationale: hooks stay fast, and a healthy local daemon answers well within 2 s. Alternatives rejected: 5 s idle (slower worst case for a hook), total deadline (more timer logic for little gain on a local Unix socket).
- **D2 (2026-11-08, user answered).** One task, one wave, one worker, one commit. Rationale: about 20 lines across four files, and the README and sidecar edits describe the same contract. Two parallel tasks would save little and add a second commit to review.
- **D3 (2026-11-08, planner default, not asked).** Add one test that proves the timeout fires: a server with `allowHalfOpen: true` never closes its side, and the test asserts the server-side connection closes within 4 s. Rationale: the review asked for the bounded wait in the test, but not for a test of the timeout itself. Without this test, D1 has no evidence. It is about 15 lines.
- **D4 (2026-11-08, planner default, not asked).** The focus test's bounded wait uses a 5 s timer that rejects, and `server.close()` plus `fs.rmSync` run in a `finally`. Rationale: the review's requested bound. 5 s is well above the 2 s timeout and keeps `make test` from hanging.
- **D5 (2026-11-08, planner default, not asked).** `AGENTS.md` line 46 keeps its other content. The only change is that the root gate runs the Antigravity tests through `make test`. Rationale: the review asked to preserve guidance for the other integrations.

## Recon log

- **R1.** `src/cli.rs:564-570` sets `SWAYVE_SOCKET`, `SWAYVE_AGENT_ID`, `SWAYVE_PROJECT_ID`, and (for subagents) `SWAYVE_PARENT_ID`. It does not set `SWAYVE_AGENT_PID`. README line 20 (`SWAYVE_PARENT_ID`, "Supplied by the launcher") is correct and stays as written.
- **R2.** `Makefile:37` runs `cd integrations/antigravity && node --test sidecar.test.mjs`. Make runs each recipe line in its own shell, so the Makefile is correct. Only the README's multi-line example is affected.
- **R3.** `sidecar.mjs` `send()` (around lines 103-111) has an `error` handler and no timeout. `socket.destroy()` emits no `error` event, so the timeout branch must log its own line.
- **R4.** The test helper `main()` swallows client errors into `logAndSkip`, so the test cannot observe a client failure directly. Only a bounded wait on the server side can catch it.
- **R5.** The sidecar calls `socket.end(payload)`. A server created with the default `allowHalfOpen: false` closes its side as soon as it sees the client's FIN. A stalled-socket test therefore needs `net.createServer({ allowHalfOpen: true }, ...)`, or the timeout never fires and the test proves nothing.
- **R6.** Wave-33 `STATE_33.md` recorded `node --test` at 10 pass in one place and 13/13 in another. The worker records the actual count from its own run. Do not hard-code a number.
- **R7.** Do not touch the Shell Injection section of the README (wave-33 H9).

## Wave table

| Wave | Task | Files (must edit) | Depends on | Parallel with |
| :--- | :--- | :--- | :--- | :--- |
| 1 | 34.1.1 | `AGENTS.md`, `integrations/antigravity/README.md`, `integrations/antigravity/sidecar.mjs`, `integrations/antigravity/sidecar.test.mjs` | none | none (single task) |

## Task 34.1.1: Address PR #1 review comments (sidecar timeout, test cleanup, docs)

- **Repo:** swayve (local, no remote writes)
- **Files in scope.**
  - Must edit: `AGENTS.md`, `integrations/antigravity/README.md`, `integrations/antigravity/sidecar.mjs`, `integrations/antigravity/sidecar.test.mjs`.
  - Can create: `planning/tasks/34.1.1.todo.md` (your micro-TODO), `.development/` scratch (gitignored).
  - Off-limits: every other file, including `planning/STATE_34.md`, `src/`, and the `Makefile`.
- **Depends on:** none.
- **Spec.**
  - **A. `AGENTS.md` line 46.** Replace the clause "the root gate does not run them" with a statement that `make test` runs the Antigravity tests through the root `Makefile`. Keep the rest of the sentence, including the guidance for other integrations.
  - **B. README line 21, `SWAYVE_AGENT_PID` row.** Remove "Supplied by the launcher." Say that `swayve spawn` does not set it, and that users set it when `process.ppid` is not the PID of the agent `agent.register` should report. Keep "Must be a positive integer", the `process.ppid` fallback, and the `exec` guidance for `hooks.json`.
  - **C. README line ~108, "Run and test" block.** Change `cd integrations/antigravity && node --test sidecar.test.mjs` to `(cd integrations/antigravity && node --test sidecar.test.mjs)`, so the shell's directory does not change. Leave the registration example's path (`node integrations/antigravity/sidecar.mjs register`) as is.
  - **D. `sidecar.mjs` `send()`.** After `net.createConnection(...)` is created, add `socket.setTimeout(2000, () => { logAndSkip("send timed out"); socket.destroy(); });`. Keep the `error` handler and the best-effort behavior unchanged.
  - **E. `sidecar.test.mjs` focus test (around lines 126-147).** Hoist the server into an outer `let`, and wrap the test in `try/finally`. Add a 5 s `setTimeout` that rejects with a clear message. In `finally`, call `server.close()` when it is open and `fs.rmSync(tmpDir, { recursive: true, force: true })`. Keep the existing byte assertion (`'{"method":"daemon.focus"}\n'`).
  - **F. `sidecar.test.mjs`, new test (D3).** Name it "send gives up on a stalled daemon". Create a server with `net.createServer({ allowHalfOpen: true }, ...)` that never ends its side, and record when its connection closes. Call `main(["focus"], { SWAYVE_SOCKET: sockPath }, 100)` after listen. Resolve when the server-side connection emits `close`. Reject after 4 s. Clean up the server and temp directory in `finally`.
- **Anchors (grep, do not read whole files).**
  - `grep -n "does not run them" AGENTS.md` (A)
  - `grep -n "Supplied by the launcher" integrations/antigravity/README.md` (B, expect line 20 to remain and line 21 to change)
  - `grep -n "cd integrations/antigravity" integrations/antigravity/README.md` (C)
  - `grep -n "function send" integrations/antigravity/sidecar.mjs` (D)
  - `grep -n "focus runs without SWAYVE_AGENT_ID" integrations/antigravity/sidecar.test.mjs` (E)
- **Reference implementation.** None needed. Follow the existing style of `sidecar.test.mjs` (`node:test`, `node:assert/strict`, `assert.equal`).
- **Constraints.**
  - Do not add `.js` files. No new dependencies.
  - Do not change the wire format or the messages.
  - Do not touch the README's Shell Injection section.
  - Do not read from the socket in the sidecar.
  - Do not run remote-write git commands. Do not commit; the orchestrator commits.
- **Hazards.**
  - R3: `destroy()` emits no `error`, so the timeout must log its own line.
  - R4: the test cannot observe client errors directly. The bounded wait is the only guard.
  - R5: F must use `allowHalfOpen: true`, or the stall never happens.
- **Acceptance (mechanical).**
  1. `cd integrations/antigravity && node --test sidecar.test.mjs` exits 0, with `fail 0`. Record the pass count.
  2. `cd integrations/antigravity && node --test sidecar.test.mjs 2>&1 | grep -ci warn` prints `0`.
  3. `grep -c "does not run them" AGENTS.md` prints `0`.
  4. `grep -n "Supplied by the launcher" integrations/antigravity/README.md` matches only the `SWAYVE_PARENT_ID` row (line 20).
  5. `grep -c "^cd integrations/antigravity" integrations/antigravity/README.md` prints `0`.
  6. `env -u SWAYVE_SOCKET -u XDG_RUNTIME_DIR node integrations/antigravity/sidecar.mjs focus; echo $?` prints one stderr line and `0`.
  7. `git diff --stat` lists only the four must-edit files.

## Wave gates (orchestrator, after Task 34.1.1)

- Targeted acceptance above, green.
- `make test` run serially from the repo root. This runs cargo, pi, antigravity, and hook tests. Must exit 0.
- One commit, conventional format, no wave or orchestration number in the message. Suggested: `fix(antigravity): address PR review on sidecar timeout, test cleanup, and docs`.
- Mark 34.1.1 done in `planning/STATE_34.md`. The user resolves the GitHub threads by hand.
