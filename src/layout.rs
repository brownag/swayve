use crate::sway::SwayCommand;
use crate::types::{AgentId, AgentRegistry, ProjectId};

#[derive(Debug, Clone)]
pub enum LayoutAction {
    WindowCreated {
        con_id: i64,
        app_id: String,
        project_id: ProjectId,
        parent_id: Option<AgentId>,
    },
    WindowDestroyed {
        agent_id: AgentId,
        parent_id: Option<AgentId>,
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
        state: &dyn AgentRegistry,
    ) -> Vec<SwayCommand> {
        let mut commands = Vec::new();
        match action {
            LayoutAction::WindowCreated {
                con_id,
                app_id,
                project_id,
                parent_id,
            } => {
                if app_id.starts_with("agent.monitor.") {
                    let target_id_str = &app_id["agent.monitor.".len()..];
                    let target_id = AgentId(target_id_str.to_string());
                    if let Some(target_agent) = state.agents().get(&target_id) {
                        commands.push(SwayCommand::MoveToWorkspace {
                            con_id,
                            workspace: format!("proj:{}", project_id.0),
                        });
                        commands.push(SwayCommand::SplitHorizontal {
                            parent_mark: target_agent.mark.clone(),
                        });
                        commands.push(SwayCommand::MoveToContainer {
                            con_id,
                            target_mark: target_agent.mark.clone(),
                        });
                    }
                } else if let Some(agent) = state.agents().values().find(|a| a.app_id == app_id) {
                    // --- Basic Setup ---
                    commands.push(SwayCommand::MarkWindow {
                        con_id,
                        mark: agent.mark.clone(),
                    });
                    commands.push(SwayCommand::MoveToWorkspace {
                        con_id,
                        workspace: format!("proj:{}", project_id.0),
                    });

                    // --- Hierarchical Logic ---
                    if let Some(p_id) = parent_id {
                        let children_mark = p_id.to_children_mark();

                        // 2. Determine if it's the first child or a subsequent child
                        let has_siblings = state.agents().values().any(|a| {
                            a.parent_id.as_ref() == Some(&p_id) && a.id != agent.id && a.container_id.is_some()
                        });

                        if !has_siblings {
                            // First child
                            if let Some(parent_agent) = state.agents().get(&p_id) {
                                commands.push(SwayCommand::SplitHorizontal {
                                    parent_mark: parent_agent.mark.clone(),
                                });
                                commands.push(SwayCommand::MarkWindow {
                                    con_id,
                                    mark: children_mark.clone(),
                                });
                                commands.push(SwayCommand::SetLayoutTabbed { mark: children_mark });
                            }
                        } else {
                            // Subsequent child
                            commands.push(SwayCommand::MoveToContainer {
                                con_id,
                                target_mark: children_mark,
                            });
                        }
                    }
                }
            }
            LayoutAction::WindowDestroyed {
                agent_id,
                parent_id,
            } => {
                // The children mark lives on the first child's container, so it dies
                // with that window. Any later sibling would then be moved to a mark
                // that no longer exists, so hand the mark to a surviving sibling.
                if let Some(p_id) = parent_id {
                    let mut survivors: Vec<_> = state
                        .agents()
                        .values()
                        .filter(|a| {
                            a.parent_id.as_ref() == Some(&p_id)
                                && a.id != agent_id
                                && a.container_id.is_some()
                        })
                        .collect();
                    survivors.sort_by_key(|a| a.container_id);

                    if let Some(heir) = survivors.first() {
                        commands.push(SwayCommand::MarkWindow {
                            con_id: heir.container_id.expect("filtered on container_id"),
                            mark: p_id.to_children_mark(),
                        });
                    }
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
    use crate::daemon::DaemonState;
    use crate::types::{AgentId, ProjectId};

    #[test]
    fn test_window_created_basic() {
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
            parent_id: None,
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

    #[test]
    fn test_window_created_first_child() {
        let config = Config::default();
        let mut state = DaemonState::new(config);

        let parent_id = AgentId("parent".to_string());
        let parent_mark = "mark_parent";
        let project_id = ProjectId("test-project".to_string());
        let pid = 1000;
        let app_id = "agent.parent".to_string();

        state.register_agent(
            parent_id.clone(),
            project_id.clone(),
            pid,
            app_id.clone(),
            None,
        );
        
        if let Some(agent) = state.agents.get_mut(&parent_id) {
            agent.mark = parent_mark.to_string();
            agent.container_id = Some(10);
        }

        let child_id = AgentId("child".to_string());
        let child_app_id = "agent.child".to_string();
        let child_pid = 1001;
        let children_mark = "mark_children_parent";

        let lm = LayoutManager::new();
        let action = LayoutAction::WindowCreated {
            con_id: 42,
            app_id: child_app_id.clone(),
            project_id: project_id.clone(),
            parent_id: Some(parent_id.clone()),
        };

        // Setup child in state
        state.register_agent(
            child_id.clone(),
            project_id.clone(),
            child_pid,
            child_app_id.clone(),
            Some(parent_id.clone()),
        );
        if let Some(a) = state.agents.get_mut(&child_id) {
             a.container_id = Some(42);
        }

        let commands = lm.compute_commands(action, &state);

        assert!(commands.iter().any(|c| matches!(c, SwayCommand::SplitHorizontal { .. })));
        assert!(commands.iter().any(|c| matches!(c, SwayCommand::SetLayoutTabbed { .. })));
        assert!(commands.iter().any(|c| {
            if let SwayCommand::MarkWindow { con_id, mark } = c {
                *con_id == 42 && mark == children_mark
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_window_created_subsequent_child() {
        let config = Config::default();
        let mut state = DaemonState::new(config);

        let parent_id = AgentId("parent".to_string());
        let parent_mark = "mark_parent";
        let project_id = ProjectId("test-project".to_string());
        let pid = 1000;
        let app_id = "agent.parent".to_string();

        state.register_agent(
            parent_id.clone(),
            project_id.clone(),
            pid,
            app_id.clone(),
            None,
        );
        if let Some(a) = state.agents.get_mut(&parent_id) {
            a.container_id = Some(10);
            a.mark = parent_mark.to_string();
        }
        state.pid_to_agent.insert(pid, parent_id.clone());

        // 1. First child already exists
        let child1_id = AgentId("child1".to_string());
        state.register_agent(
            child1_id.clone(),
            project_id.clone(),
            1001,
            "agent.child1".to_string(),
            Some(parent_id.clone()),
        );
        if let Some(a) = state.agents.get_mut(&child1_id) {
            a.container_id = Some(11);
        }

        // 2. Second child arrives
        let child2_id = AgentId("child2".to_string());
        let child2_app_id = "agent.child2".to_string();
        let child2_pid = 1002;
        state.register_agent(
            child2_id.clone(),
            project_id.clone(),
            child2_pid,
            child2_app_id.clone(),
            Some(parent_id.clone()),
        );
        if let Some(a) = state.agents.get_mut(&child2_id) {
            a.container_id = Some(42);
        }

        let lm = LayoutManager::new();
        let action = LayoutAction::WindowCreated {
            con_id: 42,
            app_id: child2_app_id,
            project_id: project_id.clone(),
            parent_id: Some(parent_id.clone()),
        };

        let commands = lm.compute_commands(action, &state);

        assert_eq!(commands.len(), 3);
        assert!(commands.iter().any(|c| matches!(c, SwayCommand::MoveToContainer { .. })));
        assert!(commands.iter().any(|c| matches!(c, SwayCommand::MarkWindow { con_id: 42, .. })));
        assert!(commands.iter().any(|c| matches!(c, SwayCommand::MoveToWorkspace { .. })));
    }

    /// A parent with `children` registered beneath it, each already given a window.
    fn state_with_children(children: &[(&str, i64)]) -> (DaemonState, AgentId) {
        let mut state = DaemonState::new(Config::default());
        let parent_id = AgentId("parent".to_string());
        let project = ProjectId("proj".to_string());
        state.register_agent(parent_id.clone(), project.clone(), 1, "agent.parent".to_string(), None);

        for (idx, (name, con_id)) in children.iter().enumerate() {
            let child = AgentId(name.to_string());
            state.register_agent(
                child.clone(),
                project.clone(),
                10 + idx as u32,
                format!("agent.{}", name),
                Some(parent_id.clone()),
            );
            state.agents.get_mut(&child).unwrap().container_id = Some(*con_id);
        }
        (state, parent_id)
    }

    #[test]
    fn test_window_destroyed_rehomes_children_mark() {
        // The first child carries mark_children_parent; when it dies the mark must
        // move to a survivor, or later siblings get moved to a mark that is gone.
        let (state, parent_id) = state_with_children(&[("first", 100), ("second", 200)]);

        let lm = LayoutManager::new();
        let commands = lm.compute_commands(
            LayoutAction::WindowDestroyed {
                agent_id: AgentId("first".to_string()),
                parent_id: Some(parent_id),
            },
            &state,
        );

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            SwayCommand::MarkWindow { con_id: 200, mark } if mark == "mark_children_parent"
        ));
    }

    #[test]
    fn test_window_destroyed_rehomes_to_lowest_survivor() {
        // Registration order differs from con_id order: the heir is the lowest
        // surviving con_id (200), not the first registered survivor (300).
        let (state, parent_id) =
            state_with_children(&[("first", 100), ("second", 300), ("third", 200)]);

        let lm = LayoutManager::new();
        let commands = lm.compute_commands(
            LayoutAction::WindowDestroyed {
                agent_id: AgentId("first".to_string()),
                parent_id: Some(parent_id),
            },
            &state,
        );

        assert_eq!(commands.len(), 1);
        assert!(matches!(
            &commands[0],
            SwayCommand::MarkWindow { con_id: 200, mark } if mark == "mark_children_parent"
        ));
    }

    #[test]
    fn test_window_destroyed_without_survivors_is_silent() {
        // Last child of the parent: nothing left to carry the mark
        let (state, parent_id) = state_with_children(&[("only", 100)]);

        let lm = LayoutManager::new();
        let commands = lm.compute_commands(
            LayoutAction::WindowDestroyed {
                agent_id: AgentId("only".to_string()),
                parent_id: Some(parent_id),
            },
            &state,
        );
        assert!(commands.is_empty());

        // A root agent has no children container to maintain either
        let commands = lm.compute_commands(
            LayoutAction::WindowDestroyed {
                agent_id: AgentId("parent".to_string()),
                parent_id: None,
            },
            &state,
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn test_window_created_monitor() {
        let config = Config::default();
        let mut state = DaemonState::new(config);

        let sub_id = AgentId("sub-1".to_string());
        let project_id = ProjectId("test-proj".to_string());

        state.register_agent(
            sub_id.clone(),
            project_id.clone(),
            100,
            "agent.sub.1".to_string(),
            None,
        );

        let lm = LayoutManager::new();
        let action = LayoutAction::WindowCreated {
            con_id: 99,
            app_id: "agent.monitor.sub-1".to_string(),
            project_id: project_id.clone(),
            parent_id: None,
        };

        let commands = lm.compute_commands(action, &state);

        assert_eq!(commands.len(), 3);
        assert!(matches!(
            &commands[0],
            SwayCommand::MoveToWorkspace { con_id: 99, workspace } if workspace == "proj:test-proj"
        ));
        assert!(matches!(
            &commands[1],
            SwayCommand::SplitHorizontal { parent_mark } if parent_mark == &sub_id.to_mark()
        ));
        assert!(matches!(
            &commands[2],
            SwayCommand::MoveToContainer { con_id: 99, target_mark } if target_mark == &sub_id.to_mark()
        ));
    }
}
