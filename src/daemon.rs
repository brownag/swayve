use crate::config::Config;
use crate::layout::{LayoutAction, LayoutManager};
use crate::sway::{SwayCommand, SwayEvent};
use crate::types::{AgentId, AgentIPCMessage, AgentRegistry, AgentStatus, AgentWindow, DiffFrame, ProjectId};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

pub enum SubMode {
    Full,
    Diff {
        needs_snapshot: bool,
        sent: HashMap<AgentId, AgentWindow>,
    },
}

pub struct Subscriber {
    pub tx: mpsc::Sender<String>,
    pub mode: SubMode,
}

pub struct DaemonState {
    pub config: Config,
    pub agents: HashMap<AgentId, AgentWindow>,
    pub pid_to_agent: HashMap<u32, AgentId>,
    pub con_to_agent: HashMap<i64, AgentId>,
    pub subscribers: Vec<Subscriber>,
}

impl DaemonState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            agents: HashMap::new(),
            pid_to_agent: HashMap::new(),
            con_to_agent: HashMap::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn push_full_subscriber(&mut self, tx: mpsc::Sender<String>) {
        self.subscribers.push(Subscriber { tx, mode: SubMode::Full });
    }

    pub fn push_diff_subscriber(&mut self, tx: mpsc::Sender<String>) {
        self.subscribers.push(Subscriber {
            tx,
            mode: SubMode::Diff { needs_snapshot: true, sent: HashMap::new() },
        });
    }

    pub fn broadcast_status(&mut self) {
        if self.subscribers.is_empty() {
            return;
        }
        let agents_list: Vec<&AgentWindow> = self.agents.values().collect();
        let json = match serde_json::to_string(&agents_list) {
            Ok(j) => j,
            Err(e) => {
                eprintln!(
                    "[daemon] Failed to serialize agents for subscription broadcast: {:?}",
                    e
                );
                return;
            }
        };

        self.subscribers.retain(|sub| {
            if !matches!(sub.mode, SubMode::Full) {
                return true;
            }
            match sub.tx.try_send(json.clone()) {
                Ok(()) => true,
                Err(mpsc::error::TrySendError::Full(_)) => {
                    eprintln!("[daemon] Subscriber channel full; retaining subscriber");
                    true
                }
                Err(mpsc::error::TrySendError::Closed(_)) => false,
            }
        });
        self.broadcast_diff();
    }

    fn broadcast_diff(&mut self) {
        let agents = &self.agents;
        self.subscribers.retain_mut(|sub| {
            let SubMode::Diff { needs_snapshot, sent } = &mut sub.mode else {
                return true;
            };
            let (snapshot, upsert, remove): (bool, Vec<&AgentWindow>, Vec<&AgentId>) = if *needs_snapshot {
                (true, agents.values().collect(), Vec::new())
            } else {
                let up = agents
                    .iter()
                    .filter(|(id, a)| {
                        sent.get(*id).is_none_or(|old| old != *a)
                    })
                    .map(|(_, a)| a)
                    .collect();
                let rm = sent.keys().filter(|id| !agents.contains_key(*id)).collect();
                (false, up, rm)
            };
            if !snapshot && upsert.is_empty() && remove.is_empty() {
                return true;
            }
            let frame = DiffFrame {
                snapshot,
                upsert: upsert.into_iter().cloned().collect(),
                remove: remove.into_iter().cloned().collect(),
            };
            let json = match frame.to_line() {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("[daemon] Failed to serialize diff broadcast: {:?}", e);
                    return true;
                }
            };
            match sub.tx.try_send(json) {
                Ok(()) => {
                    *sent = agents.clone();
                    *needs_snapshot = false;
                    true
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    eprintln!("[daemon] Diff subscriber channel full; will send snapshot next");
                    *needs_snapshot = true;
                    true
                }
                Err(mpsc::error::TrySendError::Closed(_)) => false,
            }
        });
    }

    /// Registers an agent. Returns a monitor request when the auto-monitor rule applies; the
    /// loop executes it (G54).
    pub fn register_agent(&mut self, agent_id: AgentId, project_id: ProjectId, pid: u32, app_id: String, parent_id: Option<AgentId>) -> Option<MonitorRequest> {
        let is_subagent = if let Some(ref p_id) = parent_id {
            self.agents.contains_key(p_id)
        } else {
            false
        };
        if let Some(ref p_id) = parent_id
            && !is_subagent {
                // Usually a bare name passed where the full agent ID is needed.
                eprintln!("[daemon] Parent {} is not registered; {} is not linked to it", p_id.0, agent_id.0);
            }

        let mark = agent_id.to_mark();
        // Re-registration keeps an existing window binding and status (G79).
        let (container_id, status) = match self.agents.get(&agent_id) {
            Some(old) => (old.container_id, old.status.clone()),
            None => (None, AgentStatus::Idle),
        };
        let agent = AgentWindow {
            id: agent_id.clone(),
            project: project_id,
            parent_id,
            container_id,
            app_id,
            mark,
            status,
        };
        self.agents.insert(agent_id.clone(), agent);
        self.pid_to_agent.insert(pid, agent_id.clone());

        if let Some(ref hook) = self.config.hooks.on_agent_spawn {
            execute_lifecycle_hook(hook, &agent_id, "idle");
        }

        self.auto_monitor_wanted(is_subagent)
            .then_some(MonitorRequest { agent_id })
    }

    /// The one auto-monitor rule (D3). `subagent_evidence` is true when a parent is known,
    /// either registered explicitly or found by process-tree auditing.
    fn auto_monitor_wanted(&self, subagent_evidence: bool) -> bool {
        subagent_evidence && self.config.daemon.auto_monitor
    }

    /// Window-created rules: the monitor-window route, the agent-window bind, process-tree
    /// auditing and the auto-monitor decision. Returns the layout actions and monitor requests,
    /// in the order the loop should run them (monitors first).
    pub fn on_window_created(
        &mut self,
        con_id: i64,
        app_id: &str,
        pid: Option<i32>,
    ) -> (Vec<LayoutAction>, Vec<MonitorRequest>) {
        let mut actions = Vec::new();
        let mut monitors = Vec::new();

        if let Some(target) = app_id.strip_prefix("agent.monitor.") {
            let target_id = AgentId(target.to_string());
            if let Some(agent) = self.agents.get(&target_id) {
                println!("[daemon] Sway monitor window created for agent {}: con_id={}", target_id.0, con_id);
                actions.push(LayoutAction::WindowCreated {
                    con_id,
                    app_id: app_id.to_string(),
                    project_id: agent.project.clone(),
                    parent_id: agent.parent_id.clone(),
                });
            }
            return (actions, monitors);
        }

        if !app_id.starts_with("agent.") {
            return (actions, monitors);
        }
        let Some(agent_id) = register_agent_window(app_id, con_id, self) else {
            return (actions, monitors);
        };

        if self.config.daemon.process_tree_audit_enabled
            && let Some(con_pid) = pid
        {
            let found = match self.agents.get_mut(&agent_id) {
                Some(agent) => apply_process_tree_auditing(
                    agent,
                    con_pid as u32,
                    &self.pid_to_agent,
                    self.config.daemon.max_ancestry_depth,
                ),
                None => false,
            };
            if self.auto_monitor_wanted(found) {
                monitors.push(MonitorRequest { agent_id: agent_id.clone() });
            }
        }

        // Read parent_id after auditing so an auto-detected parent reaches the layout on this event.
        if let Some(agent) = self.agents.get(&agent_id) {
            println!("[daemon] Sway window created for agent {}: con_id={}", agent_id.0, con_id);
            actions.push(LayoutAction::WindowCreated {
                con_id,
                app_id: app_id.to_string(),
                project_id: agent.project.clone(),
                parent_id: agent.parent_id.clone(),
            });
        }
        (actions, monitors)
    }

    /// Window-destroyed rules: unbind the window, remove the agent, promote its children to
    /// roots, and return the layout action. Returns nothing when no agent owned the window,
    /// or when the agent has since been rebound to another window (stale mapping only dropped).
    pub fn on_window_destroyed(&mut self, con_id: i64) -> Vec<LayoutAction> {
        let Some(agent_id) = self.con_to_agent.remove(&con_id) else {
            return Vec::new();
        };
        if self.agents.get(&agent_id).and_then(|a| a.container_id).is_some_and(|c| c != con_id) {
            return Vec::new();
        }
        println!("[daemon] Sway window destroyed for agent: {}", agent_id.0);
        let parent_id = self.agents.get(&agent_id).and_then(|a| a.parent_id.clone());
        if self.agents.remove(&agent_id).is_some() {
            self.pid_to_agent.retain(|_, id| id != &agent_id);
        }
        promote_orphans(self, &agent_id);
        vec![LayoutAction::WindowDestroyed { agent_id, parent_id }]
    }

    pub fn spawn_monitor_terminal(&self, agent_id: &AgentId) {
        let monitor_app_id = format!("agent.monitor.{}", agent_id.0);
        let monitor_title = format!("Monitor: {}", agent_id.0);
        let logs_dir = match crate::cli::swayve_runtime_dir() {
            Ok(d) => d.join("logs"),
            Err(e) => {
                eprintln!("[daemon] Cannot spawn monitor terminal: {}", e);
                return;
            }
        };
        let monitor_cmd = format!("tail -f {}/{}.log", logs_dir.display(), agent_id.0);

        let mut term_args = Vec::new();
        for arg in &self.config.terminal.args {
            if arg == "{command}" {
                if let Some(split_cmds) = shlex::split(&monitor_cmd) {
                    for sub_arg in split_cmds {
                        term_args.push(sub_arg);
                    }
                } else {
                    term_args.push(monitor_cmd.clone());
                }
            } else {
                let replaced = arg
                    .replace("{app_id}", &monitor_app_id)
                    .replace("{title}", &monitor_title);
                term_args.push(replaced);
            }
        }

        if let Err(e) = std::process::Command::new(&self.config.terminal.default_emulator)
            .args(&term_args)
            .spawn()
        {
            eprintln!(
                "[daemon] Failed to spawn monitor terminal for agent {}: {:?}",
                agent_id.0, e
            );
        }
    }

    pub fn update_status(&mut self, agent_id: &AgentId, status: AgentStatus) -> Option<AgentWindow> {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.status = status;
            let status_str = agent_status_to_str(&agent.status);
            if let Some(ref hook) = self.config.hooks.on_agent_status_change {
                execute_lifecycle_hook(hook, agent_id, status_str);
            }
            Some(agent.clone())
        } else {
            None
        }
    }

    pub fn get_border_color(&self, status: &AgentStatus) -> String {
        match status {
            AgentStatus::Idle => self.config.border_colors.idle.clone(),
            AgentStatus::Thinking => self.config.border_colors.thinking.clone(),
            AgentStatus::ExecutingTool => self.config.border_colors.executing_tool.clone(),
            AgentStatus::AwaitingHumanInput => self.config.border_colors.awaiting_human_input.clone(),
            AgentStatus::Completed => self.config.border_colors.completed.clone(),
            AgentStatus::Failed(_) => self.config.border_colors.failed.clone(),
        }
    }
}

