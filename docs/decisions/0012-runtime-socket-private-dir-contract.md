---
id: "0012"
title: "Runtime Socket Lives in a Private XDG Runtime Directory; SWAYVE_SOCKET Is an Explicit Override"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-08
authors:
  - "Andrew G. Brown"
supersedes: null
superseded_by: null
source_context: "Orchestration 19 (EXECUTION_PLAN_19.md D2, D4; commits cc3c7b3, 30f576e, 40c2777); src/cli.rs runtime_dir_from, ensure_private_dir; src/daemon.rs start_daemon_ipc"
---

# 12. Runtime Socket Lives in a Private XDG Runtime Directory; SWAYVE_SOCKET Is an Explicit Override

## Context and Problem Statement
The daemon's IPC socket is the only entry point for agents, hooks, and the HUD (see [0003](0003-daemon-owns-agent-state.md) and [0009](0009-thin-socket-integrations.md)). Anyone who can connect to it can change agent state and focus. A socket in a shared directory such as `/tmp` can be pre-created or replaced by another local user. The path also has to be stable enough for external clients, such as the Antigravity sidecar, to build against.

## Decision Drivers
* Only the owning user may reach the daemon socket.
* External clients need one documented default path with no guessing.
* Tests and power users need to point the daemon and clients at a custom socket.

## Considered Options
1. Default to `/tmp/swayve.sock` (the path used in early docs and the Antigravity README).
2. Default to `$XDG_RUNTIME_DIR/swayve/swayve.sock`, created with mode 0700 and checked for ownership, with no override.
3. Same as option 2, plus an explicit `SWAYVE_SOCKET` override that bypasses the directory check.

## Decision Outcome
Chosen option: option 3. The default is `$XDG_RUNTIME_DIR/swayve/swayve.sock`. The daemon refuses to start when `XDG_RUNTIME_DIR` is unset or empty. `ensure_private_dir` creates the directory with mode 0700, then requires the owner to be the current uid and no group or other bits to be set. Stale-socket removal in `start_daemon_ipc` deletes only a socket owned by the current uid and refuses any other path. `SWAYVE_SOCKET`, when set, is used as given and skips the directory check. The user chose that path explicitly, so the check is not applied to it.

### Consequences
* **Positive**: the default socket is private to the user. A pre-planted file or a foreign-owned socket is refused, not silently reused.
* **Positive**: external clients have one documented default. The override covers tests and unusual setups.
* **Negative**: the daemon will not start outside a systemd or login session that sets `XDG_RUNTIME_DIR`.
* **Negative**: `SWAYVE_SOCKET` skips the ownership check. A user who points it at a shared path takes on that risk. This is accepted because the path is explicit.
* **Negative**: the foreign-owner branches cannot be exercised with a real foreign owner, because tests run as the normal user and cannot create a root-owned path. They are covered by tests that inject the expected uid: `ensure_private_dir_for(p, uid)` and `prepare_socket_path(path, uid)`. The public functions keep their signatures and pass `current_uid()`. The same-owner stale-socket branch and the regular-file refusal are covered by tests.

## Validation & Compliance
* `src/cli.rs`: `test_ensure_private_dir_creates_0700`, `test_ensure_private_dir_rejects_loose_mode`.
* `src/daemon.rs`: `test_stale_socket_same_owner_is_replaced`, `test_stale_socket_removal_refuses_regular_file`.
* `README.md` "Socket Contract" section and `integrations/antigravity/README.md` state the same default and override.
