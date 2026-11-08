# Agent prompt: external HUD consumes the diff subscription (G53)

You are creating the HUD repository, which is outside the swayve repo. Its location is `~/workspace/swayve-hud`. It does not exist yet. Do not edit anything in the swayve repo. Do not add a git remote, push, or run any remote-write git command. Local `git init` and commits are allowed.

## Goal
The HUD shows the live agent tree from Swayve, using `daemon.subscribe_diff` and not the full-list stream. Done means the HUD matches the daemon's state after a daemon restart, and every item in "Acceptance" passes.

## Socket contract (swayve `README.md` Socket Contract, `docs/decisions/0008`, `0013`)
- Socket path: `$SWAYVE_SOCKET` if set; otherwise `$XDG_RUNTIME_DIR/swayve/swayve.sock`.
- Send one line: `{"method":"daemon.subscribe_diff"}\n`. Read newline-delimited JSON.
- Each line is `{"snapshot":bool,"upsert":[AgentWindow...],"remove":["agent_id",...]}`. The first line is always a snapshot.
- Fold each line into a map keyed by the agent's `id`:
  - `snapshot: true` clears the map first.
  - Each `upsert` entry replaces or inserts by `id`.
  - Each `remove` id deletes the entry.
- A subscriber that falls behind gets a snapshot on its next line. The line is sent on the next state change, not as soon as the writer catches up (`src/daemon.rs:128-131`, `94-95`). Handle it the same way as the first line. Do not special-case it.
- Skip any line that is not JSON. Change nothing and do not re-render.
- Do not use `daemon.subscribe` (the full-list stream) for the HUD.

Reference implementation: `fold_diff_line` in swayve `src/cli.rs`, with its unit tests. Port the same behavior. Do not copy its code without reading it.

## Scaffold (no HUD exists yet)
Stack: Rust, edition 2024, matching swayve. Dependencies: `serde_json` only. No async runtime needed: a blocking `std::os::unix::net::UnixStream` is enough for one subscriber.
1. `mkdir -p ~/workspace/swayve-hud && cd ~/workspace/swayve-hud && git init`. No remote.
2. `cargo init --bin --name swayve-hud`. Add `serde_json` with `cargo add serde_json`.
3. Create `src/main.rs` (connect, subscribe, print the folded list per change) and `src/fold.rs` (the fold function, with its unit test in `#[cfg(test)] mod tests` at the bottom).
4. Create a `Makefile` with `check`, `test` (`cargo test`), and `build` targets, to match swayve's layout.
5. Create a short `README.md`: what it does, the socket path rule, and how to run it.
6. Commit with `git add -A && git commit -m "chore: scaffold swayve-hud"`.

## Work
1. Connect to the socket path rule above. Send `{"method":"daemon.subscribe_diff"}\n`. Do not use `daemon.subscribe`.
2. Fold each line with the fold above. Skip non-JSON lines and do not re-render on them.
3. On EOF or a read error, reconnect with a bounded backoff (for example 0.5s doubling to 5s, then capped). Do not spin. `swayve status --watch` does not reconnect (`src/cli.rs:619-668` returns on EOF), so the HUD owns this loop.
4. Add one test that feeds a snapshot line, an upsert line and a remove line through the fold, and asserts the final list. Use the smallest runnable check, not a framework.
5. Run `make all` (or `cargo test` if no Makefile target is wired yet). Report the exact command and the pass count.