fn agent_status_to_str(status: &AgentStatus) -> &'static str {
    match status {
        AgentStatus::Idle => "idle",
        AgentStatus::Thinking => "thinking",
        AgentStatus::ExecutingTool => "executing_tool",
        AgentStatus::AwaitingHumanInput => "awaiting_human_input",
        AgentStatus::Completed => "completed",
        AgentStatus::Failed(_) => "failed",
    }
}

fn execute_lifecycle_hook(hook_cmd: &str, agent_id: &AgentId, status: &str) {
    if let Err(e) = std::process::Command::new("sh")
        .arg("-c")
        .arg(hook_cmd)
        .env("SWAYVE_AGENT_ID", &agent_id.0)
        .env("SWAYVE_AGENT_STATUS", status)
        .spawn()
    {
        eprintln!(
            "[daemon] Failed to execute lifecycle hook '{}': {:?}",
            hook_cmd, e
        );
    }
}

impl AgentRegistry for DaemonState {
    fn agents(&self) -> &HashMap<AgentId, AgentWindow> {
        &self.agents
    }
}

/// A request for the loop to spawn a monitor terminal for an agent (G54). `DaemonState` decides;
/// the loop executes.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorRequest {
    pub agent_id: AgentId,
}

pub enum DaemonEvent {
    AgentIPC(AgentIPCMessage),
    Sway(SwayEvent),
    Focus,
    Subscribe(mpsc::Sender<String>),
    SubscribeDiff(mpsc::Sender<String>),
    /// Internal barrier: the loop replies once every earlier event has been applied. Not on the wire.
    Sync(tokio::sync::oneshot::Sender<()>),
}

