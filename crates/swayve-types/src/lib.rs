use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_mark(&self) -> String {
        format!("mark_{}", self.0.replace('-', "_"))
    }

    pub fn to_children_mark(&self) -> String {
        format!("mark_children_{}", self.0.replace('-', "_"))
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl ProjectId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

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

impl AgentStatus {
    /// Parses an `AgentStatus` from a JSON value.
    pub fn from_value(v: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(v.clone()).ok()
    }
}

impl fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Thinking => write!(f, "thinking"),
            AgentStatus::ExecutingTool => write!(f, "executing_tool"),
            AgentStatus::AwaitingHumanInput => write!(f, "awaiting_human_input"),
            AgentStatus::Completed => write!(f, "completed"),
            AgentStatus::Failed(msg) => write!(f, "failed: {msg}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentWindow {
    pub id: AgentId,
    pub project: ProjectId,
    pub parent_id: Option<AgentId>,
    pub container_id: Option<i64>,
    pub app_id: String,
    pub mark: String,
    pub status: AgentStatus,
}

impl AgentWindow {
    /// Parses an `AgentWindow` from a JSON value. Requires `id`, `project`, `app_id`, `mark`,
    /// and `status`. Allows optional or null `parent_id` and `container_id`. Ignores extra keys.
    pub fn from_value(v: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(v.clone()).ok()
    }
}

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
    /// Subscribe to daemon agent status broadcast updates.
    #[serde(rename = "daemon.subscribe")]
    SubscribeStatus,
    /// Subscribe to diff-based agent status updates (snapshot first, then deltas).
    #[serde(rename = "daemon.subscribe_diff")]
    SubscribeDiff,
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

/// One line of the `daemon.subscribe_diff` stream. The daemon builds it, and the CLI/HUD folds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFrame {
    pub snapshot: bool,
    pub upsert: Vec<AgentWindow>,
    pub remove: Vec<AgentId>,
}

impl DiffFrame {
    /// Parses a `DiffFrame` from a JSON value.
    pub fn from_value(v: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(v.clone()).ok()
    }

    /// Parses a `DiffFrame` from a JSON string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    /// The wire line, without the trailing newline. Keys come out alphabetical (`remove`,
    /// `snapshot`, `upsert`) because the frame goes through `serde_json::Value`. Declaration
    /// order would change the bytes pinned in `tests/diff_golden.rs`.
    pub fn to_line(&self) -> serde_json::Result<String> {
        serde_json::to_string(&serde_json::to_value(self)?)
    }

    /// Applies the frame to folded state: a snapshot clears it, then upserts and removes apply.
    pub fn fold_into(self, map: &mut HashMap<String, AgentWindow>) {
        if self.snapshot {
            map.clear();
        }
        for a in self.upsert {
            map.insert(a.id.0.clone(), a);
        }
        for id in self.remove {
            map.remove(&id.0);
        }
    }
}

impl std::str::FromStr for DiffFrame {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DiffFrame::from_str(s).ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_status_roundtrip() {
        let cases = [
            (AgentStatus::Idle, "\"idle\""),
            (AgentStatus::Thinking, "\"thinking\""),
            (AgentStatus::ExecutingTool, "\"executing_tool\""),
            (AgentStatus::AwaitingHumanInput, "\"awaiting_human_input\""),
            (AgentStatus::Completed, "\"completed\""),
            (AgentStatus::Failed("boom".into()), "{\"failed\":\"boom\"}"),
        ];
        for (st, json) in cases {
            assert_eq!(serde_json::to_string(&st).unwrap(), json);
            assert_eq!(serde_json::from_str::<AgentStatus>(json).unwrap(), st);
        }
    }

    #[test]
    fn agent_id_marks() {
        let id = AgentId("a-b-c".into());
        assert_eq!(id.to_mark(), "mark_a_b_c");
        assert_eq!(id.to_children_mark(), "mark_children_a_b_c");
    }

