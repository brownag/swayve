use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn to_mark(&self) -> String {
        format!("mark_{}", self.0.replace('-', "_"))
    }

    pub fn to_children_mark(&self) -> String {
        format!("mark_children_{}", self.0.replace('-', "_"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Thinking,
    ExecutingTool,
    AwaitingHumanInput,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWindow {
    pub id: AgentId,
    pub project: ProjectId,
    pub parent_id: Option<AgentId>,
    pub container_id: Option<i64>,
    pub app_id: String,
    pub mark: String,
    pub status: AgentStatus,
}

use std::collections::HashMap;

pub trait AgentRegistry {
    fn agents(&self) -> &HashMap<AgentId, AgentWindow>;
}

/// Incoming IPC events from sub-agents
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum AgentIPCMessage {
    #[serde(rename = "agent.register")]
    Register {
        id: Option<serde_json::Value>,
        params: RegisterParams,
    },
    #[serde(rename = "agent.update_status")]
    UpdateStatus {
        params: UpdateStatusParams,
    },
    /// Ask the daemon to advance focus to the next window in the agent hierarchy.
    #[serde(rename = "daemon.focus")]
    Focus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub agent_id: AgentId,
    pub agent_type: String,
    pub project_id: ProjectId,
    pub pid: u32,
    pub app_id: String,
    pub parent_agent_id: Option<AgentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusParams {
    pub agent_id: AgentId,
    #[serde(alias = "state")]
    pub status: AgentStatus,
}
