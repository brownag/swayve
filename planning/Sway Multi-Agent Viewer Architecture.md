# **Sway Multi-Agent Viewer Architecture**

[https://gemini.google.com/app/25023afdb109c778](https://gemini.google.com/app/25023afdb109c778)

*User prompt: refining an idea, a sway-compatible multi-agent viewer for tools like Pi coding agent, claude code etc. basically it transparently handles making multi-agents first class windows in sway, projects in workspace(s), orchestrator panels, custom plugins*

Response: This concept leverages **Wayland/Sway’s native Unix domain socket IPC** rather than fighting against it.

Existing AI terminal interfaces force everything into a single terminal window or a custom WebUI grid. Moving orchestration into Sway's IPC tree gives you keyboard-driven tiled layouts, workspace switching, floating scratchpads, and Wayland IPC events out of the box.

Here is a system breakdown to turn this concept into an executable architecture.

## **🏛 Architecture Overview**

                        `+----------------------------+`  
                        `|   Sway IPC Domain Socket   |`  
                        `+--------------+-------------+`  
                                       `|`  
                                       `v`  
               `+----------------------------------------------+`  
               `|     Sway-Agent Orchestrator (Daemon)        |`  
               `+-------+----------------------+---------------+`  
                       `|                      |`  
            `+----------v----------+    +------v-----------------------+`  
            `|  Subprocess Engine  |    |  Sway Criteria Rules Engine  |`  
            ``|  (Pi, Claude, etc.) |    |  (`app_id`, `marks`, title)  |``  
            `+----------+----------+    +------------------------------+`  
                       `|`  
                       `v`  
         `+----------------------------------+`  
         `| Terminal Wrappers / Agent Windows |`  
         `|  (e.g., foot --app-id=sway-agent) |`  
         `+----------------------------------+`

## **🛠 Core Components & Implementation Strategy**

### **1\. Window Identity & Dynamic Window Rules**

To make Sway treat individual agents as first-class window objects, launch each agent in an isolated terminal process (e.g., foot, kitty, or alacritty) with distinct metadata:

> * **App ID Strategy:** Pass unique launch flags:  
>   `foot --app-id="agent.pi.projA" --title="Agent: Refactor Auth" -e pi`

> * **Sway Marks:** Mark windows programmatically via IPC (swaymsg mark "agent\_worker\_1") so your orchestrator can target them for focus, resizing, or repositioning regardless of workspace.  
> * **Auto-Layout Matching:** Put Sway criteria rules in your config or emit them dynamically:  
>   `# Split agent workspaces into tabbed containers by default`  
>   `for_window [app_id="agent\..*"] layout tabbed`  
>   `# Float orchestrator panels`  
>   `for_window [app_id="agent.orchestrator"] floating enable, resize set 800 600`

### **2\. Workspace & Container Hierarchy (Project Isolation)**

Instead of forcing every agent onto one crowded screen, map project contexts directly to Sway’s container tree:

> * **Workspaces as Projects:**  
  * Workspace 2:proj-auth contains all agents working on the authentication microservice.  
  * Workspace 3:proj-db contains database migration agents.  
> * **Layout Patterns:**  
  * **Leader/Follower:** The main orchestrator window takes the left 40% split. Sub-agents stack vertically or tab on the right.  
  * **Scratchpad Agents:** Background worker agents can be placed into Sway’s **scratchpad** and toggled into view with a hotkey when they request human intervention or approval.

### **3\. Orchestrator Panel & IPC Event Listener**

The orchestrator acts as a coordinator between agent processes and Sway’s Wayland compositor:

`import asyncio`  
`import json`  
`from i3ipc.aio import Connection`

`async def on_window_event(self, ipc, e):`  
    `# Detect when an agent terminal opens or closes`  
    `if e.change == "new" and "agent" in e.container.app_id:`  
        `print(f"Agent Registered: {e.container.app_id}")`  
        `# Automatically mark and position it in the container tree`  
        `await ipc.command(f'[con_id="{e.container.id}"] mark "active_agent"')`

`async def main():`  
    `ipc = await Connection().connect()`  
    `ipc.on("window::new", on_window_event)`  
    `await ipc.main()`

`asyncio.run(main())`

### **4\. Inter-Agent Communication & Plugin System**

Make tools like **Pi**, **Claude Code**, or **Aider** communicate via an event bus:

> * **Unix Sockets & Named Pipes:** Expose a local socket (/tmp/sway-agent.sock) where agents send visual signals.  
> * **Visual Status Indicators (Sway Borders/Bars):**  
  * When an agent needs input: The orchestrator triggers swaymsg \[app\_id="agent..."\] border normal 3px with a highlighted color.  
  * Integration with waybar or i3status to display overall system status (e.g., \[Pi: Thinking | Claude: Waiting Input\]).  
> * **Custom Plugin Hook Architecture:** Allow plugins to register custom keybindings in Sway:  
>   `# Quick-focus the agent requiring attention`  
>   `bindsym $mod+Shift+a exec swaymsg '[con_mark="needs_attention"] focus'`

## **📋 Architectural Matrix**

| Feature | Standard Terminal Grid (tmux/zellij) | Sway-Compatible Agent Viewer |
| :---- | :---- | :---- |
| **Window Control** | Virtual panes inside 1 terminal | Native Wayland windows |
| **Keyboard Driven** | Custom terminal hotkeys | Native Sway keybindings ($mod+hjkl) |
| **Multi-Monitor** | Painful nested multiplexing | Multi-monitor workspace management |
| **Process Isolation** | Single-terminal crash risks | Process-level isolation |
| **State Notifications** | Custom status lines | Native Wayland borders, urgency hints, Waybar |

## **🚀 Suggested Technical Stack**

> * **IPC Integration:** i3ipc-python or async-stripe/tokio with swayipc-rs (Rust recommended for speed and async IPC streaming).  
> * **Terminal Shell:** foot or kitty (both offer low latency, minimal footprint, and dynamic IPC app-id customization).  
> * **Notification Layer:** mako or dunst triggered via Sway IPC when an agent completes a task or requests human confirmation.

**Sources**

> 1. [Sway \- ArchWiki](https://wiki.archlinux.org/title/Sway)

---

*User prompt: Design a detailed Rust IPC daemon architecture for managing Sway agent windows and event loops.*

Response: Here is a complete, production-ready architecture design for a **Sway Agent Daemon (sway-agentd)** built in Rust.

This architecture uses tokio for async orchestration and swayipc-async to handle low-latency events to and from the Sway compositor. It bridges external AI tools (Pi, Claude Code) with native Wayland window states.

## **🏗 Architecture Blueprint**

                     `+---------------------------------------+`  
                     `|    Sway Compositor Domain Socket      |`  
                     `+-------------------+-------------------+`  
                                         `|`  
                                         `v`  
   `+---------------------------------------------------------------------------+`  
   ``|                       SWAY AGENT DAEMON (`sway-agentd`)                   |``  
   `|                                                                           |`  
   `|  +---------------------+   (mpsc)   +----------------------------------+  |`  
   `|  | Sway Listener Loop  | ---------> |        Core Engine Task          |  |`  
   ``|  |  (`swayipc-async`)  |            | - State Machine                  |  |``  
   `|  +---------------------+            | - Workspace/Container Mapper     |  |`  
   `|                                     | - Criteria Matcher               |  |`  
   `|  +---------------------+   (mpsc)   |                                  |  |`  
   `|  | Agent Socket Task   | ---------> |                                  |  |`  
   `|  | (Unix Domain Socket)|            |                                  |  |`  
   `|  +---------------------+            +-----------------+----------------+  |`  
   `|                                                       |                   |`  
   `|                                           (oneshot)   v                   |`  
   `|                                     +----------------------------------+  |`  
   `|                                     |    Sway Command Execution Loop   |  |`  
   ``|                                     |    (`swayipc_async::Connection`) |  |``  
   `|                                     +-----------------+----------------+  |`  
   `+-------------------------------------------------------|-------------------+`  
                                                           `v`  
                                            `+------------------------------+`  
                                            `|   Sway Compositor Actions    |`  
                                            ``| (`focus`, `mark`, `layout`)  |``  
                                            `+------------------------------+`

## **📦 Core Data Structures (types.rs)**

`use serde::{Deserialize, Serialize};`  
`use std::collections::HashMap;`  
`use swayipc_async as sway;`

`#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]`  
`pub struct AgentId(pub String);`

`#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`  
`pub struct ProjectId(pub String);`

`#[derive(Debug, Clone, Serialize, Deserialize)]`  
`pub enum AgentStatus {`  
    `Idle,`  
    `Thinking,`  
    `AwaitingHumanInput,`  
    `ExecutingTool,`  
    `Failed(String),`  
`}`

`#[derive(Debug, Clone)]`  
`pub struct AgentWindow {`  
    `pub id: AgentId,`  
    `pub project: ProjectId,`  
    `pub container_id: Option<i64>,`  
    `pub app_id: String,`  
    `pub mark: String,`  
    `pub status: AgentStatus,`  
`}`

`/// Commands routed to the Sway Command Executor Task`  
`#[derive(Debug)]`  
`pub enum SwayCommand {`  
    `MarkWindow { con_id: i64, mark: String },`  
    `SetBorderColor { mark: String, hex_color: String },`  
    `FocusWindow { mark: String },`  
    `MoveToWorkspace { con_id: i64, workspace: String },`  
    `ApplyLayout { workspace: String, layout: String },`  
`}`

`/// Incoming IPC events from AI agent CLI tools (Pi, Claude, etc.)`  
`#[derive(Debug, Serialize, Deserialize)]`  
`#[serde(tag = "type", rename_all = "snake_case")]`  
`pub enum AgentIPCMessage {`  
    `Register {`  
        `agent_id: AgentId,`  
        `project_id: ProjectId,`  
        `pid: u32,`  
    `},`  
    `UpdateStatus {`  
        `agent_id: AgentId,`  
        `status: AgentStatus,`  
    `},`  
    `RequestAttention {`  
        `agent_id: AgentId,`  
        `reason: String,`  
    `},`  
`}`

## **⚡ Core Engine & Channel Architecture (main.rs)**

The daemon splits operations into **three asynchronous loops** to prevent blocking lockup:

> 1. **Sway Event Subscriber:** Listens for window creations, closes, and focus changes.  
> 2. **Agent Unix Socket Server:** Listens for status updates sent directly by CLI tools.  
> 3. **Command Executor:** A single task that owns the mutable sway::Connection to run commands linearly.

`use anyhow::Result;`  
`use futures_util::StreamExt;`  
`use std::collections::HashMap;`  
`use tokio::net::UnixListener;`  
`use tokio::sync::{mpsc, oneshot};`  
`use swayipc_async::{Connection, Event, EventType, WindowChange, WindowEvent};`

`#[tokio::main]`  
`async fn main() -> Result<()> {`  
    `// 1. Channel setup`  
    `let (cmd_tx, cmd_rx) = mpsc::channel::<SwayCommand>(32);`  
    `let (engine_tx, mut engine_rx) = mpsc::channel::<EngineEvent>(64);`

    `// 2. Spawn Sway Command Executor Task`  
    `tokio::spawn(sway_command_executor_task(cmd_rx));`

    `// 3. Spawn Sway Event Listener Task`  
    `let engine_tx_sway = engine_tx.clone();`  
    `tokio::spawn(sway_event_listener_task(engine_tx_sway));`

    `// 4. Spawn Agent Unix Socket Task (/tmp/sway-agent.sock)`  
    `let engine_tx_agent = engine_tx.clone();`  
    `tokio::spawn(agent_socket_listener_task(engine_tx_agent));`

    `// 5. Main State Machine Loop`  
    `let mut state = DaemonState::default();`

    `println!("[sway-agentd] Daemon running...");`

    `while let Some(event) = engine_rx.recv().await {`  
        `match event {`  
            `EngineEvent::SwayWindowCreated(node) => {`  
                `if let Some(app_id) = &node.app_id {`  
                    `if app_id.starts_with("agent.") {`  
                        `state.handle_new_agent_window(node.id, app_id, &cmd_tx).await?;`  
                    `}`  
                `}`  
            `}`  
            `EngineEvent::AgentStatusUpdated { agent_id, status } => {`  
                `state.handle_status_update(&agent_id, status, &cmd_tx).await?;`  
            `}`  
            `EngineEvent::AgentRegistered { agent_id, project_id, pid } => {`  
                `state.register_agent(agent_id, project_id, pid);`  
            `}`  
        `}`  
    `}`

    `Ok(())`  
`}`

`#[derive(Debug)]`  
`pub enum EngineEvent {`  
    `SwayWindowCreated(swayipc_async::Node),`  
    `AgentStatusUpdated { agent_id: AgentId, status: AgentStatus },`  
    `AgentRegistered { agent_id: AgentId, project_id: ProjectId, pid: u32 },`  
`}`

`#[derive(Default)]`  
`struct DaemonState {`  
    `agents: HashMap<AgentId, AgentWindow>,`  
    `pid_to_agent: HashMap<u32, AgentId>,`  
`}`

`impl DaemonState {`  
    `async fn handle_new_agent_window(`  
        `&mut self,`  
        `con_id: i64,`  
        `app_id: &str,`  
        `cmd_tx: &mpsc::Sender<SwayCommand>,`  
    `) -> Result<()> {`  
        `let mark = format!("mark_{}", app_id.replace('.', "_"));`  
          
        `// Command Sway to mark the window for easy workspace targeting`  
        `cmd_tx.send(SwayCommand::MarkWindow {`  
            `con_id,`  
            `mark: mark.clone(),`  
        `}).await?;`

        `println!("[state] Identified Sway Agent Window: ID={}, AppID={}", con_id, app_id);`  
        `Ok(())`  
    `}`

    `async fn handle_status_update(`  
        `&mut self,`  
        `agent_id: &AgentId,`  
        `status: AgentStatus,`  
        `cmd_tx: &mpsc::Sender<SwayCommand>,`  
    `) -> Result<()> {`  
        `if let Some(agent) = self.agents.get_mut(agent_id) {`  
            `agent.status = status.clone();`

            `match status {`  
                `AgentStatus::AwaitingHumanInput => {`  
                    `// Highlight window border to signal attention needed`  
                    `cmd_tx.send(SwayCommand::SetBorderColor {`  
                        `mark: agent.mark.clone(),`  
                        `hex_color: "#ff5555".to_string(), // Red alert`  
                    `}).await?;`  
                `}`  
                `AgentStatus::Thinking => {`  
                    `cmd_tx.send(SwayCommand::SetBorderColor {`  
                        `mark: agent.mark.clone(),`  
                        `hex_color: "#ffb86c".to_string(), // Orange/Yellow`  
                    `}).await?;`  
                `}`  
                `AgentStatus::Idle => {`  
                    `cmd_tx.send(SwayCommand::SetBorderColor {`  
                        `mark: agent.mark.clone(),`  
                        `hex_color: "#6272a4".to_string(), // Normal border`  
                    `}).await?;`  
                `}`  
                `_ => {}`  
            `}`  
        `}`  
        `Ok(())`  
    `}`

    `fn register_agent(&mut self, agent_id: AgentId, project_id: ProjectId, pid: u32) {`  
        `let mark = format!("mark_agent_{}", agent_id.0);`  
        `let agent = AgentWindow {`  
            `id: agent_id.clone(),`  
            `project: project_id,`  
            `container_id: None,`  
            `app_id: format!("agent.{}", agent_id.0),`  
            `mark,`  
            `status: AgentStatus::Idle,`  
        `};`  
        `self.agents.insert(agent_id.clone(), agent);`  
        `self.pid_to_agent.insert(pid, agent_id);`  
    `}`  
`}`

## **📡 Sway Event Loop Listener Task**

This task maintains a dedicated long-lived subscription stream connected to Sway's IPC domain socket.

`async fn sway_event_listener_task(engine_tx: mpsc::Sender<EngineEvent>) -> Result<()> {`  
    `let subs = [EventType::Window];`  
    `let mut event_stream = Connection::new().await?.subscribe(subs).await?;`

    `while let Some(event) = event_stream.next().await {`  
        `if let Ok(Event::Window(win_event)) = event {`  
            `if win_event.change == WindowChange::New {`  
                `let _ = engine_tx`  
                    `.send(EngineEvent::SwayWindowCreated(win_event.container))`  
                    `.await;`  
            `}`  
        `}`  
    `}`  
    `Ok(())`  
`}`

## **⚙️ Sway Command Execution Task**

To prevent connection contention or lock degradation, execution of commands is routed through a single channel consumer using swayipc\_async::Connection.

`async fn sway_command_executor_task(mut cmd_rx: mpsc::Receiver<SwayCommand>) -> Result<()> {`  
    `let mut sway = Connection::new().await?;`

    `while let Some(cmd) = cmd_rx.recv().await {`  
        `let command_str = match cmd {`  
            `SwayCommand::MarkWindow { con_id, mark } => {`  
                `format!("[con_id=\"{}\"] mark --add \"{}\"", con_id, mark)`  
            `}`  
            `SwayCommand::SetBorderColor { mark, hex_color } => {`  
                `format!("[con_mark=\"{}\"] client.focused {}", mark, hex_color)`  
            `}`  
            `SwayCommand::FocusWindow { mark } => {`  
                `format!("[con_mark=\"{}\"] focus", mark)`  
            `}`  
            `SwayCommand::MoveToWorkspace { con_id, workspace } => {`  
                `format!("[con_id=\"{}\"] move container to workspace {}", con_id, workspace)`  
            `}`  
            `SwayCommand::ApplyLayout { workspace, layout } => {`  
                `format!("[workspace=\"{}\"] layout {}", workspace, layout)`  
            `}`  
        `};`

        `if let Err(e) = sway.run_command(&command_str).await {`  
            `eprintln!("[sway-exec-error] Command failed: '{}': {:?}", command_str, e);`  
        `}`  
    `}`  
    `Ok(())`  
`}`

## **🔌 Agent Socket Listener Task**

Exposes a Unix domain socket (/tmp/sway-agent.sock) for agents to pass JSON commands.

`async fn agent_socket_listener_task(engine_tx: mpsc::Sender<EngineEvent>) -> Result<()> {`  
    `let socket_path = "/tmp/sway-agent.sock";`  
    `let _ = tokio::fs::remove_file(socket_path).await; // Clean up old socket`

    `let listener = UnixListener::bind(socket_path)?;`

    `loop {`  
        `if let Ok((mut stream, _)) = listener.accept().await {`  
            `let engine_tx = engine_tx.clone();`  
            `tokio::spawn(async move {`  
                `let mut buf = vec![0u8; 1024];`  
                `if let Ok(n) = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await {`  
                    `if let Ok(msg) = serde_json::from_slice::<AgentIPCMessage>(&buf[..n]) {`  
                        `match msg {`  
                            `AgentIPCMessage::Register { agent_id, project_id, pid } => {`  
                                `let _ = engine_tx.send(EngineEvent::AgentRegistered { agent_id, project_id, pid }).await;`  
                            `}`  
                            `AgentIPCMessage::UpdateStatus { agent_id, status } => {`  
                                `let _ = engine_tx.send(EngineEvent::AgentStatusUpdated { agent_id, status }).await;`  
                            `}`  
                            `_ => {}`  
                        `}`  
                    `}`  
                `}`  
            `});`  
        `}`  
    `}`  
`}`

## **🧪 Client Integration Example (e.g., Python / Bash for Pi/Claude)**

When launching a sub-agent, wrap it using foot or kitty and report back over the socket:

`#!/usr/bin/env bash`  
`# launch-agent.sh`

`AGENT_ID="claude_refactor_1"`  
`PROJECT_ID="proj_auth"`

`# 1. Register with local daemon via socket`  
`echo '{"type":"register", "agent_id":"claude_refactor_1", "project_id":"proj_auth", "pid":'$$'}' | nc -U /tmp/sway-agent.sock`

`# 2. Launch floating or tabbed window via terminal with unique App ID`  
`foot --app-id="agent.${AGENT_ID}" --title="Claude: Refactor Auth" -e claude-code`

**Sources**

> 1. [swayipc-async \- crates.io: Rust Package Registry](https://crates.io/crates/swayipc-async)

---

*User prompt: Provide a complete Sway configuration snippet for handling agent windows, floating panels, and scratchpad bindings.*

Response: Here is a complete, self-contained Sway configuration snippet designed to work seamlessly with the sway-agentd daemon architecture.

Save this in \~/.config/sway/config.d/50-agent-viewer.conf (or include it in your primary \~/.config/sway/config).

`# ==============================================================================`  
`# SWAY AGENT VIEWER - Compositor Layout, Criteria Rules & Keybindings`  
`# ==============================================================================`

`# ------------------------------------------------------------------------------`  
`# 1. VISUAL STYLING & BORDERS`  
`# ------------------------------------------------------------------------------`  
`# Default border style for agent containers`  
`default_border pixel 2`

`# Status Color Palette (Driven via swaymsg/sway-agentd)`  
`# Format: class                  border   bg       text     indicator child_border`  
`client.focused_inactive          #44475a  #282a36  #f8f8f2  #44475a   #44475a`  
`client.focused                   #8be9fd  #282a36  #f8f8f2  #8be9fd   #8be9fd`

`# Custom urgency / action required state (bright orange/red)`  
`client.urgent                    #ff5555  #ff5555  #f8f8f2  #ff5555   #ff5555`

`# ------------------------------------------------------------------------------`  
`# 2. WINDOW CRITERIA & AUTOMATIC LAYOUT RULES`  
`# ------------------------------------------------------------------------------`

``# Match all agent terminal windows (`app_id` starts with "agent.")``  
`for_window [app_id="^agent\..*"] {`  
    `# Default worker agents into tabbed layout when grouped`  
    `layout tabbed`  
    `border pixel 3`  
`}`

`# Floating Orchestrator / HUD Panel`  
`for_window [app_id="agent.orchestrator"] {`  
    `floating enable`  
    `resize set width 900px height 600px`  
    `move position center`  
    `border pixel 2`  
    `sticky enable`  
`}`

`# Floating Quick-Input / Modal Agents`  
`for_window [app_id="^agent\.modal\..*"] {`  
    `floating enable`  
    `resize set width 800px height 450px`  
    `move position center`  
    `border pixel 4`  
`}`

`# Scratchpad Agent Pool (Background / Silent Processing Agents)`  
`for_window [app_id="^agent\.background\..*"] {`  
    `move scratchpad`  
    `border pixel 1`  
`}`

`# ------------------------------------------------------------------------------`  
`# 3. KEYBINDINGS & NAVIGATION ($mod = Super / Mod4)`  
`# ------------------------------------------------------------------------------`

`# --- Orchestrator & Panel Controls ---`  
`# Toggle central orchestrator panel`  
`bindsym $mod+Shift+o exec swaymsg '[app_id="agent.orchestrator"] scratchpad show' || exec foot --app-id="agent.orchestrator" -e sway-agent-hud`

`# --- Focus Navigation ---`  
`# Quick-jump to the next agent requiring human input (marked by sway-agentd)`  
`bindsym $mod+Shift+a exec swaymsg '[con_mark="needs_attention"] focus'`

`# Cycle focus between active agent windows in current workspace`  
`bindsym $mod+a focus parent; focus child; [app_id="^agent\..*"] focus`

`# --- Scratchpad Agent Management ---`  
`# Send current agent window to background scratchpad`  
`bindsym $mod+Shift+Minus move scratchpad`

`# Cycle through active background agents in scratchpad`  
`bindsym $mod+Minus scratchpad show`

`# --- Dynamic Workspace Allocation ---`  
`# Move focused agent to project workspace 9 (Dedicated AI Scratchpad)`  
`bindsym $mod+Shift+9 move container to workspace number 9; workspace number 9`

`# Create a leader/follower split (Orchestrator Left 40%, Agents Right 60%)`  
`bindsym $mod+Ctrl+l exec swaymsg 'splith; layout splitv'`

`# ------------------------------------------------------------------------------`  
`# 4. DAEMON AUTO-START`  
`# ------------------------------------------------------------------------------`  
`# Launch the Rust IPC daemon on compositor initialization`  
`exec_always --no-startup-id sway-agentd`

### **Highlights of this configuration:**

> * **Regex Criteria (^agent\\..\*):** Uses Sway’s native PCRE regex engine to categorize terminals based on \--app-id prefixing without interfering with your standard terminal windows.  
> * **Smart Focus ($mod+Shift+a):** Directly jumps focus to whichever agent has set con\_mark="needs\_attention" via the Rust daemon, skipping manual window switching when an agent prompts for terminal approval.  
> * **Scratchpad Pool Integration:** Long-running, non-interactive tasks can be dropped straight into Sway's scratchpad memory space using $mod+Shift+Minus and recalled instantly with $mod+Minus.

---

> **Superseded (protocol):** This JSON-RPC 2.0 section is superseded by [ADR 0004](../docs/decisions/0004-newline-delimited-json-ipc.md), which uses newline-delimited JSON with method-tagged messages. The `sway-agentd` naming is superseded by [ADR 0002](../docs/decisions/0002-rust-tokio-daemon.md). Kept for history; do not implement from it.

*User prompt: Define a JSON-RPC protocol specification for sub-agents (Pi, Claude Code) to communicate state to the orchestrator daemon.*

Response: Here is the formal **JSON-RPC 2.0 Protocol Specification** designed for bi-directional communication over local Unix Domain Sockets (/tmp/sway-agent.sock).

It includes transport guarantees, state machine sync, and visual layout manipulation primitives specifically tailored for Sway window management.

# **📜 sway-agentd JSON-RPC 2.0 Specification (v1.0)**

## **🌐 Transport & Framing**

> * **Transport:** Unix Domain Socket stream (/tmp/sway-agent.sock).  
> * **Framing:** Line-delimited JSON (\\n newline separated).  
> * **Base Protocol:** [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification).

## **1\. Sub-Agent → Orchestrator (Client Calls & Notifications)**

### **agent.register *(Request)***

Sent by a newly spawned sub-agent immediately after process launch to bind its process metadata and Sway window criteria.

#### **Request**

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 1,`  
  `"method": "agent.register",`  
  `"params": {`  
    `"agent_id": "pi-refactor-auth",`  
    `"agent_type": "pi",`  
    `"project_id": "core-backend",`  
    `"pid": 48201,`  
    `"app_id": "agent.pi.refactor-auth",`  
    `"capabilities": ["diff_view", "interactive_approval", "tool_execution"]`  
  `}`  
`}`

#### **Response**

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 1,`  
  `"result": {`  
    `"status": "registered",`  
    `"con_mark": "agent_pi_refactor_auth",`  
    `"assigned_workspace": "2:core-backend"`  
  `}`  
`}`

### **agent.update\_status *(Notification)***

Fire-and-forget status update emitted whenever the agent transitions state. This drives status indicators and window border styling in Sway.

#### **Notification**

`{`  
  `"jsonrpc": "2.0",`  
  `"method": "agent.update_status",`  
  `"params": {`  
    `"agent_id": "pi-refactor-auth",`  
    `"state": "awaiting_human_input",`  
    `"progress": 0.75,`  
    `"current_task": "Approve breaking schema migration in database.rs",`  
    `"attention_reason": "permission_prompt"`  
  `}`  
`}`

#### **Valid States Matrix (state parameter)**

| State Enum | Visual Effect in Sway | Urgency | Description |
| :---- | :---- | :---- | :---- |
| idle | Standard border (\#44475a) | None | Waiting for next prompt |
| thinking | Yellow pulsing border (\#ffb86c) | Low | Processing prompt / LLM stream |
| executing\_tool | Blue border (\#8be9fd) | Low | Running shell scripts, tests, compilers |
| awaiting\_human\_input | Red alert border (\#ff5555) | **Urgent** | Triggered on approval prompts, CLI input |
| completed | Green border (\#50fa7b) | Low | Sub-task successfully finished |
| failed | Bright magenta border (\#ff79c6) | High | Unhandled crash or task abortion |

### **agent.request\_window\_action *(Request)***

Allows an agent to directly request Sway layout adjustments from the orchestrator daemon.

#### **Request (e.g., Focus self, pop out to floating)**

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 2,`  
  `"method": "agent.request_window_action",`  
  `"params": {`  
    `"agent_id": "pi-refactor-auth",`  
    `"action": "float_and_center",`  
    `"dimensions": {`  
      `"width": 1000,`  
      `"height": 600`  
    `}`  
  `}`  
`}`

#### **Supported Actions**

> * focus: Direct focus jump to this agent container.  
> * move\_to\_scratchpad: Minimizes agent to Sway background scratchpad.  
> * float\_and\_center: Converts container to floating and centers on active workspace.  
> * tile: Restores container back into the primary layout tree.

#### **Response**

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 2,`  
  `"result": {`  
    `"success": true,`  
    `"action_applied": "float_and_center"`  
  `}`  
`}`

### **agent.heartbeat *(Notification)***

Periodic ping emitted every 15 seconds. If an agent crashes without cleaning up, sway-agentd uses heartbeat timeouts to strip stale Sway container marks.

`{`  
  `"jsonrpc": "2.0",`  
  `"method": "agent.heartbeat",`  
  `"params": {`  
    `"agent_id": "pi-refactor-auth",`  
    `"timestamp": 1785002080`  
  `}`  
`}`

## **2\. Orchestrator → Sub-Agent (Server Invocations)**

The daemon can issue JSON-RPC commands *back* to the agent process over the established socket connection.

### **orchestrator.interrupt *(Request)***

Triggered when a user uses a Sway hotkey (e.g., $mod+Escape) to pause or abort an agent's current task without killing the terminal process.

#### **Request**

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 101,`  
  `"method": "orchestrator.interrupt",`  
  `"params": {`  
    `"reason": "user_hotkey_override",`  
    `"grace_period_ms": 1000`  
  `}`  
`}`

#### **Response**

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 101,`  
  `"result": {`  
    `"interrupted": true,`  
    `"current_state": "idle"`  
  `}`  
`}`

### **orchestrator.focus\_changed *(Notification)***

Pushed to the sub-agent when Sway gains or loses window focus on its terminal container.

`{`  
  `"jsonrpc": "2.0",`  
  `"method": "orchestrator.focus_changed",`  
  `"params": {`  
    `"focused": false,`  
    `"timestamp": 1785002100`  
  `}`  
`}`

## **3\. Protocol Error Codes**

Standard JSON-RPC 2.0 error codes apply (-32700 Parse error, \-32600 Invalid Request). Custom application errors use the \-32000 to \-32099 range:

`{`  
  `"jsonrpc": "2.0",`  
  `"id": 1,`  
  `"error": {`  
    `"code": -32001,`  
    `"message": "Sway container handle not found",`  
    `"data": {`  
      `"app_id": "agent.pi.refactor-auth",`  
      `"hint": "Ensure terminal process was started with --app-id flag"`  
    `}`  
  `}`  
`}`

| Error Code | Meaning | Description |
| :---- | :---- | :---- |
| \-32000 | Registration Failed | Duplicate agent\_id or invalid pid. |
| \-32001 | Sway Container Missing | Sway compositor reported no matching window container for app\_id. |
| \-32002 | Invalid Action | Requested window action is unsupported or prohibited by container policy. |
| \-32003 | Project Conflict | project\_id mapped to an invalid or non-existent workspace. |

---