## Acceptance
- The HUD renders the same agents as the daemon after a fresh snapshot. Compare against `swayve status --watch --diff` (or the daemon's full list). Do not compare against `swayve status --json`: it prints a static waybar summary (`src/cli.rs:364-371`), not the agent list.
- After `swayve` restarts, the HUD recovers without a manual restart.
- A non-JSON line leaves the displayed list unchanged.
- `make all` (or `cargo test`) passes in `~/workspace/swayve-hud`, with no new warnings.

## Report (15 lines or fewer)
- Status: done or blocked.
- Files changed, with line ranges.
- Test command and results.
- Files created under `~/workspace/swayve-hud`, and the commit hash.
- Anything you could not verify without a live daemon.

## Supporting implementation facts (from research)

Facts below are the ones this HUD needs. Citations are to the swayve repo at `bee60cd`.

### Wire format
Subscribe request: `{"method":"daemon.subscribe_diff"}` (`src/types.rs:67-68`).

Each frame:
| Field | Type | Required | Source |
|---|---|---|---|
| `snapshot` | bool | yes (a missing key rejects the whole line) | `src/types.rs:117-122` |
| `upsert` | array of AgentWindow | yes | `src/types.rs:120` |
| `remove` | array of agent id strings | yes | `src/types.rs:121` |

AgentWindow fields: `id` (string, yes), `project` (string, yes), `parent_id` (string or null, optional), `container_id` (i64 or null, optional), `app_id` (string, yes), `mark` (string, yes), `status` (AgentStatus, yes). Source: `src/types.rs:32-38`. Extra keys are ignored.

Key order in the golden bytes is `remove, snapshot, upsert`, and AgentWindow keys are alphabetical (`tests/diff_golden.rs:104-105`). Parsers must not depend on order.

A line is never a bare array. The full-list stream (`daemon.subscribe`) sends one JSON array per line (`tests/subscribe_golden.rs:96-97`). Do not use it.

### Fold
Reference: `DiffFrame::fold_into` (`src/types.rs:133-143`), and `fold_diff_line` (`src/cli.rs:488-494`).
- `snapshot: true` clears the map first.
- Each `upsert` replaces or inserts by `id`.
- Each `remove` id deletes its entry.
- A line that is not JSON, or fails the `DiffFrame` parse, returns `false` and changes nothing (`src/cli.rs:489-491`). The caller does not re-render.

Gotcha: a line that is valid JSON but fails `DiffFrame` (for example one AgentWindow with a bad `status`) is dropped whole, so its valid upserts and removes are lost. The daemon does not resync on parse failure. Treat a parse failure as "stale": wait for the next snapshot, or reconnect. The daemon only produces well-formed frames, so this matters only on a version mismatch.

### Snapshots and lag
- The first line is always a snapshot. Subscribe sets `needs_snapshot` (`src/daemon.rs:49-53`) and sends one right away (`src/daemon.rs:612-615`, `94-95`).
- The snapshot carries every agent in `upsert`, with `snapshot: true` and `remove: []` (`src/daemon.rs:94-95`, `110-114`). Clear-then-upsert handles it.
- A subscriber that falls behind has its line dropped, and `needs_snapshot` is set (`src/daemon.rs:128-131`). The snapshot is sent on the next broadcast, which means the next state change, not when the writer drains. A quiet agent set may not see it until something changes (ADR 0013).
- Per-subscriber channel capacity is 32 (`src/daemon.rs`, `handle_client`; ADR 0008).
- Lines with nothing new are not sent (`src/daemon.rs:107-109`).

### Reconnect and restart
- swayve has no client-side reconnect. `run_status_watch` returns on EOF (`src/cli.rs:619-668`). The HUD must supply a bounded backoff.
- After a daemon restart, `DaemonState::new` (`src/daemon.rs:511`) starts with an empty map. Nothing in `src/` persists agents. A reconnect gets a snapshot of an empty map until agents re-register. "Matches the daemon after restart" is therefore true, but the list may be empty. Whether the daemon rebuilds from the Sway tree at startup is not verified.

### Corrections that apply to this prompt
1. Acceptance item 1 was wrong and is fixed above. `swayve status --json` prints a static waybar summary (`src/cli.rs:364-371`), not the agent list. Compare against `swayve status --watch --diff` (fold at `src/cli.rs:488-494`) or the HUD's own fold. The non-watch `swayve status` lists Sway-tree windows (`src/cli.rs:383-440`), not daemon state.
2. The socket path rule is confirmed (`src/cli.rs:91-98`). An empty `SWAYVE_SOCKET` is accepted by code as a path, so treat `""` as unset.
3. "Do not use `daemon.subscribe`" is confirmed (`src/daemon.rs:45-47`).

### Test pins to mirror
- `cli::tests::test_fold_diff_snapshot_clears_then_upserts` (`src/cli.rs:863`), `test_fold_diff_upsert_then_remove` (`src/cli.rs:873`), `test_fold_diff_skips_non_json` (`src/cli.rs:884`). One HUD test should cover snapshot, then upsert, then remove, in one sequence.
- `tests/diff_golden.rs` `test_subscribe_diff_bytes_are_pinned` (line 29), constants at lines 104-105.
- `daemon::tests::test_diff_subscriber_gets_snapshot_then_diffs` (`src/daemon.rs:1301`), `test_diff_subscriber_recovers_with_snapshot_after_drop` (`src/daemon.rs:1334`).
- `tests/slow_consumer.rs` `test_stalled_diff_subscriber_recovers_with_snapshot` (line 41).
- `tests/integration_tests.rs` `test_diff_stream_matches_daemon_state` (line 1289).

### Unverified
- Whether the daemon rebuilds agents from Sway at startup. Read `run_daemon_loops`, or run `make daemon-dry` and check the first snapshot after restart.
- Whether a snapshot after lag arrives with no state change. Only the state-change case is tested (`tests/slow_consumer.rs`).
- Acceptance "after swayve restarts, the HUD recovers": depends on the HUD's backoff. Test live with `make daemon-dry` plus a restart.
