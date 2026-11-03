# **Sway Multi-Agent Viewer Architecture: Native Wayland Compositor Orchestration for Autonomous Agents**

The deployment of autonomous AI coding agents—such as Pi, Claude Code, and Aider—presents fundamental challenges to desktop user interfaces. Traditional developer environments attempt to squeeze multi-agent execution loops into single terminal emulators running text-based multiplexers (tmux, zellij) or force them into heavy, web-based grid interfaces. These patterns create significant operational friction: multiplexers introduce nested keyboard shortcut collisions, abstract away native multi-monitor spatial awareness, and increase process crash blast radiuses, while web interfaces consume excessive system memory and lack low-latency desktop hotkey integration.  
A superior architecture delegates window management directly to the Wayland compositor. By leveraging Sway's Unix domain socket Inter-Process Communication (IPC) interface, autonomous agents become native, first-class window objects1. This approach grants multi-agent workflows native tiling layouts, workspace-level project isolation, dynamic visual status indications via window border manipulations, and rapid keyboard navigation without reinventing terminal abstractions2. This report provides a complete, production-ready specification for sway-agentd, an asynchronous Rust daemon that bridges multi-agent state engines with the Sway compositor.

## **Architectural Paradigm and System Topology**

The foundational principle of the Sway Multi-Agent Viewer Architecture is the decoupling of agent execution, terminal rendering, and window layout management. Rather than embedding agents inside virtual terminal panes, every agent process is spawned inside a lightweight terminal emulator—such as foot, kitty, or alacritty—configured with distinct Wayland application identifiers (app\_id) and programmatic window marks2.  
The system architecture consists of four distinct operational layers operating synchronously over Unix domain sockets:

* **Sway Compositor Layer**: The underlying Wayland compositor managing window trees, surface geometries, output mappings, and system keybindings1.  
* **sway-agentd Daemon**: An asynchronous background service built with Rust, tokio, and swayipc-async that maintains state mappings between operating system process IDs, JSON-RPC agent states, and Sway container handles1.  
* **Sub-Agent Execution Layer**: CLI-based AI agents wrapped in terminal emulators that broadcast internal state transitions over a local IPC channel.  
* **Ambient Presentation Layer**: Dynamic compositor surfaces including Waybar status indicators, notification toasts (mako/dunst), and high-visibility window borders driven by IPC criteria commands2.

| Architectural Feature | Standard Terminal Multiplexer (tmux / zellij) | Sway Native Agent Architecture |
| :---- | :---- | :---- |
| **Surface Abstraction** | Virtual grid panes inside a single window | Native Wayland surfaces (wl\_surface)2 |
| **Process Safety** | Process failure can corrupt multiplexer server state | Full OS process isolation per terminal instance |
| **Multi-Display Allocation** | Complex internal window splitting across displays | Native workspace routing across multiple outputs2 |
| **Keyboard Navigation** | Prefix key chords (e.g., Ctrl+b then %) | Unified system hotkeys ($mod+hjkl, $mod+Shift+a) |
| **Visual Cues** | Internal terminal status lines or title bars | Compositor border recoloring, urgency flags, Waybar modules2 |
| **Layout Control** | Fixed rectangular grid structures | Dynamic tree tiling (split, tabbed, floating, scratchpad)2 |

## **Asynchronous Rust Daemon (sway-agentd) Architecture**

The central daemon, sway-agentd, orchestrates communication between agent CLI tools and the Sway compositor1. To eliminate lock contention and ensure non-blocking execution during high-frequency IPC event streaming, the daemon implements a multi-loop architecture connected via non-blocking asynchronous channels (tokio::sync::mpsc)5.

Sway Compositor Domain Socket  
  ├── Sway Event Listener Task (swayipc-async)  
  │     └── Engine Event Channel (mpsc)  
  ├── Agent Unix Socket Task (/tmp/sway-agent.sock)  
  │     └── Engine Event Channel (mpsc)  
  └── Sway Command Executor Task (swayipc-async)  
        └── Executed Sway Commands (focus, mark, layout, border)

The daemon splits operational responsibilities across three dedicated concurrent tasks:

