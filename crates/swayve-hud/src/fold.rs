use std::collections::HashMap;

use crate::types::{AgentWindow, DiffFrame};

/// Folds one `daemon.subscribe_diff` line into `map`.
///
/// Returns `false`, changing nothing in `map`, when the line is not valid JSON
/// or does not conform to the `DiffFrame` schema.
pub fn fold_diff_line(map: &mut HashMap<String, AgentWindow>, line: &str) -> bool {
    let Some(frame) = DiffFrame::from_str(line) else {
        return false;
    };
    frame.fold_into(map);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentId, AgentStatus, ProjectId};

    fn helper_agent_window(id: &str, status: AgentStatus) -> AgentWindow {
        AgentWindow {
            id: AgentId(id.to_string()),
            project: ProjectId("test-proj".to_string()),
            parent_id: None,
            container_id: Some(100),
            app_id: "alacritty".to_string(),
            mark: format!("mark_{id}"),
            status,
        }
    }

    #[test]
    fn test_fold_diff_sequence() {
        let mut map = HashMap::new();

        // 1. Initial snapshot with agent "a"
        let snap_line = serde_json::json!({
            "snapshot": true,
            "upsert": [
                {
                    "id": "a",
                    "project": "proj-1",
                    "app_id": "ghostty",
                    "mark": "mark_a",
                    "status": "idle"
                }
            ],
            "remove": []
        })
        .to_string();

        assert!(fold_diff_line(&mut map, &snap_line));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("a").unwrap().status, AgentStatus::Idle);

        // 2. Incremental upsert adding "b" and updating "a"
        let upsert_line = serde_json::json!({
            "snapshot": false,
            "upsert": [
                {
                    "id": "a",
                    "project": "proj-1",
                    "app_id": "ghostty",
                    "mark": "mark_a",
                    "status": "thinking"
                },
                {
                    "id": "b",
                    "project": "proj-1",
                    "app_id": "foot",
                    "mark": "mark_b",
                    "status": "executing_tool"
                }
            ],
            "remove": []
        })
        .to_string();

        assert!(fold_diff_line(&mut map, &upsert_line));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("a").unwrap().status, AgentStatus::Thinking);
        assert_eq!(map.get("b").unwrap().status, AgentStatus::ExecutingTool);

        // 3. Incremental remove of "a"
        let remove_line = serde_json::json!({
            "snapshot": false,
            "upsert": [],
            "remove": ["a"]
        })
        .to_string();

        assert!(fold_diff_line(&mut map, &remove_line));
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key("a"));
        assert!(map.contains_key("b"));
    }

    #[test]
    fn test_fold_diff_skips_invalid_lines_without_modifying_map() {
        let mut map = HashMap::new();
        map.insert(
            "existing".to_string(),
            helper_agent_window("existing", AgentStatus::Idle),
        );
        let snapshot_before = map.clone();

        // Completely invalid JSON
        assert!(!fold_diff_line(&mut map, "not json at all"));
        assert_eq!(map, snapshot_before);

        assert!(!fold_diff_line(&mut map, "{\"snapshot\": "));
        assert_eq!(map, snapshot_before);

        // Valid JSON but wrong schema (missing required fields)
        assert!(!fold_diff_line(&mut map, "{\"random\": 123}"));
        assert_eq!(map, snapshot_before);

        assert!(!fold_diff_line(
            &mut map,
            "{\"snapshot\": \"not a bool\", \"upsert\": [], \"remove\": []}"
        ));
        assert_eq!(map, snapshot_before);

        // Malformed upsert element
        assert!(!fold_diff_line(
            &mut map,
            "{\"snapshot\": false, \"upsert\": [{\"id\": 1}], \"remove\": []}"
        ));
        assert_eq!(map, snapshot_before);
    }

    #[test]
    fn test_fold_diff_snapshot_clears_preexisting_entries() {
        let mut map = HashMap::new();
        map.insert("old-1".to_string(), helper_agent_window("old-1", AgentStatus::Completed));
        map.insert("old-2".to_string(), helper_agent_window("old-2", AgentStatus::Idle));
        assert_eq!(map.len(), 2);

        // Snapshot with new agent "new-1"
        let snap_line = serde_json::json!({
            "snapshot": true,
            "upsert": [
                {
                    "id": "new-1",
                    "project": "proj-new",
                    "app_id": "ghostty",
                    "mark": "mark_new_1",
                    "status": "thinking"
                }
            ],
            "remove": []
        })
        .to_string();

        assert!(fold_diff_line(&mut map, &snap_line));
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key("old-1"));
        assert!(!map.contains_key("old-2"));
        assert!(map.contains_key("new-1"));
    }
}
