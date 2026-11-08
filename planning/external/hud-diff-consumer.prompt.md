# Agent prompt: external HUD consumes the diff subscription (G53)

You are working in the HUD repository, which is outside the swayve repo. Its location is `{{HUD_REPO}}`. Do not edit anything in the swayve repo. Do not push or run any remote-write git command.

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
- A subscriber that falls behind gets a snapshot on its next line. Handle that the same way as the first line. Do not special-case it.
- Skip any line that is not JSON. Change nothing and do not re-render.
- Do not use `daemon.subscribe` (the full-list stream) for the HUD.

Reference implementation: `fold_diff_line` in swayve `src/cli.rs`, with its unit tests. Port the same behavior. Do not copy its code without reading it.

## Work
1. Read the HUD's current daemon client. Record whether it uses the full-list stream, and how it reconnects.
2. Switch it to `daemon.subscribe_diff` and the fold above.
3. On EOF or a read error, reconnect with a bounded backoff. Do not spin.
4. Add one test that feeds a snapshot line, an upsert line and a remove line through the fold, and asserts the final list. Use the smallest runnable check, not a framework.
5. Run the HUD's own test command. Report the exact command and the pass count.

## Acceptance
- The HUD renders the same agents as `swayve status --json` after a fresh snapshot.
- After `swayve` restarts, the HUD recovers without a manual restart.
- A non-JSON line leaves the displayed list unchanged.
- The HUD's test command passes.

## Report (15 lines or fewer)
- Status: done or blocked.
- Files changed, with line ranges.
- Test command and results.
- Whether the HUD used the full-list stream before the change, and how it reconnects.
- Anything you could not verify without a live daemon.