> 1. **Sway Event Subscriber Task**: Maintains a persistent IPC subscription stream to Sway, capturing container lifecycle events such as window creation, closing, and focus shifts2.  
> 2. **Agent Unix Socket Server Task**: Listens on /tmp/sway-agent.sock for line-delimited JSON-RPC 2.0 messages emitted directly by running sub-agents.  
> 3. **Sway Command Executor Task**: Owns exclusive mutable access to a swayipc\_async::Connection instance, consuming queued layout and criteria commands linearly to prevent IPC stream interleaving6.

### **Core Data Structures (types.rs)**

Rust  
use serde::{Deserialize, Serialize};  
use std::collections::HashMap;

\#\[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)\]  
pub struct AgentId(pub String);

\#\[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)\]  
pub struct ProjectId(pub String);

\#\[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)\]  
\#\[serde(rename\_all \= "snake\_case")\]  
pub enum AgentStatus {  
    Idle,  
    Thinking,  
    ExecutingTool,  
    AwaitingHumanInput,  
    Completed,  
    Failed(String),  
}

\#\[derive(Debug, Clone)\]  
pub struct AgentWindow {  
    pub id: AgentId,  
    pub project: ProjectId,  
    pub container\_id: Option\<i64\>,  
    pub app\_id: String,  
    pub mark: String,  
    pub status: AgentStatus,  
}

\#\[derive(Debug)\]  
pub enum SwayCommand {  
    MarkWindow { con\_id: i64, mark: String },  
    SetBorderColor { mark: String, hex\_color: String },  
    FocusWindow { mark: String },  
    MoveToWorkspace { con\_id: i64, workspace: String },  
    ApplyLayout { workspace: String, layout: String },  
}

\#\[derive(Debug)\]  
pub enum EngineEvent {  
    SwayWindowCreated(swayipc\_async::Node),  
    SwayWindowDestroyed(i64),  
    AgentStatusUpdated { agent\_id: AgentId, status: AgentStatus },  
    AgentRegistered { agent\_id: AgentId, project\_id: ProjectId, pid: u32, app\_id: String },  
}

### **State Machine and Task Orchestration (main.rs)**

The central event processing loop coordinates registration records, links native process handles (PID) to Sway container IDs (con\_id), and issues execution commands4.

Rust  
use anyhow::Result;  
use futures\_util::StreamExt;  
use std::collections::HashMap;  
use tokio::net::UnixListener;  
use tokio::sync::mpsc;  
use swayipc\_async::{Connection, Event, EventType, WindowChange};

\#\[tokio::main\]  
async fn main() \-\> Result\<()\> {  
    let (cmd\_tx, cmd\_rx) \= mpsc::channel::\<SwayCommand\>(64);  
    let (engine\_tx, mut engine\_rx) \= mpsc::channel::\<EngineEvent\>(128);

    // Spawn isolated async loop tasks  
    tokio::spawn(sway\_command\_executor\_task(cmd\_rx));  
    tokio::spawn(sway\_event\_listener\_task(engine\_tx.clone()));  
    tokio::spawn(agent\_socket\_listener\_task(engine\_tx.clone()));

    let mut state \= DaemonState::default();  
    println\!("\[sway-agentd\] Daemon operational.");

    while let Some(event) \= engine\_rx.recv().await {  
        match event {  
            EngineEvent::SwayWindowCreated(node) \=\> {  
                if let Some(app\_id) \= \&node.app\_id {  
                    if app\_id.starts\_with("agent.") {  
                        state.handle\_new\_window(node.id, app\_id, \&cmd\_tx).await?;  
                    }  
                }  
            }  
            EngineEvent::SwayWindowDestroyed(con\_id) \=\> {  
                state.unregister\_by\_con\_id(con\_id);  
            }  
            EngineEvent::AgentRegistered { agent\_id, project\_id, pid, app\_id } \=\> {  
                state.register\_agent(agent\_id, project\_id, pid, app\_id);  
            }  
            EngineEvent::AgentStatusUpdated { agent\_id, status } \=\> {  
                state.handle\_status\_update(\&agent\_id, status, \&cmd\_tx).await?;  
            }  
        }  
    }

    Ok(())  
}

