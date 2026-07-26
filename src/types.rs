use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

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
    pub container_id: Option<i64>,
    pub app_id: String,
    pub mark: String,
    pub status: AgentStatus,
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
    #[serde(rename = "agent.heartbeat")]
    Heartbeat {
        params: HeartbeatParams,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterParams {
    pub agent_id: AgentId,
    pub agent_type: String,
    pub project_id: ProjectId,
    pub pid: u32,
    pub app_id: String,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusParams {
    pub agent_id: AgentId,
    #[serde(alias = "state")]
    pub status: AgentStatus,
    pub progress: Option<f64>,
    pub current_task: Option<String>,
    pub attention_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatParams {
    pub agent_id: AgentId,
    pub timestamp: u64,
}