    #[test]
    fn test_agent_status_parsing_and_display() {
        let cases = vec![
            (serde_json::json!("idle"), AgentStatus::Idle, "idle"),
            (
                serde_json::json!("thinking"),
                AgentStatus::Thinking,
                "thinking",
            ),
            (
                serde_json::json!("executing_tool"),
                AgentStatus::ExecutingTool,
                "executing_tool",
            ),
            (
                serde_json::json!("awaiting_human_input"),
                AgentStatus::AwaitingHumanInput,
                "awaiting_human_input",
            ),
            (
                serde_json::json!("completed"),
                AgentStatus::Completed,
                "completed",
            ),
            (
                serde_json::json!({"failed": "connection dropped"}),
                AgentStatus::Failed("connection dropped".to_string()),
                "failed: connection dropped",
            ),
        ];

        for (val, expected_status, expected_display) in cases {
            let parsed = AgentStatus::from_value(&val).expect("status should parse");
            assert_eq!(parsed, expected_status);
            assert_eq!(parsed.to_string(), expected_display);
        }

        assert_eq!(AgentStatus::from_value(&serde_json::json!(123)), None);
        assert_eq!(AgentStatus::from_value(&serde_json::json!(true)), None);
        assert_eq!(
            AgentStatus::from_value(&serde_json::json!({"failed": 404})),
            None
        );
        assert_eq!(
            AgentStatus::from_value(&serde_json::json!({"unknown": "object"})),
            None
        );
    }

    #[test]
    fn test_agent_window_parsing() {
        let val = serde_json::json!({
            "id": "agent-1",
            "project": "proj-a",
            "parent_id": "agent-root",
            "container_id": 42,
            "app_id": "ghostty",
            "mark": "mark_agent_1",
            "status": "idle",
            "extra_field": "ignored"
        });

        let window = AgentWindow::from_value(&val).expect("should parse valid agent window");
        assert_eq!(window.id.0, "agent-1");
        assert_eq!(window.project.0, "proj-a");
        assert_eq!(window.parent_id, Some(AgentId("agent-root".to_string())));
        assert_eq!(window.container_id, Some(42));
        assert_eq!(window.app_id, "ghostty");
        assert_eq!(window.mark, "mark_agent_1");
        assert_eq!(window.status, AgentStatus::Idle);

        let val_nulls = serde_json::json!({
            "id": "agent-2",
            "project": "proj-b",
            "parent_id": null,
            "container_id": null,
            "app_id": "alacritty",
            "mark": "mark_agent_2",
            "status": "thinking"
        });
        let window_nulls =
            AgentWindow::from_value(&val_nulls).expect("should parse null optional fields");
        assert_eq!(window_nulls.parent_id, None);
        assert_eq!(window_nulls.container_id, None);

        let val_omitted = serde_json::json!({
            "id": "agent-3",
            "project": "proj-c",
            "app_id": "foot",
            "mark": "mark_agent_3",
            "status": "completed"
        });
        let window_omitted =
            AgentWindow::from_value(&val_omitted).expect("should parse omitted optional fields");
        assert_eq!(window_omitted.parent_id, None);
        assert_eq!(window_omitted.container_id, None);

        let val_missing = serde_json::json!({
            "id": "agent-4",
            "project": "proj-d",
            "app_id": "foot",
            "status": "completed"
        });
        assert_eq!(AgentWindow::from_value(&val_missing), None);

        let val_bad_container = serde_json::json!({
            "id": "agent-5",
            "project": "proj-e",
            "container_id": "not-a-number",
            "app_id": "foot",
            "mark": "mark_agent_5",
            "status": "completed"
        });
        assert_eq!(AgentWindow::from_value(&val_bad_container), None);
    }

    #[test]
    fn test_diff_frame_parsing() {
        let frame_json = r#"{
            "snapshot": true,
            "upsert": [
                {
                    "id": "agent-1",
                    "project": "proj-a",
                    "app_id": "ghostty",
                    "mark": "mark_agent_1",
                    "status": "idle"
                }
            ],
            "remove": ["agent-old"]
        }"#;

        let frame = DiffFrame::from_str(frame_json).expect("should parse valid frame");
        assert!(frame.snapshot);
        assert_eq!(frame.upsert.len(), 1);
        assert_eq!(frame.upsert[0].id.0, "agent-1");
        assert_eq!(frame.remove, vec![AgentId("agent-old".to_string())]);

        let frame_via_trait: DiffFrame = frame_json.parse().expect("should parse via FromStr trait");
        assert_eq!(frame, frame_via_trait);

        assert_eq!(DiffFrame::from_str("{not valid json"), None);

        let missing_upsert = r#"{"snapshot": false, "remove": []}"#;
        assert_eq!(DiffFrame::from_str(missing_upsert), None);

        let invalid_upsert_item = r#"{
            "snapshot": false,
            "upsert": [{"bad": "data"}],
            "remove": []
        }"#;
        assert_eq!(DiffFrame::from_str(invalid_upsert_item), None);

        let invalid_remove_item = r#"{
            "snapshot": false,
            "upsert": [],
            "remove": [123]
        }"#;
        assert_eq!(DiffFrame::from_str(invalid_remove_item), None);
    }
}