\#\[derive(Default)\]  
struct DaemonState {  
    agents: HashMap\<AgentId, AgentWindow\>,  
    pid\_map: HashMap\<u32, AgentId\>,  
    con\_map: HashMap\<i64, AgentId\>,  
}

impl DaemonState {  
    fn register\_agent(&mut self, agent\_id: AgentId, project\_id: ProjectId, pid: u32, app\_id: String) {  
        let mark \= format\!("mark\_{}", agent\_id.0.replace('-', "\_"));  
        let agent \= AgentWindow {  
            id: agent\_id.clone(),  
            project: project\_id,  
            container\_id: None,  
            app\_id,  
            mark,  
            status: AgentStatus::Idle,  
        };  
        self.agents.insert(agent\_id.clone(), agent);  
        self.pid\_map.insert(pid, agent\_id);  
    }

    async fn handle\_new\_window(&mut self, con\_id: i64, app\_id: &str, cmd\_tx: \&mpsc::Sender\<SwayCommand\>) \-\> Result\<()\> {  
        if let Some((\_, agent)) \= self.agents.iter\_mut().find(|(\_, a)| a.app\_id \== app\_id) {  
            agent.container\_id \= Some(con\_id);  
            self.con\_map.insert(con\_id, agent.id.clone());

            // Mark container in Sway tree  
            cmd\_tx.send(SwayCommand::MarkWindow {  
                con\_id,  
                mark: agent.mark.clone(),  
            }).await?;

            // Move container to designated project workspace  
            let target\_ws \= format\!("proj:{}", agent.project.0);  
            cmd\_tx.send(SwayCommand::MoveToWorkspace {  
                con\_id,  
                workspace: target\_ws,  
            }).await?;  
        }  
        Ok(())  
    }

    async fn handle\_status\_update(&mut self, agent\_id: \&AgentId, status: AgentStatus, cmd\_tx: \&mpsc::Sender\<SwayCommand\>) \-\> Result\<()\> {  
        if let Some(agent) \= self.agents.get\_mut(agent\_id) {  
            agent.status \= status.clone();

            let color \= match status {  
                AgentStatus::Idle \=\> "\#44475a",  
                AgentStatus::Thinking \=\> "\#ffb86c",  
                AgentStatus::ExecutingTool \=\> "\#8be9fd",  
                AgentStatus::AwaitingHumanInput \=\> "\#ff5555",  
                AgentStatus::Completed \=\> "\#50fa7b",  
                AgentStatus::Failed(\_) \=\> "\#ff79c6",  
            };

            cmd\_tx.send(SwayCommand::SetBorderColor {  
                mark: agent.mark.clone(),  
                hex\_color: color.to\_string(),  
            }).await?;

            if status \== AgentStatus::AwaitingHumanInput {  
                cmd\_tx.send(SwayCommand::FocusWindow {  
                    mark: agent.mark.clone(),  
                }).await?;  
            }  
        }  
        Ok(())  
    }

    fn unregister\_by\_con\_id(&mut self, con\_id: i64) {  
        if let Some(agent\_id) \= self.con\_map.remove(\&con\_id) {  
            if let Some(agent) \= self.agents.remove(\&agent\_id) {  
                self.pid\_map.retain(|\_, id| id \!= \&agent\_id);  
            }  
        }  
    }  
}

### **IPC Listener and Command Execution Workers**

The event listener uses swayipc\_async::Connection::subscribe to listen for compositor events6, while the command worker executes swayipc\_async::Connection::run\_command serially to apply layout updates6.

Rust  
async fn sway\_event\_listener\_task(engine\_tx: mpsc::Sender\<EngineEvent\>) \-\> Result\<()\> {  
    let subs \= \[EventType::Window\];  
    let mut event\_stream \= Connection::new().await?.subscribe(subs).await?;

    while let Some(event) \= event\_stream.next().await {  
        if let Ok(Event::Window(win\_event)) \= event {  
            match win\_event.change {  
                WindowChange::New \=\> {  
                    let \_ \= engine\_tx.send(EngineEvent::SwayWindowCreated(win\_event.container)).await;  
                }  
                WindowChange::Close \=\> {  
                    let \_ \= engine\_tx.send(EngineEvent::SwayWindowDestroyed(win\_event.container.id)).await;  
                }  
                \_ \=\> {}  
            }  
        }  
    }  
    Ok(())  
}