pub(crate) async fn prepare_socket_path(path: &Path, expected_uid: u32) -> Result<()> {
    match fs::symlink_metadata(path).await {
        Ok(md) => {
            use std::os::unix::fs::{FileTypeExt, MetadataExt};
            if md.file_type().is_socket() && md.uid() == expected_uid {
                fs::remove_file(path).await?;
            } else {
                anyhow::bail!(
                    "{} exists and is not a socket owned by the current user; refusing to remove it",
                    path.display()
                );
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

pub async fn start_daemon_ipc(
    socket_path: &str,
    event_tx: mpsc::Sender<DaemonEvent>,
) -> Result<()> {
    start_daemon_ipc_with_limits(socket_path, event_tx, crate::ratelimit::IpcLimits::default()).await
}

pub async fn start_daemon_ipc_with_limits(
    socket_path: &str,
    event_tx: mpsc::Sender<DaemonEvent>,
    limits: crate::ratelimit::IpcLimits,
) -> Result<()> {
    let path = Path::new(socket_path);
    prepare_socket_path(path, crate::cli::current_uid()?).await?;

    let listener = UnixListener::bind(path)?;
    println!("[daemon] Listening on Unix socket: {}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let event_tx = event_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, event_tx, limits).await {
                        eprintln!("[daemon] Error handling client: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[daemon] Connection accept error: {:?}", e);
            }
        }
    }
}

async fn handle_client(
    stream: UnixStream,
    event_tx: mpsc::Sender<DaemonEvent>,
    limits: crate::ratelimit::IpcLimits,
) -> Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer_opt = Some(writer);
    let mut buf = Vec::new();
    let mut limiter = crate::ratelimit::RateLimiter::new(
        limits.lines_per_sec,
        limits.burst,
        std::time::Instant::now(),
    );

    loop {
        buf.clear();
        if reader.read_until(b'\n', &mut buf).await? == 0 {
            break;
        }
        if !limiter.allow(std::time::Instant::now()) {
            eprintln!("[daemon] Rate limit exceeded; dropping IPC line");
            continue;
        }
        let Ok(line) = std::str::from_utf8(&buf) else {
            eprintln!("[daemon] Skipping non-UTF-8 IPC line");
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<AgentIPCMessage>(line) {
            // Focus is a daemon-level request, not agent bookkeeping.
            Ok(AgentIPCMessage::Focus) => {
                let _ = event_tx.send(DaemonEvent::Focus).await;
            }
            Ok(AgentIPCMessage::SubscribeStatus) => {
                let (sub_tx, mut sub_rx) = mpsc::channel::<String>(32);
                if event_tx.send(DaemonEvent::Subscribe(sub_tx)).await.is_err() {
                    return Ok(());
                }
                if let Some(mut writer) = writer_opt.take() {
                    tokio::spawn(async move {
                        use tokio::io::AsyncWriteExt;
                        while let Some(msg) = sub_rx.recv().await {
                            if writer.write_all(msg.as_bytes()).await.is_err()
                                || writer.write_all(b"\n").await.is_err()
                            {
                                break;
                            }
                        }
                    });
                }
            }
            Ok(AgentIPCMessage::SubscribeDiff) => {
                let (sub_tx, mut sub_rx) = mpsc::channel::<String>(32);
                if event_tx.send(DaemonEvent::SubscribeDiff(sub_tx)).await.is_err() {
                    return Ok(());
                }
                if let Some(mut writer) = writer_opt.take() {
                    tokio::spawn(async move {
                        use tokio::io::AsyncWriteExt;
                        while let Some(msg) = sub_rx.recv().await {
                            if writer.write_all(msg.as_bytes()).await.is_err()
                                || writer.write_all(b"\n").await.is_err()
                            {
                                break;
                            }
                        }
                    });
                }
            }
            Ok(msg) => {
                let _ = event_tx.send(DaemonEvent::AgentIPC(msg)).await;
            }
            Err(e) => {
                eprintln!("[daemon] Failed to parse IPC message: {:?}. Line: {}", e, line);
            }
        }
    }

    Ok(())
}

pub async fn run_daemon_loops(
    _dry_run: bool,
    config: Config,
    _socket_path: &str,
    mut event_rx: mpsc::Receiver<DaemonEvent>,
    sway_cmd_tx: mpsc::Sender<SwayCommand>,
) -> Result<()> {
    let mut state = DaemonState::new(config);
    let layout_manager = LayoutManager::new();
    let mut focus = FocusCycle::default();

    while let Some(event) = event_rx.recv().await {
        match event {
            DaemonEvent::AgentIPC(msg) => {
                match msg {
                    AgentIPCMessage::Register { id: _, params } => {
                        println!("[daemon] Registering agent: {} (parent: {:?})", params.agent_id.0, params.parent_agent_id);
                        let monitor = state.register_agent(
                            params.agent_id.clone(),
                            params.project_id.clone(),
                            params.pid,
                            params.app_id.clone(),
                            params.parent_agent_id.clone(),
                        );
                        if let Some(req) = monitor {
                            state.spawn_monitor_terminal(&req.agent_id);
                        }
                        state.broadcast_status();
                    }
                    AgentIPCMessage::UpdateStatus { params } => {
                        println!(
                            "[daemon] Agent {} status update: {:?}",
                            params.agent_id.0, params.status
                        );
                        if let Some(agent) = state.update_status(&params.agent_id, params.status.clone()) {
                            let color = state.get_border_color(&params.status);
                            let _ = sway_cmd_tx
                                .send(SwayCommand::SetBorderColor {
                                    mark: agent.mark.clone(),
                                    hex_color: color,
                                })
                                .await;

                            if params.status == AgentStatus::AwaitingHumanInput {
                                let _ = sway_cmd_tx
                                    .send(SwayCommand::FocusWindow {
                                        mark: agent.mark.clone(),
                                    })
                                    .await;
                            }
                        }
                        state.broadcast_status();
                    }
                    // Routed to DaemonEvent::Focus by handle_client; never arrives here.
                    AgentIPCMessage::Focus => {}
                    // Routed to DaemonEvent::Subscribe by handle_client; never arrives here.
                    AgentIPCMessage::SubscribeStatus => {}
                    // Routed to DaemonEvent::SubscribeDiff by handle_client; never arrives here.
                    AgentIPCMessage::SubscribeDiff => {}
                }
            }
            DaemonEvent::Sway(sway_event) => {
                match sway_event {
                    SwayEvent::WindowCreated { con_id, app_id, pid } => {
                        let (actions, monitors) = state.on_window_created(con_id, &app_id, pid);
                        for req in monitors {
                            state.spawn_monitor_terminal(&req.agent_id);
                        }
                        for action in actions {
                            delegate_to_layout(action, &layout_manager, &state, &sway_cmd_tx).await;
                        }
                    }
                    SwayEvent::WindowDestroyed { con_id } => {
                        let actions = state.on_window_destroyed(con_id);
                        if !actions.is_empty() {
                            for action in actions {
                                delegate_to_layout(action, &layout_manager, &state, &sway_cmd_tx).await;
                            }
                            state.broadcast_status();
                        }
                    }
                    SwayEvent::WindowFocused { con_id } => {
                        // Manual focus on an agent window moves the cycle; other windows are ignored.
                        if let Some(agent_id) = state.con_to_agent.get(&con_id) {
                            focus.observe(agent_id.clone());
                        }
                    }
                }
            }
            DaemonEvent::Focus => {
                // Focus event handler: hierarchical sibling cycling
                if let Some(target) = focus.next(&state.agents)
                    && let Some(agent) = state.agents.get(&target) {
                        let _ = sway_cmd_tx
                            .send(SwayCommand::FocusWindow {
                                mark: agent.mark.clone(),
                            })
                            .await;
                    }
            }
            DaemonEvent::Subscribe(sub_tx) => {
                state.push_full_subscriber(sub_tx);
                state.broadcast_status();
            }
            DaemonEvent::SubscribeDiff(sub_tx) => {
                state.push_diff_subscriber(sub_tx);
                state.broadcast_status();
            }
            DaemonEvent::Sync(done) => {
                let _ = done.send(());
            }
        }
    }

    Ok(())
}

/// Detach the children of a destroyed agent so they become roots.
/// Left alone they keep pointing at an id that is no longer registered, and
/// focus traversal then returns a target that cannot be resolved.
fn promote_orphans(state: &mut DaemonState, destroyed: &AgentId) {
    for agent in state.agents.values_mut() {
        if agent.parent_id.as_ref() == Some(destroyed) {
            agent.parent_id = None;
        }
    }
}

/// Bind a newly created Sway window to its registered agent.
/// Sets the agent's `container_id` and records the reverse mapping.
/// Returns the agent's id, or `None` when no agent claims this `app_id`.
fn register_agent_window(app_id: &str, con_id: i64, state: &mut DaemonState) -> Option<AgentId> {
    let agent = state.agents.values_mut().find(|a| a.app_id == app_id)?;
    let old = agent.container_id.replace(con_id);
    let agent_id = agent.id.clone();
    if let Some(old) = old.filter(|&o| o != con_id) {
        state.con_to_agent.remove(&old);
    }
    state.con_to_agent.insert(con_id, agent_id.clone());
    Some(agent_id)
}

/// Fill in a missing `parent_id` by walking the process tree for a registered ancestor.
/// No-op when the agent already knows its parent. Returns whether a parent was detected.
fn apply_process_tree_auditing(
    agent: &mut AgentWindow,
    con_pid: u32,
    pid_to_agent: &HashMap<u32, AgentId>,
    max_depth: usize,
) -> bool {
    if agent.parent_id.is_some() {
        return false;
    }

    match find_registered_parent_ancestor(con_pid, pid_to_agent, max_depth, get_parent_pid) {
        Some(ancestor_id) => {
            println!(
                "[daemon] Auto-detected parent agent {} for child {} via process tree auditing (PID={})",
                ancestor_id.0, agent.id.0, con_pid
            );
            agent.parent_id = Some(ancestor_id);
            true
        }
        None => false,
    }
}

/// Hand a layout action to the LayoutManager and forward its commands to Sway.
async fn delegate_to_layout(
    action: LayoutAction,
    layout_manager: &LayoutManager,
    state: &DaemonState,
    sway_cmd_tx: &mpsc::Sender<SwayCommand>,
) {
    for cmd in layout_manager.compute_commands(action, state) {
        let _ = sway_cmd_tx.send(cmd).await;
    }
}

/// The focus cycle's cursor: the last agent focused by the cycle or by the user (G58).
#[derive(Debug, Default)]
pub struct FocusCycle {
    last: Option<AgentId>,
}

impl FocusCycle {
    /// Records a manual focus on an agent window.
    pub fn observe(&mut self, agent: AgentId) {
        self.last = Some(agent);
    }

    /// Returns the next target and moves the cursor to it. The cursor moves only when a target
    /// is found; otherwise it is unchanged (D4).
    pub fn next(&mut self, agents: &HashMap<AgentId, AgentWindow>) -> Option<AgentId> {
        let target = next_focus_target(agents, self.last.as_ref())?;
        self.last = Some(target.clone());
        Some(target)
    }
}

/// Compute the next window to focus, stepping through the agent hierarchy in
/// pre-order: each root, then its children, then the next root, wrapping at the
/// end. Only agents with a `container_id` take part, ordered by `con_id`.
/// Returns `None` when there is nothing to focus or nowhere to move.
fn next_focus_target(
    agents: &HashMap<AgentId, AgentWindow>,
    last_focused: Option<&AgentId>,
) -> Option<AgentId> {
    let mut candidates: Vec<&AgentWindow> =
        agents.values().filter(|a| a.container_id.is_some()).collect();
    candidates.sort_by_key(|a| a.container_id);

    let mut order: Vec<&AgentWindow> = Vec::with_capacity(candidates.len());
    for root in candidates.iter().copied().filter(|a| a.parent_id.is_none()) {
        order.push(root);
        order.extend(
            candidates
                .iter()
                .copied()
                .filter(|a| a.parent_id.as_ref() == Some(&root.id)),
        );
    }
    // A child whose parent has no window yet belongs to no root above, and would
    // otherwise drop out of the cycle entirely.
    let dangling: Vec<&AgentWindow> = candidates
        .iter()
        .copied()
        .filter(|a| !order.iter().any(|o| o.id == a.id))
        .collect();
    order.extend(dangling);

    match last_focused.and_then(|id| order.iter().position(|a| &a.id == id)) {
        // Somewhere in the cycle: step forward, or stay put when alone.
        Some(_) if order.len() < 2 => None,
        Some(idx) => Some(order[(idx + 1) % order.len()].id.clone()),
        // Nothing focused, or the focused agent is gone: start at the top.
        None => order.first().map(|a| a.id.clone()),
    }
}

fn get_parent_pid(pid: u32) -> Option<u32> {
    let stat_path = format!("/proc/{}/stat", pid);
    if let Ok(contents) = std::fs::read_to_string(stat_path)
        && let Some(r_paren) = contents.rfind(')') {
            let rest = &contents[r_paren + 1..];
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() >= 2
                && let Ok(ppid) = fields[1].parse::<u32>() {
                    return Some(ppid);
                }
        }
    None
}

/// Walks up from `start_pid` looking for a registered agent. `parent_of` gives a process's
/// parent pid; production passes `get_parent_pid` (`/proc`), tests pass a fake chain.
fn find_registered_parent_ancestor(
    start_pid: u32,
    pid_to_agent: &HashMap<u32, AgentId>,
    max_depth: usize,
    parent_of: impl Fn(u32) -> Option<u32>,
) -> Option<AgentId> {
    let mut current_pid = start_pid;
    let mut depth = 0;
    while current_pid > 1 && depth < max_depth {
        if let Some(ppid) = parent_of(current_pid) {
            if let Some(agent_id) = pid_to_agent.get(&ppid) {
                return Some(agent_id.clone());
            }
            current_pid = ppid;
            depth += 1;
        } else {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stale_socket_removal_refuses_regular_file() {
        let p = std::env::temp_dir().join(format!("swayve_regular_{}", std::process::id()));
        std::fs::write(&p, b"x").unwrap();
        let (tx, _rx) = mpsc::channel(1);
        let r = start_daemon_ipc(p.to_str().unwrap(), tx).await;
        let still = p.exists();
        let _ = std::fs::remove_file(&p);
        assert!(r.is_err());
        assert!(still);
    }

    #[tokio::test]
    async fn test_stale_socket_same_owner_is_replaced() {
        use std::os::unix::fs::FileTypeExt;
        let p = std::env::temp_dir().join(format!("swayve_stale_{}", std::process::id()));
        let _ = std::fs::remove_file(&p);
        drop(std::os::unix::net::UnixListener::bind(&p).unwrap());
        let (tx, _rx) = mpsc::channel(1);
        let path = p.to_str().unwrap().to_string();
        let task = tokio::spawn(async move { start_daemon_ipc(&path, tx).await });
        // A stale socket refuses connections, so a successful connect proves replacement.
        let mut replaced = false;
        for _ in 0..100 {
            let is_sock = std::fs::symlink_metadata(&p)
                .map(|m| m.file_type().is_socket())
                .unwrap_or(false);
            if is_sock && std::os::unix::net::UnixStream::connect(&p).is_ok() {
                replaced = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        task.abort();
        let _ = std::fs::remove_file(&p);
        assert!(replaced);
    }

    #[tokio::test]
    async fn test_configured_limit_reaches_handle_client() {
        use tokio::io::AsyncWriteExt;
        let (mut client, server) = UnixStream::pair().unwrap();
        let (event_tx, mut event_rx) = mpsc::channel(8);
        let limits = crate::ratelimit::IpcLimits { lines_per_sec: 1.0, burst: 2.0 };
        tokio::spawn(handle_client(server, event_tx, limits));
        // Burst of 2: the first two focus lines pass, the third is dropped.
        for _ in 0..3 {
            client.write_all(b"{\"method\":\"daemon.focus\"}\n").await.unwrap();
        }
        client.flush().await.unwrap();
        let mut focus_events = 0;
        while let Ok(Some(DaemonEvent::Focus)) = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            event_rx.recv(),
        )
        .await
        {
            focus_events += 1;
        }
        assert_eq!(focus_events, 2);
    }

    #[tokio::test]
    async fn test_malformed_ipc_line_keeps_client() {
        use tokio::io::AsyncWriteExt;
        let (mut client, server) = UnixStream::pair().unwrap();
        let (event_tx, mut event_rx) = mpsc::channel(4);
        tokio::spawn(handle_client(server, event_tx, crate::ratelimit::IpcLimits::default()));
        client.write_all(b"not json\n").await.unwrap();
        client.write_all(b"{\"method\":\"daemon.focus\"}\n").await.unwrap();
        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), event_rx.recv())
            .await
            .expect("timed out waiting for event");
        assert!(matches!(ev, Some(DaemonEvent::Focus)));
    }

    #[tokio::test]
    async fn test_non_utf8_ipc_line_is_skipped() {
        use tokio::io::AsyncWriteExt;
        let (mut client, server) = UnixStream::pair().unwrap();
        let (event_tx, mut event_rx) = mpsc::channel(4);
        tokio::spawn(handle_client(server, event_tx, crate::ratelimit::IpcLimits::default()));
        client.write_all(b"\xff\xfe\n").await.unwrap();
        client.write_all(b"{\"method\":\"daemon.focus\"}\n").await.unwrap();
        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), event_rx.recv())
            .await
            .expect("timed out waiting for event");
        assert!(matches!(ev, Some(DaemonEvent::Focus)));
    }

    #[tokio::test]
    async fn test_prepare_socket_path_refuses_foreign_uid() {
        let p = std::env::temp_dir().join(format!("swayve_foreign_{}", std::process::id()));
        let _ = std::fs::remove_file(&p);
        drop(std::os::unix::net::UnixListener::bind(&p).unwrap());
        let uid = crate::cli::current_uid().unwrap();
        let r = prepare_socket_path(&p, uid.wrapping_add(1)).await;
        let still = p.exists();
        let _ = std::fs::remove_file(&p);
        assert!(r.is_err());
        assert!(still);
    }

    #[tokio::test]
    async fn test_prepare_socket_path_removes_matching_socket() {
        let p = std::env::temp_dir().join(format!("swayve_match_{}", std::process::id()));
        let _ = std::fs::remove_file(&p);
        drop(std::os::unix::net::UnixListener::bind(&p).unwrap());
        let r = prepare_socket_path(&p, crate::cli::current_uid().unwrap()).await;
        let gone = !p.exists();
        let _ = std::fs::remove_file(&p);
        assert!(r.is_ok());
        assert!(gone);
    }

    #[test]
    fn test_early_termination_at_depth_limit() {
        // Chain: pid 100 -> 101 -> ... -> 110 (each pid's parent is pid + 1).
        // The registered agent is at pid 105, five steps above pid 100.
        let parent_of = |p: u32| if (100..110).contains(&p) { Some(p + 1) } else { None };
        let mut pid_to_agent = HashMap::new();
        let agent_id = AgentId("test_agent".to_string());
        pid_to_agent.insert(105, agent_id.clone());

        // A limit of 3 stops before the agent is reached.
        assert_eq!(find_registered_parent_ancestor(100, &pid_to_agent, 3, parent_of), None);
        // A limit of 10 reaches it.
        assert_eq!(find_registered_parent_ancestor(100, &pid_to_agent, 10, parent_of), Some(agent_id));
    }

    #[test]
    fn test_process_tree_audit_disabled() {
        // Test that when process_tree_audit_enabled is false, scanning is skipped
        let mut config = Config::default();
        config.daemon.process_tree_audit_enabled = false;

        // Verify the config is set correctly
        assert!(!config.daemon.process_tree_audit_enabled);
        assert_eq!(config.daemon.max_ancestry_depth, 32);
    }

    #[test]
    fn test_focus_ipc_roundtrip() {
        // The CLI writes this line; handle_client must route it to DaemonEvent::Focus.
        let wire = serde_json::to_string(&AgentIPCMessage::Focus).unwrap();
        assert_eq!(wire, r#"{"method":"daemon.focus"}"#);
        assert!(matches!(
            serde_json::from_str::<AgentIPCMessage>(&wire).unwrap(),
            AgentIPCMessage::Focus
        ));
    }

    fn mock_agent(id: &str, parent: Option<&str>, con_id: Option<i64>) -> AgentWindow {
        let agent_id = AgentId(id.to_string());
        AgentWindow {
            mark: agent_id.to_mark(),
            id: agent_id,
            project: ProjectId("proj".to_string()),
            parent_id: parent.map(|p| AgentId(p.to_string())),
            container_id: con_id,
            app_id: format!("agent.{}", id),
            status: AgentStatus::Idle,
        }
    }

    fn registry(agents: Vec<AgentWindow>) -> HashMap<AgentId, AgentWindow> {
        agents.into_iter().map(|a| (a.id.clone(), a)).collect()
    }

    #[test]
    fn test_focus_cycle_next_moves_cursor() {
        let flat = registry(vec![
            mock_agent("solo_1", None, Some(1)),
            mock_agent("solo_2", None, Some(2)),
        ]);
        let mut focus = FocusCycle::default();
        focus.observe(AgentId("solo_1".to_string()));
        assert_eq!(focus.next(&flat), Some(AgentId("solo_2".to_string())));
        assert_eq!(focus.next(&flat), Some(AgentId("solo_1".to_string())));
    }

    #[test]
    fn test_focus_cycle_next_none_keeps_cursor() {
        let empty: HashMap<AgentId, AgentWindow> = HashMap::new();
        let mut focus = FocusCycle::default();
        focus.observe(AgentId("ghost".to_string()));
        assert_eq!(focus.next(&empty), None);
        assert_eq!(focus.last, Some(AgentId("ghost".to_string())));
    }

    #[test]
    fn test_focus_cycle_next_sets_cursor_to_target() {
        let flat = registry(vec![mock_agent("solo_1", None, Some(1))]);
        let mut focus = FocusCycle::default();
        assert_eq!(focus.next(&flat), Some(AgentId("solo_1".to_string())));
        assert_eq!(focus.last, Some(AgentId("solo_1".to_string())));
    }

    #[test]
    fn test_hierarchical_focus() {
        // Empty registry: nothing to focus
        assert_eq!(next_focus_target(&HashMap::new(), None), None);

        // Agents without a container_id are not focus candidates
        let unmapped = registry(vec![mock_agent("solo", None, None)]);
        assert_eq!(next_focus_target(&unmapped, None), None);

        // A parent with two children, ordered by con_id
        let agents = registry(vec![
            mock_agent("parent", None, Some(1)),
            mock_agent("child_a", Some("parent"), Some(2)),
            mock_agent("child_b", Some("parent"), Some(3)),
        ]);
        let parent = AgentId("parent".to_string());
        let child_a = AgentId("child_a".to_string());
        let child_b = AgentId("child_b".to_string());

        // Nothing focused yet: start at the lowest con_id
        assert_eq!(next_focus_target(&agents, None), Some(parent.clone()));

        // A stale focus target that is no longer registered falls back to the first
        let stale = AgentId("gone".to_string());
        assert_eq!(next_focus_target(&agents, Some(&stale)), Some(parent.clone()));

        // Parent descends into its first child, then cycles siblings, then wraps to parent
        assert_eq!(next_focus_target(&agents, Some(&parent)), Some(child_a.clone()));
        assert_eq!(next_focus_target(&agents, Some(&child_a)), Some(child_b.clone()));
        assert_eq!(next_focus_target(&agents, Some(&child_b)), Some(parent.clone()));

        // A lone agent has nowhere to go
        let orphan = registry(vec![mock_agent("orphan", None, Some(1))]);
        let orphan_id = AgentId("orphan".to_string());
        assert_eq!(next_focus_target(&orphan, Some(&orphan_id)), None);
        // ...but an unfocused lone agent is still worth focusing
        assert_eq!(next_focus_target(&orphan, None), Some(orphan_id));
    }

    #[test]
    fn test_promote_orphans() {
        let mut state = DaemonState::new(Config::default());
        let parent = AgentId("parent".to_string());
        let child = AgentId("child".to_string());
        state.register_agent(parent.clone(), ProjectId("p".to_string()), 1, "agent.parent".to_string(), None);
        state.register_agent(child.clone(), ProjectId("p".to_string()), 2, "agent.child".to_string(), Some(parent.clone()));

        // The parent's window dies and it leaves the registry
        state.agents.remove(&parent);
        promote_orphans(&mut state, &parent);

        // The child must not be left pointing at an agent that no longer exists
        assert_eq!(state.agents[&child].parent_id, None);
    }

    #[test]
    fn test_focus_cycles_between_roots() {
        // Unrelated agents with no hierarchy: the ordinary case
        let flat = registry(vec![
            mock_agent("solo_1", None, Some(1)),
            mock_agent("solo_2", None, Some(2)),
        ]);
        let solo_1 = AgentId("solo_1".to_string());
        let solo_2 = AgentId("solo_2".to_string());
        assert_eq!(next_focus_target(&flat, Some(&solo_1)), Some(solo_2.clone()));
        assert_eq!(next_focus_target(&flat, Some(&solo_2)), Some(solo_1.clone()));

        // A root's subtree is traversed before moving on to the next root
        let mixed = registry(vec![
            mock_agent("root_a", None, Some(1)),
            mock_agent("kid", Some("root_a"), Some(2)),
            mock_agent("root_b", None, Some(3)),
        ]);
        let root_a = AgentId("root_a".to_string());
        let kid = AgentId("kid".to_string());
        let root_b = AgentId("root_b".to_string());
        assert_eq!(next_focus_target(&mixed, Some(&root_a)), Some(kid.clone()));
        assert_eq!(next_focus_target(&mixed, Some(&kid)), Some(root_b.clone()));
        assert_eq!(next_focus_target(&mixed, Some(&root_b)), Some(root_a.clone()));
    }

    #[test]
    fn test_lifecycle_hooks_execution() {
        let out_path = std::env::temp_dir().join(format!("swayve-{}-lifecycle-hook.txt", std::process::id())).to_string_lossy().into_owned();
        let _ = std::fs::remove_file(&out_path);

        let mut config = Config::default();
        config.hooks.on_agent_spawn = Some(format!("echo \"spawn $SWAYVE_AGENT_ID $SWAYVE_AGENT_STATUS\" >> {}", out_path));
        config.hooks.on_agent_status_change = Some(format!("echo \"status $SWAYVE_AGENT_ID $SWAYVE_AGENT_STATUS\" >> {}", out_path));

        let mut state = DaemonState::new(config);
        let agent_id = AgentId("test_hook_agent".to_string());

        state.register_agent(
            agent_id.clone(),
            ProjectId("proj".to_string()),
            123,
            "agent.test".to_string(),
            None,
        );

        state.update_status(&agent_id, AgentStatus::Thinking);

        // Wait for background hooks to finish execution
        std::thread::sleep(std::time::Duration::from_millis(200));

        let contents = std::fs::read_to_string(&out_path).unwrap_or_default();
        let _ = std::fs::remove_file(&out_path);

        assert!(contents.contains("spawn test_hook_agent idle"));
        assert!(contents.contains("status test_hook_agent thinking"));
    }

    #[test]
    fn test_auto_monitor_subagent_registration() {
        let mut config = Config::default();
        config.daemon.auto_monitor = true;
        config.terminal.default_emulator = "true".to_string();

        let mut state = DaemonState::new(config);
        let parent = AgentId("parent_agent".to_string());
        let child = AgentId("child_agent".to_string());
        let project = ProjectId("test_project".to_string());

        // Register parent agent (auto_monitor should not trigger for root agent)
        state.register_agent(
            parent.clone(),
            project.clone(),
            100,
            "agent.parent".to_string(),
            None,
        );

        // Register child agent under active parent (auto_monitor should trigger)
        state.register_agent(
            child.clone(),
            project.clone(),
            101,
            "agent.child".to_string(),
            Some(parent.clone()),
        );

        assert!(state.agents.contains_key(&child));
    }

    #[test]
    fn test_auto_monitor_request_only_for_subagent() {
        let mut config = Config::default();
        config.daemon.auto_monitor = true;
        let mut state = DaemonState::new(config);
        let parent = AgentId("p".to_string());
        let child = AgentId("c".to_string());
        let project = ProjectId("proj".to_string());
        let root = state.register_agent(parent.clone(), project.clone(), 1, "agent.p".to_string(), None);
        assert_eq!(root, None);
        let sub = state.register_agent(child.clone(), project, 2, "agent.c".to_string(), Some(parent));
        assert_eq!(sub, Some(MonitorRequest { agent_id: child }));
    }

    #[test]
    fn test_auto_monitor_off_gives_no_request() {
        let mut state = DaemonState::new(Config::default());
        let parent = AgentId("p".to_string());
        let project = ProjectId("proj".to_string());
        state.register_agent(parent.clone(), project.clone(), 1, "agent.p".to_string(), None);
        let sub = state.register_agent(AgentId("c".to_string()), project, 2, "agent.c".to_string(), Some(parent));
        assert_eq!(sub, None);
    }

    #[test]
    fn test_window_destroyed_promotes_children() {
        let mut state = DaemonState::new(Config::default());
        let project = ProjectId("proj".to_string());
        let parent = AgentId("p".to_string());
        let child = AgentId("c".to_string());
        state.register_agent(parent.clone(), project.clone(), 1, "agent.p".to_string(), None);
        state.register_agent(child.clone(), project, 2, "agent.c".to_string(), Some(parent.clone()));
        state.con_to_agent.insert(10, parent.clone());
        let actions = state.on_window_destroyed(10);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            LayoutAction::WindowDestroyed { agent_id, parent_id: None } if *agent_id == parent
        ));
        assert_eq!(state.agents.get(&child).unwrap().parent_id, None);
    }

    #[test]
    fn test_reregister_keeps_container_binding() {
        let mut state = DaemonState::new(Config::default());
        let a = AgentId("a".to_string());
        state.register_agent(a.clone(), ProjectId("p1".to_string()), 1, "agent.a".to_string(), None);
        assert_eq!(register_agent_window("agent.a", 10, &mut state), Some(a.clone()));
        state.agents.get_mut(&a).unwrap().status = AgentStatus::Thinking;

        state.register_agent(a.clone(), ProjectId("p2".to_string()), 2, "agent.a".to_string(), None);

        let agent = state.agents.get(&a).unwrap();
        assert_eq!(agent.container_id, Some(10));
        assert_eq!(state.con_to_agent.get(&10), Some(&a));
        assert_eq!(agent.status, AgentStatus::Thinking);
        assert_eq!(agent.project, ProjectId("p2".to_string()));
        assert_eq!(state.pid_to_agent.get(&2), Some(&a));
    }

    #[test]
    fn test_rebind_releases_old_container() {
        let mut state = DaemonState::new(Config::default());
        let a = AgentId("a".to_string());
        state.register_agent(a.clone(), ProjectId("p".to_string()), 1, "agent.a".to_string(), None);
        register_agent_window("agent.a", 10, &mut state);
        register_agent_window("agent.a", 20, &mut state);
        assert!(state.on_window_destroyed(10).is_empty());
        assert_eq!(state.agents.get(&a).unwrap().container_id, Some(20));
        assert_eq!(state.con_to_agent.get(&10), None);
        assert_eq!(state.con_to_agent.get(&20), Some(&a));
    }

    #[test]
    fn test_subscribe_status_ipc_serde() {
        let wire = serde_json::to_string(&AgentIPCMessage::SubscribeStatus).unwrap();
        assert_eq!(wire, r#"{"method":"daemon.subscribe"}"#);
        assert!(matches!(
            serde_json::from_str::<AgentIPCMessage>(&wire).unwrap(),
            AgentIPCMessage::SubscribeStatus
        ));
    }

    #[test]
    fn test_daemon_broadcast_status() {
        let mut state = DaemonState::new(Config::default());
        let (tx1, mut rx1) = mpsc::channel(10);
        let (tx2, mut rx2) = mpsc::channel(10);

        state.push_full_subscriber(tx1);
        state.push_full_subscriber(tx2);

        let agent_id = AgentId("test_sub_agent".to_string());
        state.register_agent(
            agent_id.clone(),
            ProjectId("proj".to_string()),
            123,
            "agent.sub".to_string(),
            None,
        );
        state.broadcast_status();

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();
        assert_eq!(msg1, msg2);
        assert!(msg1.contains("test_sub_agent"));

        // Drop rx1 and verify closed subscriber gets pruned on next broadcast
        drop(rx1);
        state.broadcast_status();
        assert_eq!(state.subscribers.len(), 1);
    }

    #[test]
    fn test_daemon_broadcast_drops_newest_when_subscriber_full() {
        let mut state = DaemonState::new(Config::default());
        let (tx, mut rx) = mpsc::channel(32);
        state.push_full_subscriber(tx);

        state.register_agent(
            AgentId("full_sub_agent".to_string()),
            ProjectId("proj".to_string()),
            124,
            "agent.full".to_string(),
            None,
        );

        // Fill the channel, then one more snapshot that must be dropped.
        for _ in 0..32 {
            state.broadcast_status();
        }
        state.broadcast_status();
        assert_eq!(state.subscribers.len(), 1);

        let mut received = 0;
        while rx.try_recv().is_ok() {
            received += 1;
        }
        assert_eq!(received, 32);

        // Subscriber stays registered and receives the next snapshot.
        state.broadcast_status();
        let msg = rx.try_recv().unwrap();
        assert!(msg.contains("full_sub_agent"));
        assert_eq!(state.subscribers.len(), 1);
    }

    #[test]
    fn test_diff_subscriber_gets_snapshot_then_diffs() {
        let mut state = DaemonState::new(Config::default());
        let (tx, mut rx) = mpsc::channel(32);
        state.push_diff_subscriber(tx);
        for n in ["a1", "a2"] {
            state.register_agent(AgentId(n.to_string()), ProjectId("proj".to_string()), 1, format!("app.{n}"), None);
        }
        state.broadcast_status();
        assert!(rx.try_recv().unwrap().contains("\"snapshot\":true"));

        state.update_status(&AgentId("a1".to_string()), AgentStatus::Thinking);
        state.broadcast_status();
        let v: serde_json::Value = serde_json::from_str(&rx.try_recv().unwrap()).unwrap();
        assert_eq!(v["snapshot"], false);
        assert_eq!(v["upsert"].as_array().unwrap().len(), 1);
        assert!(v["upsert"][0].to_string().contains("a1"));
        assert!(v["remove"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_diff_subscriber_recovers_with_snapshot_after_drop() {
        let mut state = DaemonState::new(Config::default());
        let (tx, mut rx) = mpsc::channel(1);
        state.push_diff_subscriber(tx);
        state.register_agent(AgentId("a1".to_string()), ProjectId("proj".to_string()), 1, "app.a1".to_string(), None);
        state.broadcast_status(); // snapshot fills the channel
        state.update_status(&AgentId("a1".to_string()), AgentStatus::Thinking);
        state.broadcast_status(); // diff dropped
        assert!(rx.try_recv().unwrap().contains("\"snapshot\":true"));
        assert!(rx.try_recv().is_err());

        state.broadcast_status();
        assert!(rx.try_recv().unwrap().contains("\"snapshot\":true"));
        assert_eq!(state.subscribers.len(), 1);
    }

    #[test]
    #[ignore]
    fn measure_broadcast_cost() {
        for n in [5usize, 20, 50, 200] {
            for k in [1usize, 10] {
                let mut state = DaemonState::new(Config::default());
                for i in 0..n {
                    state.register_agent(
                        AgentId(format!("agent_{i}")),
                        ProjectId("proj".to_string()),
                        1000 + i as u32,
                        format!("agent.{i}"),
                        None,
                    );
                }
                let mut rxs = Vec::new();
                for _ in 0..k {
                    let (tx, rx) = mpsc::channel(10_000);
                    state.push_full_subscriber(tx);
                    rxs.push(rx);
                }
                let start = std::time::Instant::now();
                for _ in 0..200 {
                    state.broadcast_status();
                }
                let mean_us = start.elapsed().as_secs_f64() * 1e6 / 200.0;
                let payload_bytes = serde_json::to_string(&state.agents.values().collect::<Vec<_>>())
                    .unwrap()
                    .len();
                println!("N={n} K={k} mean_us={mean_us:.1} payload_bytes={payload_bytes}");
                drop(rxs);
            }
        }
    }

    #[test]
    #[ignore]
    fn measure_broadcast_cost_diff() {
        for n in [5usize, 20, 50, 200, 1000] {
            for k in [1usize, 10] {
                let mut state = DaemonState::new(Config::default());
                for i in 0..n {
                    state.register_agent(
                        AgentId(format!("agent_{i}")),
                        ProjectId("proj".to_string()),
                        1000 + i as u32,
                        format!("agent.{i}"),
                        None,
                    );
                }
                let mut rxs = Vec::new();
                for _ in 0..k {
                    let (tx, rx) = mpsc::channel(10_000);
                    state.push_diff_subscriber(tx);
                    rxs.push(rx);
                }
                state.broadcast_status(); // snapshot; fills `sent`
                let id = AgentId("agent_0".to_string());
                let start = std::time::Instant::now();
                for i in 0..200 {
                    let s = if i % 2 == 0 { AgentStatus::Thinking } else { AgentStatus::Idle };
                    state.update_status(&id, s);
                    state.broadcast_status();
                }
                let mean_us = start.elapsed().as_secs_f64() * 1e6 / 200.0;
                let _snapshot = rxs[0].try_recv().unwrap();
                let (mut lines, mut bytes) = (0usize, 0usize);
                while let Ok(l) = rxs[0].try_recv() {
                    lines += 1;
                    bytes += l.len();
                }
                let avg_line_bytes = bytes as f64 / lines as f64;
                println!(
                    "N={n} K={k} mean_us={mean_us:.1} mode=diff lines={lines} avg_line_bytes={avg_line_bytes:.0}"
                );
                drop(rxs);
            }
        }
    }
}
