use crate::config::Config;
use crate::layout::{LayoutAction, LayoutManager};
use crate::sway::{SwayCommand, SwayEvent};
use crate::types::{AgentId, AgentIPCMessage, AgentRegistry, AgentStatus, AgentWindow, ProjectId};
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
        self.pid_to_agent.insert(pid, agent_id.clone());

        if let Some(ref hook) = self.config.hooks.on_agent_spawn {
            execute_lifecycle_hook(hook, &agent_id, "idle");
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

pub enum DaemonEvent {
    AgentIPC(AgentIPCMessage),
    Sway(SwayEvent),
    Focus,
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
            // Focus is a daemon-level request, not agent bookkeeping.
            Ok(AgentIPCMessage::Focus) => {
                let _ = event_tx.send(DaemonEvent::Focus).await;
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
    let mut last_focused: Option<AgentId> = None;

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
                    // Routed to DaemonEvent::Focus by handle_client; never arrives here.
                    AgentIPCMessage::Focus => {}
                }
            }
            DaemonEvent::Sway(sway_event) => {
                match sway_event {
                    SwayEvent::WindowCreated { con_id, app_id, pid } => {
                        if app_id.starts_with("agent.") {
                            if let Some(agent_id) = register_agent_window(&app_id, con_id, &mut state) {
                                if state.config.daemon.process_tree_audit_enabled {
                                    if let (Some(con_pid), Some(agent)) =
                                        (pid, state.agents.get_mut(&agent_id))
                                    {
                                        apply_process_tree_auditing(
                                            agent,
                                            con_pid as u32,
                                            &state.pid_to_agent,
                                            state.config.daemon.max_ancestry_depth,
                                        );
                                    }
                                }

                                // Read parent_id after auditing so an auto-detected
                                // parent reaches the layout on this same event.
                                if let Some(agent) = state.agents.get(&agent_id) {
                                    let project_id = agent.project.clone();
                                    let parent_id = agent.parent_id.clone();

                                    println!(
                                        "[daemon] Sway window created for agent {}: con_id={}",
                                        agent_id.0, con_id
                                    );

                                    delegate_to_layout(
                                        LayoutAction::WindowCreated {
                                            con_id,
                                            app_id: app_id.clone(),
                                            project_id,
                                            parent_id,
                                        },
                                        &layout_manager,
                                        &state,
                                        &sway_cmd_tx,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                    SwayEvent::WindowDestroyed { con_id } => {
                        if let Some(agent_id) = state.con_to_agent.remove(&con_id) {
                            println!("[daemon] Sway window destroyed for agent: {}", agent_id.0);
                            let parent_id =
                                state.agents.get(&agent_id).and_then(|a| a.parent_id.clone());

                            if state.agents.remove(&agent_id).is_some() {
                                state.pid_to_agent.retain(|_, id| id != &agent_id);
                            }
                            promote_orphans(&mut state, &agent_id);

                            delegate_to_layout(
                                LayoutAction::WindowDestroyed {
                                    agent_id: agent_id.clone(),
                                    parent_id,
                                },
                                &layout_manager,
                                &state,
                                &sway_cmd_tx,
                            )
                            .await;
                        }
                    }
                }
            }
            DaemonEvent::Focus => {
                // Focus event handler: hierarchical sibling cycling
                if let Some(target) = next_focus_target(&state.agents, last_focused.as_ref()) {
                    if let Some(agent) = state.agents.get(&target) {
                        let _ = sway_cmd_tx
                            .send(SwayCommand::FocusWindow {
                                mark: agent.mark.clone(),
                            })
                            .await;
                        last_focused = Some(target);
                    }
                }
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
    agent.container_id = Some(con_id);
    let agent_id = agent.id.clone();
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

    match find_registered_parent_ancestor(con_pid, pid_to_agent, max_depth) {
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

fn find_registered_parent_ancestor(
    start_pid: u32,
    pid_to_agent: &HashMap<u32, AgentId>,
    max_depth: usize,
) -> Option<AgentId> {
    let mut current_pid = start_pid;
    let mut depth = 0;
    while current_pid > 1 && depth < max_depth {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_early_termination_at_depth_limit() {
        // Test that find_registered_parent_ancestor respects max_depth limit
        let mut pid_to_agent = HashMap::new();
        let agent_id = AgentId("test_agent".to_string());

        // Mock: agent registered at PID 100
        pid_to_agent.insert(100, agent_id.clone());

        // When max_depth is 5, the function should stop after 5 iterations
        // even if it hasn't found a match
        let result = find_registered_parent_ancestor(1, &pid_to_agent, 5);

        // Since PID 1 doesn't have a registered agent and we limit to depth 5,
        // the result should be None (respecting the depth limit)
        assert_eq!(result, None);
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
        let out_path = format!("/tmp/test_swayve_hook_{}.txt", std::process::id());
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
}