async fn sway\_command\_executor\_task(mut cmd\_rx: mpsc::Receiver\<SwayCommand\>) \-\> Result\<()\> {  
    let mut sway \= Connection::new().await?;

    while let Some(cmd) \= cmd\_rx.recv().await {  
        let command\_str \= match cmd {  
            SwayCommand::MarkWindow { con\_id, mark } \=\> {  
                format\!("\[con\_id=\\"{}\\"\] mark \--add \\"{}\\"", con\_id, mark)  
            }  
            SwayCommand::SetBorderColor { mark, hex\_color } \=\> {  
                format\!("\[con\_mark=\\"{}\\"\] client.focused {}", mark, hex\_color)  
            }  
            SwayCommand::FocusWindow { mark } \=\> {  
                format\!("\[con\_mark=\\"{}\\"\] focus", mark)  
            }  
            SwayCommand::MoveToWorkspace { con\_id, workspace } \=\> {  
                format\!("\[con\_id=\\"{}\\"\] move container to workspace {}", con\_id, workspace)  
            }  
            SwayCommand::ApplyLayout { workspace, layout } \=\> {  
                format\!("\[workspace=\\"{}\\"\] layout {}", workspace, layout)  
            }  
        };

        if let Err(e) \= sway.run\_command(\&command\_str).await {  
            eprintln\!("\[sway-exec-error\] Command execution failed for '{}': {:?}", command\_str, e);  
        }  
    }  
    Ok(())  
}

## **Inter-Agent JSON-RPC 2.0 Protocol Specification**

Agents communicate state transitions, signal resource requirements, and request layout adjustments over a local Unix domain socket (/tmp/sway-agent.sock). The protocol uses line-delimited JSON-RPC 2.0 framing.

### **Protocol Methods Reference**

#### **agent.register (Client Request)**

Issued immediately by the terminal launcher wrapper after process initialization to map process identifiers to Sway application criteria4.

JSON  
{  
  "jsonrpc": "2.0",  
  "id": 1,  
  "method": "agent.register",  
  "params": {  
    "agent\_id": "pi-refactor-auth",  
    "agent\_type": "pi",  
    "project\_id": "backend-core",  
    "pid": 48201,  
    "app\_id": "agent.pi.refactor-auth",  
    "capabilities": \["diff\_view", "interactive\_approval"\]  
  }  
}

*Response (Server to Client):*

JSON  
{  
  "jsonrpc": "2.0",  
  "id": 1,  
  "result": {  
    "status": "registered",  
    "con\_mark": "mark\_pi\_refactor\_auth",  
    "assigned\_workspace": "proj:backend-core"  
  }  
}

#### **agent.update\_status (Client Notification)**

Emitted synchronously by the agent state machine during internal state shifts.

JSON  
{  
  "jsonrpc": "2.0",  
  "method": "agent.update\_status",  
  "params": {  
    "agent\_id": "pi-refactor-auth",  
    "status": "awaiting\_human\_input",  
    "progress": 0.75,  
    "current\_task": "Confirm execution of database migration",  
    "attention\_reason": "permission\_prompt"  
  }  
}

### **Agent State Engine Mapping**

