# Execution Plan: swayve

## Goal & Scope
* **Goal**: Build `swayve`, a native Wayland/Sway window-management and agent-coordination ecosystem. It bridges running AI agents with Sway's compositor using an asynchronous daemon and a CLI tool to automate workspace creation, layouts, window grouping, focus triggers, and state coloring.
* **In Scope**:
  * The background daemon subcommand (`swayve daemon`) listening on `/tmp/swayve.sock` and managing the Sway IPC subscription (`swayipc-async`).
  * The controller CLI `swayve` supporting commands:
    * `spawn --wave <name> --agent <type> --project <project>`: Launches an agent wrapped in a terminal (like `foot`).
    * `panel toggle`: Triggers the floating HUD/Orchestrator panel.
    * `status`: Checks health and active agent statuses.
  * System config directory `~/.config/swayve/config.toml` (with default fallback configs).
  * Window criteria/layout rules integration matching the architecture design.
* **Out of Scope (Deferred)**:
  * Full Waybar status bar widget styling (we will provide the `swayve status` JSON output format, but configuring users' bars is deferred).
  * Arbitrary non-tiling window manager support (strict focus on Sway/i3).

## Decision Log
* **2026-07-26**: Decided on a single binary architecture (`swayve`) with subcommands to share code, simplify deployment, and keep compilation fast.
* **2026-07-26**: Default to `foot` as the terminal emulator for spawning agents, customizable via `~/.config/swayve/config.toml` or CLI.
* **2026-07-26**: Border colors for agent status states will be defined in `~/.config/swayve/config.toml`, defaulting to Dracula theme values.
* **2026-07-26**: Implement a Dry-run/Mock mode (`--dry-run` or `SWAYVE_DRY_RUN=1`) that logs Sway IPC commands instead of executing them to facilitate offline testing.
* **2026-07-26**: `swayve panel toggle` will query the Sway window tree (using Sway IPC or the daemon) to toggle scratchpad visibility of the orchestrator window or spawn it if not present.

## Wave & Task DAG

| Wave | Task ID | Description | Files in Scope | Depends On |
| :--- | :--- | :--- | :--- | :--- |
| **Wave 1** | **T1_SCAFFOLD** | Initialize Cargo project, define dependencies, config parsing, and shared protocol types. | `Cargo.toml`, `src/config.rs`, `src/types.rs`, `src/main.rs` | None |
| **Wave 2** | **T2_DAEMON_IPC** | Implement Unix domain socket listener and JSON-RPC protocol server/state machine. | `src/daemon.rs` | T1_SCAFFOLD |
| **Wave 3** | **T3_SWAY_INTEGRATION** | Implement Sway IPC connector (event subscriber, command executor) with Dry-run support. | `src/sway.rs` | T2_DAEMON_IPC |
| **Wave 4** | **T4_CLI_CONTROLLER** | Implement CLI commands (`spawn`, `panel toggle`, `status`) and tie them into `src/main.rs`. | `src/cli.rs`, `src/main.rs` | T3_SWAY_INTEGRATION |
| **Wave 5** | **T5_INTEGRATION_TESTS** | Author integration tests running the daemon in Dry-run mode and verifying state workflows. | `tests/integration_tests.rs` | T4_CLI_CONTROLLER |

## Reconnaissance Findings & Hazards
* **Workspace State**: Currently a greenfield project. Only architectural design documents are present.
* **Compiler**: Rustc/Cargo version 1.91.1 is ready.
* **Hazards**: Since we will run tests in headless/non-Sway environments (like normal CI or development containers), Dry-run mode is critical to prevent testing failures due to lack of `SWAYSOCK`.

---

## Task Specifications

### T1_SCAFFOLD
* **ID**: `T1_SCAFFOLD`
* **Repo**: `swayve`
* **Files-in-scope**: `Cargo.toml`, `src/config.rs`, `src/types.rs`, `src/main.rs`
* **Depends-on**: None
* **Spec prose**: 
  Initialize the cargo binary project (`cargo init`). Add dependency entries for `tokio` (full), `serde` (derive), `serde_json`, `swayipc-async`, `clap` (derive), and `toml`.
  Implement `src/types.rs` containing `AgentId`, `ProjectId`, `AgentStatus`, `AgentWindow`, and the JSON-RPC models.
  Implement `src/config.rs` to read from `~/.config/swayve/config.toml` (and fallback to default Dracula border colors: Idle `#44475a`, Thinking `#ffb86c`, ExecutingTool `#8be9fd`, AwaitingHumanInput `#ff5555`, Completed `#50fa7b`, Failed `#ff79c6`).
* **Constraints**: None.
* **Grep Anchors**: `struct AgentStatus`, `fn load_config`
* **Acceptance + Targeted test command**: `cargo check && cargo build`

### T2_DAEMON_IPC
* **ID**: `T2_DAEMON_IPC`
* **Repo**: `swayve`
* **Files-in-scope**: `src/daemon.rs`
* **Depends-on**: `T1_SCAFFOLD`
* **Spec prose**:
  Implement the Unix domain socket server (`/tmp/swayve.sock`) and JSON-RPC message processing loop.
  The daemon state machine should support:
  1. `agent.register` request: registers the agent with its ID, PID, and workspace routing.
  2. `agent.update_status` notification: updates the status and notifies any visual managers.
  3. `agent.heartbeat` notification.
* **Constraints**: Do not import Sway IPC client code yet; focus on client-to-daemon state machine and socket loop.
* **Grep Anchors**: `struct DaemonState`, `fn handle_client`
* **Acceptance + Targeted test command**: `cargo check`

### T3_SWAY_INTEGRATION
* **ID**: `T3_SWAY_INTEGRATION`
* **Repo**: `swayve`
* **Files-in-scope**: `src/sway.rs`, `src/daemon.rs`
* **Depends-on**: `T2_DAEMON_IPC`
* **Spec prose**:
  Implement the Sway IPC connection task using `swayipc-async`.
  Listen to container creation and destruction events.
  Execute command buffers (`focus`, `mark`, `border`, `move container to workspace`).
  Integrate the dry-run mock mode: if `dry_run` is enabled (either via config, CLI arg, or `SWAYVE_DRY_RUN=1`), log all commands to standard output instead of attempting to connect to `SWAYSOCK`.
* **Constraints**: Maintain clean error handling if `SWAYSOCK` is missing and dry-run is disabled.
* **Grep Anchors**: `fn connect_sway`, `fn execute_sway_command`
* **Acceptance + Targeted test command**: `cargo check`

### T4_CLI_CONTROLLER
* **ID**: `T4_CLI_CONTROLLER`
* **Repo**: `swayve`
* **Files-in-scope**: `src/cli.rs`, `src/main.rs`
* **Depends-on**: `T3_SWAY_INTEGRATION`
* **Spec prose**:
  Implement the CLI arguments parsing using `clap`. Subcommands:
  - `daemon [--dry-run]`: Runs the socket and Sway listener loops.
  - `spawn --wave <name> --agent <type> --project <project>`: Launches `foot` (or custom terminal) with appropriate App ID.
  - `panel toggle`: Toggles floating panel.
  - `status`: Reads agent states from daemon or outputs JSON for Waybar.
  Tie the CLI execution into `src/main.rs`.
* **Constraints**: Command names and flags must match the spec exactly.
* **Grep Anchors**: `enum Commands`, `fn run_cli`
* **Acceptance + Targeted test command**: `cargo build`

### T5_INTEGRATION_TESTS
* **ID**: `T5_INTEGRATION_TESTS`
* **Repo**: `swayve`
* **Files-in-scope**: `tests/integration_tests.rs`
* **Depends-on**: `T4_CLI_CONTROLLER`
* **Spec prose**:
  Write an integration test suite. Run the `swayve daemon --dry-run` in a background thread or child process.
  Connect to `/tmp/swayve.sock` and send registration and status update payloads.
  Verify the daemon registers the agent correctly and logs the correct Sway commands.
* **Constraints**: Ensure `/tmp/swayve.sock` is deleted before and after testing.
* **Grep Anchors**: `fn test_daemon_registration`
* **Acceptance + Targeted test command**: `cargo test`
