# Execution Plan 16: Subscriber Drop-on-Full Is Recorded and Tested (Goal G5)

## Goal & Scope

**Goal**: Close G5 from `planning/GOALS.md`. When a HUD subscriber's channel is full, the daemon's behavior is recorded in ADR 0008 and covered by a test.

Current behavior (`src/daemon.rs`, `broadcast_status`, `try_send` near line 49): on a full channel, the newest snapshot is dropped, the subscriber is kept, and a line is logged (`Subscriber channel full; retaining subscriber`). On a closed channel, the subscriber is pruned. The channel is created with capacity 32 at `src/daemon.rs:233`. Each broadcast carries the full agent list, so the next broadcast resynchronizes a subscriber that lost a snapshot.

**In scope:**
- `src/daemon.rs`: one new unit test in the existing `mod tests`, next to `test_daemon_broadcast_status`.
- `docs/decisions/0008-push-status-subscription-for-hud.md`: amend in place to state the drop-on-full policy and its consequence. Status stays `Accepted`.

**Out of scope:**
- Changing the policy. Blocking the sender breaks ADR 0008's driver that subscribers must not block the event loop. Dropping the oldest snapshot needs a different channel type.
- Diff broadcast (G9, deferred; depends on G5).
- The writer task in `handle_client` (`src/daemon.rs:233` onward).
- Changing the channel capacity.
- Adding a new ADR. This is a clarification, not a reversal, so `docs/decisions/README.md` is unchanged.

**Verbatim constraints**: from `planning/GOALS.md` G5: "The subscriber channel's drop-on-full behavior is recorded and covered by a test (ADR 0008; `try_send` at `src/daemon.rs:49`)." Exit criteria: "test fills a capacity-32 subscriber channel, broadcasts once more, asserts the subscriber stays registered and receives later messages; `cargo test` passes with no new warnings."

## Decision Log

- **D1: Keep drop-newest-on-full; record it; do not change it** (2026-11-07, user chose recommended option a). Blocking violates ADR 0008's non-blocking driver. Drop-oldest needs a different channel type and is larger than G5.
- **D2: Single task, one wave, one worker, run serially** (2026-11-07, user chose recommended option a). The change is one test and one ADR edit in two files.
- **D3: Amend ADR 0008 in place; status stays Accepted** (2026-11-07, user chose recommended option a). The behavior is clarified, not reversed. The ADR's open "should be confirmed" line is closed.
- **D4: Mid-tier worker** (2026-11-07, default carried from plans 13–15).
- **D5: Acceptance bar**: targeted test per task; `make all` serially at wave end.

## Hazard Notes

- **Production drain rate**: the writer task in `handle_client` drains the channel, so a full channel is rare in practice. The test fills the channel by calling `broadcast_status` directly, without the writer, so it is deterministic.
- **Stderr**: the drop path logs with `eprintln!`. The test must not depend on the log text. Assert on channel state only.
- **Test placement**: `broadcast_status` is a `DaemonState` method, so the test goes in `src/daemon.rs` `mod tests`, not `tests/integration_tests.rs`.
- **Registration side effects**: `register_agent` with `Config::default()` and no parent does not spawn a monitor terminal and runs no hook. Keep the agent without a parent.
- **Pid collisions**: use a pid not used by other tests in the same module (the existing test uses 123; use 124).

## Wave Table

| Wave | Task ID | Description | Files in Scope | Depends On |
|:-----|:--------|:------------|:---------------|:-----------|
| **Wave 1** | **16.1.1** | Add drop-on-full test; amend ADR 0008 | `src/daemon.rs`, `docs/decisions/0008-push-status-subscription-for-hud.md` | None |

One task, so no disjointness question arises.

---

## Task Specifications

### Task 16.1.1: Drop-on-Full Test and ADR 0008 Amendment

**ID**: `16.1.1`
**Repo**: `swayve`
**Files-in-scope**:
- must edit: `src/daemon.rs` (test module only), `docs/decisions/0008-push-status-subscription-for-hud.md`
- can create: `planning/tasks/16.1.1.todo.md`
- scratch: `.development/` only

