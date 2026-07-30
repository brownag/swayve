use crate::config::Config;
use crate::layout::{LayoutAction, LayoutManager};
use crate::sway::{SwayCommand, SwayEvent};
use crate::types::{AgentId, AgentIPCMessage, AgentStatus, AgentWindow, ProjectId};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

pub struct DaemonState {
    pub config: Config,
    pub agents: HashMap<AgentId, AgentWindow>,
    pub pid_to_agent: HashMap<u32, AgentId>,
    pub con_to_agent: HashMap<i64, AgentId>,
}

impl DaemonState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            agents: HashMap::new(),
            pid_to_agent: HashMap::new(),
            con_to_agent: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, agent_id: AgentId, project_id: ProjectId, pid: u32, app_id: String, parent_id: Option<AgentId>) {
        let mark = agent_id.to_mark();
        let agent = AgentWindow {
            id: agent_id.clone(),
            project: project_id,
            parent_id,
            container_id: None,
            app_id,
            mark,
            status: AgentStatus::Idle,
        };
        self.agents.insert(agent_id.clone(), agent);
        self.pid_to_agent.insert(pid, agent_id);
    }

    pub fn update_status(&mut self, agent_id: &AgentId, status: AgentStatus) -> Option<AgentWindow> {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.status = status;
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

pub enum DaemonEvent {
    AgentIPC(AgentIPCMessage),
    Sway(SwayEvent),
}

pub async fn start_daemon_ipc(
    socket_path: &str,
    event_tx: mpsc::Sender<DaemonEvent>,
) -> Result<()> {
    let path = Path::new(socket_path);
    if path.exists() {
        fs::remove_file(path).await?;
    }

    let listener = UnixListener::bind(path)?;
    println!("[daemon] Listening on Unix socket: {}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let event_tx = event_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, event_tx).await {
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

async fn handle_client(stream: UnixStream, event_tx: mpsc::Sender<DaemonEvent>) -> Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<AgentIPCMessage>(&line) {
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

    while let Some(event) = event_rx.recv().await {
        match event {
            DaemonEvent::AgentIPC(msg) => {
                match msg {
                    AgentIPCMessage::Register { id: _, params } => {
                        println!("[daemon] Registering agent: {} (parent: {:?})", params.agent_id.0, params.parent_agent_id);
                        state.register_agent(
                            params.agent_id.clone(),
                            params.project_id.clone(),
                            params.pid,
                            params.app_id.clone(),
                            params.parent_agent_id.clone(),
                        );
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
                    }
                }
            }
            DaemonEvent::Sway(sway_event) => {
                match sway_event {
                    SwayEvent::WindowCreated { con_id, app_id, pid } => {
                        if app_id.starts_with("agent.") {
                            // Find the agent matching the app_id, set container_id, and extract details
                            let agent_info = if let Some((_, agent)) = state
                                .agents
                                .iter_mut()
                                .find(|(_, a)| a.app_id == app_id)
                            {
                                agent.container_id = Some(con_id);
                                state.con_to_agent.insert(con_id, agent.id.clone());

                                // Perform dynamic process tree auditing if parent_id is missing
                                if agent.parent_id.is_none() {
                                    if let Some(con_pid) = pid {
                                        if let Some(ancestor_id) = find_registered_parent_ancestor(con_pid as u32, &state.pid_to_agent) {
                                            println!(
                                                "[daemon] Auto-detected parent agent {} for child {} via process tree auditing (PID={})",
                                                ancestor_id.0, agent.id.0, con_pid
                                            );
                                            agent.parent_id = Some(ancestor_id);
                                        }
                                    }
                                }

                                Some((
                                    agent.id.clone(),
                                    agent.mark.clone(),
                                    agent.project.0.clone(),
                                    agent.parent_id.clone(),
                                ))
                            } else {
                                None
                            };

                            if let Some((agent_id, _agent_mark, project_name, parent_id)) = agent_info {
                                println!(
                                    "[daemon] Sway window created for agent {}: con_id={}",
                                    agent_id.0, con_id
                                );

                                // Delegate layout setup to LayoutManager
                                let action = LayoutAction::WindowCreated {
                                    con_id,
                                    app_id: app_id.clone(),
                                    project_id: ProjectId(project_name.clone()),
                                    parent_id: parent_id.clone(),
                                };
                                for cmd in layout_manager.compute_commands(action, &state) {
                                    let _ = sway_cmd_tx.send(cmd).await;
                                }
                            }
                        }
                    }
                    SwayEvent::WindowDestroyed { con_id } => {
                        if let Some(agent_id) = state.con_to_agent.remove(&con_id) {
                            println!("[daemon] Sway window destroyed for agent: {}", agent_id.0);
                            if let Some(_agent) = state.agents.remove(&agent_id) {
                                state.pid_to_agent.retain(|_, id| id != &agent_id);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_parent_pid(pid: u32) -> Option<u32> {
    let stat_path = format!("/proc/{}/stat", pid);
    if let Ok(contents) = std::fs::read_to_string(stat_path) {
        if let Some(r_paren) = contents.rfind(')') {
            let rest = &contents[r_paren + 1..];
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() >= 2 {
                if let Ok(ppid) = fields[1].parse::<u32>() {
                    return Some(ppid);
                }
            }
        }
    }
    None
}

const MAX_ANCESTRY_DEPTH: usize = 32;

fn find_registered_parent_ancestor(
    start_pid: u32,
    pid_to_agent: &HashMap<u32, AgentId>,
) -> Option<AgentId> {
    let mut current_pid = start_pid;
    let mut depth = 0;
    while current_pid > 1 && depth < MAX_ANCESTRY_DEPTH {
        if let Some(ppid) = get_parent_pid(current_pid) {
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
