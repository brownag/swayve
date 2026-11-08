use std::path::Path;
mod common;

use std::time::Duration;
use swayve::config::Config;
use swayve::daemon::{run_daemon_loops, start_daemon_ipc, DaemonEvent};
use swayve::sway::SwayCommand;
use swayve::types::{AgentId, AgentIPCMessage, AgentStatus, ProjectId, RegisterParams, UpdateStatusParams};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

const AGENT_A: &str = r#"{"id":"golden-a","project":"golden-proj","parent_id":null,"container_id":null,"app_id":"agent.pi.golden-a","mark":"mark_golden_a","status":"idle"}"#;
const AGENT_B: &str = r#"{"id":"golden-b","project":"golden-proj","parent_id":null,"container_id":null,"app_id":"agent.pi.golden-b","mark":"mark_golden_b","status":"awaiting_human_input"}"#;

async fn wait_for_socket(path: &str) {
    for _ in 0..200 {
        if UnixStream::connect(path).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("socket {path} never became ready");
}

/// Connect, send `daemon.subscribe`, and return the first line received.
async fn subscribe_first_line(path: &str) -> String {
    let mut stream = UnixStream::connect(path).await.expect("connect subscriber");
    let payload = serde_json::to_string(&AgentIPCMessage::SubscribeStatus).unwrap() + "\n";
    stream.write_all(payload.as_bytes()).await.expect("send subscribe");
    stream.flush().await.unwrap();
    let mut lines = BufReader::new(stream).lines();
    timeout(Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout waiting for first subscribe line")
        .expect("read error")
        .expect("stream closed")
}

#[tokio::test]
async fn test_subscribe_full_list_golden_bytes() {
    let socket_path = common::unique_path("golden", ".sock");
    let _ = fs::remove_file(Path::new(socket_path)).await;

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, _sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let listener = tokio::spawn(async move { start_daemon_ipc(&socket_path_str, event_tx).await });
    let daemon = tokio::spawn(async move {
        run_daemon_loops(true, Config::default(), socket_path, event_rx, sway_cmd_tx).await
    });
    wait_for_socket(socket_path).await;

    let mut client = UnixStream::connect(socket_path).await.expect("connect client");
    let mut msgs = Vec::new();
    for (id, pid) in [("golden-a", 41001), ("golden-b", 41002)] {
        msgs.push(AgentIPCMessage::Register {
            id: None,
            params: RegisterParams {
                agent_id: AgentId(id.to_string()),
                agent_type: "pi".to_string(),
                project_id: ProjectId("golden-proj".to_string()),
                pid,
                app_id: format!("agent.pi.{id}"),
                parent_agent_id: None,
            },
        });
    }
    msgs.push(AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: AgentId("golden-b".to_string()),
            status: AgentStatus::AwaitingHumanInput,
        },
    });
    for m in &msgs {
        let line = serde_json::to_string(m).unwrap() + "\n";
        client.write_all(line.as_bytes()).await.expect("send message");
    }
    client.flush().await.unwrap();

    // Cross-connection ordering is not guaranteed: probe until the state has settled.
    let mut settled = false;
    for _ in 0..200 {
        if subscribe_first_line(socket_path).await.contains("awaiting_human_input") {
            settled = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(settled, "daemon state never reached the expected status");

    let line = subscribe_first_line(socket_path).await;
    assert!(!line.starts_with('{'), "first line must be a bare array, got: {line}");
    assert!(line.starts_with('['), "first line must be a JSON array, got: {line}");

    // `agents` is a HashMap, so element order varies per process; each ordering is byte-exact.
    let golden_ab = format!("[{AGENT_A},{AGENT_B}]");
    let golden_ba = format!("[{AGENT_B},{AGENT_A}]");
    assert!(
        line == golden_ab || line == golden_ba,
        "line matches neither golden ordering:\n{line}"
    );

    listener.abort();
    daemon.abort();
    let _ = fs::remove_file(Path::new(socket_path)).await;
}