**Depends-on**: None

**Spec prose**:

1. In `src/daemon.rs` `mod tests`, add a `#[test]` named `test_daemon_broadcast_drops_newest_when_subscriber_full`. Model it on `test_daemon_broadcast_status` (grep anchor below).
   - Create `let (tx, mut rx) = mpsc::channel(32);` and push `tx` onto `state.subscribers`.
   - Register one agent with `register_agent(AgentId("full_sub_agent".to_string()), ProjectId("proj".to_string()), 124, "agent.full".to_string(), None)`.
   - Call `state.broadcast_status()` 32 times to fill the channel.
   - Call `state.broadcast_status()` once more (the 33rd snapshot is dropped).
   - Assert `state.subscribers.len() == 1`.
   - Drain `rx` with `try_recv` in a loop and assert exactly 32 messages were received.
   - Call `state.broadcast_status()` once more and assert `rx.try_recv()` returns `Ok` and the message contains `full_sub_agent`.
   - Assert `state.subscribers.len() == 1` again.
2. In `docs/decisions/0008-push-status-subscription-for-hud.md`, replace the third "Negative" consequence about slow subscribers (currently ends "...should be confirmed before relying on it.") with a statement of the policy:
   - When a subscriber's channel is full, the newest snapshot is dropped, the subscriber stays registered, and the daemon logs a line.
   - Because each snapshot is the full list, the next successful broadcast restores the subscriber's view. A slow subscriber can therefore show stale state until the next change.
   - A closed channel prunes the subscriber.
   - Cite `src/daemon.rs` `broadcast_status` and the test name.
   - Keep `status: "Accepted"` in the frontmatter. Do not change the Decision Outcome or the Considered Options.
3. Do not change `broadcast_status`, the channel capacity, or any non-test code in `src/daemon.rs`.

**Constraints (do not touch)**:
- Any non-test code in `src/daemon.rs`, including `handle_client` and the writer task.
- `docs/decisions/README.md` and every other ADR.
- `tests/integration_tests.rs`.
- Any file not listed under "must edit".

**Grep anchors**:
- `fn broadcast_status` (`src/daemon.rs`, around line 39): the function under test.
- `try_send` / `TrySendError::Full` (`src/daemon.rs`, around lines 49-53): the policy being tested.
- `mpsc::channel::<String>(32)` (`src/daemon.rs:233`): production capacity, reference only.
- `fn test_daemon_broadcast_status` (`src/daemon.rs`, around line 790): the reference test to model on.
- `should be confirmed before relying on it` (`docs/decisions/0008-push-status-subscription-for-hud.md`): the line to replace.

**Reference implementation**: `test_daemon_broadcast_status` in `src/daemon.rs` `mod tests`.

**Acceptance**:
- `cargo test --lib test_daemon_broadcast_drops_newest_when_subscriber_full` passes.
- `grep -c "should be confirmed before relying on it" docs/decisions/0008-push-status-subscription-for-hud.md` returns `0`.
- `grep -n 'status: "Accepted"' docs/decisions/0008-push-status-subscription-for-hud.md` returns one line.
- `cargo check` shows no new warnings.

**Targeted test**: `cargo test --lib test_daemon_broadcast_drops_newest_when_subscriber_full`

---

## Summary Table

| Task | Purpose | Risk | Effort |
|------|---------|------|--------|
| 16.1.1 | Drop-on-full policy tested; ADR 0008 amended | Low (test plus doc) | ~40 LOC test, ~6 lines doc |

## Wave 1 End Gate

- `make all` run serially (check, test, build).
- Expected: 25 unit tests (24 plus the new one) and 10 integration tests pass. Release build succeeds.

## Manual Verification (deferred)

- Live-Sway focus check from plan 13 remains outstanding.
- Malformed-config live check from plan 15 remains outstanding.
