use crate::daemon::DaemonState;
use crate::sway::SwayCommand;
use crate::types::ProjectId;

#[derive(Debug, Clone)]
pub enum LayoutAction {
    WindowCreated {
        con_id: i64,
        app_id: String,
        project_id: ProjectId,
    },
}

pub struct LayoutManager;

impl LayoutManager {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_commands(
        &self,
        action: LayoutAction,
        state: &DaemonState,
    ) -> Vec<SwayCommand> {
        let mut commands = Vec::new();
        match action {
            LayoutAction::WindowCreated {
                con_id,
                app_id,
                project_id,
            } => {
                // We still need to find the agent to get the mark
                if let Some(agent) = state.agents.values().find(|a| a.app_id == app_id) {
                    commands.push(SwayCommand::MarkWindow {
                        con_id,
                        mark: agent.mark.clone(),
                    });
                    commands.push(SwayCommand::MoveToWorkspace {
                        con_id,
                        workspace: format!("proj:{}", project_id.0),
                    });
                }
            }
        }
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::{AgentId, ProjectId};

    #[test]
    fn test_window_created() {
        let config = Config::default();
        let mut state = DaemonState::new(config);

        let agent_id = AgentId("test-agent".to_string());
        let project_id = ProjectId("test-project".to_string());
        let pid = 1234;
        let app_id = "agent.test".to_string();

        state.register_agent(
            agent_id.clone(),
            project_id.clone(),
            pid,
            app_id.clone(),
            None,
        );

        let lm = LayoutManager::new();
        let action = LayoutAction::WindowCreated {
            con_id: 42,
            app_id: app_id.clone(),
            project_id: project_id.clone(),
        };

        let commands = lm.compute_commands(action, &state);

        assert_eq!(commands.len(), 2);
        
        // Verify MarkWindow
        let has_mark = commands.iter().any(|c| {
            if let SwayCommand::MarkWindow { con_id, .. } = c {
                *con_id == 42
            } else {
                false
            }
        });
        assert!(has_mark);

        // Verify MoveToWorkspace
        let has_move = commands.iter().any(|c| {
            if let SwayCommand::MoveToWorkspace { con_id, workspace } = c {
                *con_id == 42 && workspace == "proj:test-project"
            } else {
                false
            }
        });
        assert!(has_move);
    }
}