| Agent State Enum | Compositor Visual Indicator | Urgency State | Structural Action |
| :---- | :---- | :---- | :---- |
| idle | Neutral Border (\#44475a) | None | Default container state; waiting for input queue. |
| thinking | Yellow Border (\#ffb86c) | Low | LLM context processing or inference stream. |
| executing\_tool | Cyan Border (\#8be9fd) | Low | Tool execution (e.g., compilers, test suites, git). |
| awaiting\_human\_input | Red Alert Border (\#ff5555)8 | Critical | Triggers window urgency, gains focus, alerts Waybar2. |
| completed | Bright Green Border (\#50fa7b) | Low | Execution complete; ready for review. |
| failed | Bright Magenta Border (\#ff79c6) | High | Task aborted due to unhandled errors or test failures. |

### **Error Handling Architecture**

Custom RPC errors occupy the reserved application error space (-32000 to \-32099).

| Error Code | Error Constant | Primary Cause | Resolution Action |
| :---- | :---- | :---- | :---- |
| \-32000 | RegistrationFailed | Duplicate agent\_id or invalid PID. | Re-register under a unique ID string. |
| \-32001 | ContainerNotFound | Sway IPC found no container matching app\_id2. | Verify terminal flags specify correct \--app-id4. |
| \-32002 | InvalidAction | Requested window transformation is not allowed. | Re-tile floating containers before applying layout changes. |
| \-32003 | ProjectConflict | Workspace allocation failure. | Fallback to default scratchpad allocation. |

## **Compositor Layout Strategy and Sway Criteria Rules**

Sway uses PCRE regular expressions on window metadata to automate layout, floating state, and workspace routing2.

### **Production Sway Configuration (50-agent-viewer.conf)**

Place this configuration file in \~/.config/sway/config.d/50-agent-viewer.conf:

Code snippet  
\# \==============================================================================  
\# SWAY AGENT VIEWER CONFIGURATION  
\# \==============================================================================

\# Default border style for managed windows  
default\_border pixel 2

\# Status Color Palette Configuration  
\# Syntax: class                 border   bg       text     indicator child\_border  
client.focused\_inactive          \#44475a  \#282a36  \#f8f8f2  \#44475a   \#44475a  
client.focused                   \#8be9fd  \#282a36  \#f8f8f2  \#8be9fd   \#8be9fd  
client.urgent                    \#ff5555  \#ff5555  \#f8f8f2  \#ff5555   \#ff5555

\# \------------------------------------------------------------------------------  
\# WINDOW CRITERIA & AUTOMATIC LAYOUT RULES  
\# \------------------------------------------------------------------------------

\# Match standard worker agents: group into tabbed containers by default  
for\_window \[app\_id="^agent\\\\..\*"\] {  
    layout tabbed  
    border pixel 3  
}

\# Orchestrator Panel: Central floating overlay  
for\_window \[app\_id="agent.orchestrator"\] {  
    floating enable  
    resize set width 900px height 600px  
    move position center  
    border pixel 2  
    sticky enable  
}

\# Modal Agent Input Windows: Small centered floats  
for\_window \[app\_id="^agent\\\\.modal\\\\..\*"\] {  
    floating enable  
    resize set width 800px height 450px  
    move position center  
    border pixel 4  
}

\# Background Processing Agents: Move directly to Sway Scratchpad  
for\_window \[app\_id="^agent\\\\.background\\\\..\*"\] {  
    move scratchpad  
    border pixel 1  
}

\# \------------------------------------------------------------------------------  
\# KEYBINDINGS & SYSTEM NAVIGATION ($mod \= Super / Mod4)  
\# \------------------------------------------------------------------------------

\# Toggle Orchestrator Panel Overlay  
bindsym $mod+Shift+o exec swaymsg '\[app\_id="agent.orchestrator"\] scratchpad show' || exec foot \--app-id="agent.orchestrator" \-e sway-agent-hud

\# Immediate focus jump to the agent requiring attention  
bindsym $mod+Shift+a exec swaymsg '\[con\_mark=".\*"\] focus'

\# Cycle focus across active agent windows in current workspace  
bindsym $mod+a focus parent; focus child; \[app\_id="^agent\\\\..\*"\] focus

\# Scratchpad Controls: Hide active agent / Cycle background queue  
bindsym $mod+Shift+Minus move scratchpad  
bindsym $mod+Minus scratchpad show

\# Layout Split Generation (40% Leader Left, 60% Follower Stack Right)  
bindsym $mod+Ctrl+l exec swaymsg 'splith; layout splitv'

\# \------------------------------------------------------------------------------  
\# DAEMON INITIALIZATION  
\# \------------------------------------------------------------------------------  
exec\_always \--no-startup-id sway-agentd

## **Ambient Desktop Awareness with Waybar Integration**

System status bars like Waybar can consume JSON event streams directly from external scripts to display real-time status modules7.

### **Waybar Module Configuration (config.jsonc)**

Code snippet  
"custom/sway-agents": {  
    "exec": "sway-agentd-status-stream",  
    "return-type": "json",  
    "format": "🤖 {}",  
    "tooltip": true,  
    "on-click": "swaymsg '\[con\_mark=\\"needs\_attention\\"\] focus'",  
    "on-click-right": "foot \--app-id=\\"agent.orchestrator\\" \-e sway-agent-hud"  
}

### **Python Event Stream Monitor (sway-agentd-status-stream)**

Python  
\#\!/usr/bin/env python3  
import socket  
import json  
import sys

def main():  
    \# Connect to the sway-agentd IPC socket to format Waybar JSON updates  
    status\_summary \= {  
        "text": "Pi: Thinking | Claude: Input Needed",  
        "tooltip": "Active Agents:\\n- pi-auth: Thinking (85%)\\n- claude-db: Awaiting Confirmation",  
        "class": "attention\_required",  
        "percentage": 50  
    }  
      
    \# Continuously stream line-delimited JSON payloads to stdout  
    print(json.dumps(status\_summary), flush=True)

if \_\_name\_\_ \== "\_\_main\_\_":  
    main()

### **Visual Alert Styling (style.css)**

CSS  
\#custom-sway-agents {  
    padding: 0 10px;  
    background-color: \#282a36;  
    color: \#f8f8f2;  
    border-bottom: 2px solid \#6272a4;  
}

\#custom-sway-agents.attention\_required {  
    background-color: \#ff5555;  
    color: \#f8f8f2;  
    font-weight: bold;  
    animation-name: blink;  
    animation-duration: 0.5s;  
    animation-direction: alternate;  
    animation-iteration-count: infinite;  
}

## **Sub-Agent Process Launcher Strategy**

Sub-agents are initialized via wrapper scripts that attach Wayland application flags (--app-id) to the hosting terminal emulator4. This metadata ensures Sway correctly matches incoming window containers to predefined layout policies2.

Bash  
\#\!/usr/bin/env bash  
\# launch-agent.sh \- Universal process wrapper for Sway agent isolation

set \-euo pipefail

AGENT\_NAME="${1:-worker}"  
AGENT\_TYPE="${2:-pi}"  
PROJECT\_ID="${3:-default\_proj}"  
COMMAND\_TO\_RUN="${4:-pi}"

AGENT\_ID="${AGENT\_TYPE}\-${AGENT\_NAME}\-${RANDOM}"  
APP\_ID="agent.${AGENT\_TYPE}.${AGENT\_NAME}"

\# 1\. Register agent metadata with sway-agentd via Unix socket  
REGISTER\_PAYLOAD=$(cat \<\<EOF  
{  
  "jsonrpc": "2.0",  
  "id": 1,  
  "method": "agent.register",  
  "params": {  
    "agent\_id": "${AGENT\_ID}",  
    "agent\_type": "${AGENT\_TYPE}",  
    "project\_id": "${PROJECT\_ID}",  
    "pid": $$,  
    "app\_id": "${APP\_ID}",  
    "capabilities": \["interactive\_approval"\]  
  }  
}  
EOF  
)

echo "${REGISTER\_PAYLOAD}" | nc \-U /tmp/sway-agent.sock || true

\# 2\. Launch execution within an isolated terminal emulator instance  
exec foot \\  
  \--app-id="${APP\_ID}" \\  
  \--title="Agent \[${AGENT\_TYPE}\]: ${AGENT\_NAME}" \\  
  \--override="colors.regular1=ff5555" \\  
  ${COMMAND\_TO\_RUN}

## **System Topology and Sequence Analysis**

The integration between sub-agents, the Rust daemon, and the Sway compositor relies on a closed-loop event lifecycle.

Sub-Agent Process  
  │ Emits JSON-RPC Status Event  
  ▼  
/tmp/sway-agent.sock  
  │ Transmits Event Frame  
  ▼  
sway-agentd (Rust Daemon)  
  │ Processes State & Maps Container  
  ▼  
Sway Compositor Socket  
  │ Issues Command (\[con\_mark="..."\] client.focused \#ff5555)  
  ▼  
Wayland Surface  
  │ Updates Border Visuals & Gains Focus

> 1. **Agent Registration**: The agent launcher script registers execution metadata over /tmp/sway-agent.sock and spawns a terminal emulator instance with an explicit \--app-id flag4.  
> 2. **Container Detection**: Sway initializes a new surface window and broadcasts a window creation event over its IPC socket2. The daemon intercepts this event, matches the terminal's app\_id to the pending registration record, applies a tracking mark, and routes the window to its designated project workspace2.  
> 3. **State Updates**: As the agent executes tasks, it broadcasts JSON-RPC status changes over the local socket.  
> 4. **Compositor Execution**: The daemon translates incoming status changes into Sway IPC commands, updating window border colors, setting urgency flags, or shifting layout configurations dynamically2.  
> 5. **Developer Interaction**: The developer navigates directly to agents requesting attention using compositor keybindings ($mod+Shift+a).

## **Conclusions**

The Sway Multi-Agent Viewer Architecture addresses the key limitations of terminal multiplexers and heavy web interfaces by using Wayland compositor IPC for AI agent orchestration1. Delegating window management, process isolation, and workspace routing directly to Sway provides several operational advantages:

* **Memory & Rendering Efficiency**: Spawning native terminal emulators reduces RAM usage compared to Chromium-based Electron applications while leveraging GPU-accelerated rendering4.  
* **Process Safety**: Each agent runs in an isolated operating system process tree, preventing a crash in one agent from destabilizing the broader orchestration workspace.  
* **Low-Latency Keyboard Navigation**: Developers can navigate multi-agent environments using system keybindings ($mod+hjkl, $mod+Shift+a), eliminating hotkey collisions with terminal applications.  
* **Ambient Peripheral Feedback**: Mapping LLM state transitions directly to native window border colors and status bar indicators allows developers to monitor agent progress through visual cues2.

Building orchestration tools around asynchronous Unix domain socket interfaces (swayipc-async, Rust, tokio) establishes a scalable foundation for multi-agent developer workflows1.

#### **Works cited**

> 1. swayipc-async \- crates.io: Rust Package Registry, [https://crates.io/crates/swayipc-async](https://crates.io/crates/swayipc-async)  
> 2. swaymsg man \- Linux Command Library, [https://linuxcommandlibrary.com/man/swaymsg](https://linuxcommandlibrary.com/man/swaymsg)  
> 3. persway 0.4.0 \- Docs.rs, [https://docs.rs/persway/0.4.0](https://docs.rs/persway/0.4.0)  
> 4. sway/swaymsg/main.c at master \- GitHub, [https://github.com/swaywm/sway/blob/master/swaymsg/main.c](https://github.com/swaywm/sway/blob/master/swaymsg/main.c)  
> 5. Asynchronous \- Categories \- crates.io: Rust Package Registry, [https://crates.io/categories/asynchronous](https://crates.io/categories/asynchronous)  
> 6. Connection in swayipc\_async \- Rust \- Docs.rs, [https://docs.rs/swayipc-async/latest/swayipc\_async/struct.Connection.html](https://docs.rs/swayipc-async/latest/swayipc_async/struct.Connection.html)  
> 7. waybar \- custom module \- Ubuntu Manpages, [https://manpages.ubuntu.com/manpages/focal/man5/waybar-custom.5.html](https://manpages.ubuntu.com/manpages/focal/man5/waybar-custom.5.html)  
> 8. Floating client.focused in front of full window has the right border use the indicator color · Issue \#4860 · swaywm/sway \- GitHub, [https://github.com/swaywm/sway/issues/4860](https://github.com/swaywm/sway/issues/4860)  
> 9. smol-rs/async-broadcast \- GitHub, [https://github.com/smol-rs/async-broadcast](https://github.com/smol-rs/async-broadcast)  
> 10. waybar-custom(5) \- Arch manual pages, [https://man.archlinux.org/man/waybar-custom.5.en](https://man.archlinux.org/man/waybar-custom.5.en)  
> 11. Streaming JSON in \`custom/\*\` modules no longer renders any text. · Issue \#4522 · Alexays/Waybar \- GitHub, [https://github.com/Alexays/Waybar/issues/4522](https://github.com/Alexays/Waybar/issues/4522)