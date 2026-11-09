//! Byte-exact check of `daemon.subscribe_diff` output (G56). Captured before the `DiffFrame`
//! refactor (captured at f1b5a37), so the refactor must not change these bytes.
//! Keys are alphabetical because the daemon builds frames through `serde_json::Value`. One agent only, so HashMap order
//! cannot change the output.
mod common;

use std::time::Duration;
use swayve::config::Config;
use swayve::daemon::{DaemonEvent, run_daemon_loops, start_daemon_ipc};
use swayve::sway::SwayCommand;
use swayve::types::{AgentId, AgentIPCMessage, AgentStatus, ProjectId, RegisterParams, UpdateStatusParams};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

async fn wait_for_socket(path: &str) {
    for _ in 0..100 {
        if UnixStream::connect(path).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("socket {path} never came up");
}

#[tokio::test]
async fn test_subscribe_diff_bytes_are_pinned() {
    let socket_path = common::unique_path("diff_golden", ".sock");
    let _ = fs::remove_file(socket_path).await;

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(64);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(64);
    tokio::spawn(async move { while sway_cmd_rx.recv().await.is_some() {} });

    let listener = tokio::spawn(async move { start_daemon_ipc(socket_path, event_tx).await });
    let daemon = tokio::spawn(async move {
        run_daemon_loops(true, Config::default(), socket_path, event_rx, sway_cmd_tx).await
    });
    wait_for_socket(socket_path).await;

    let line = |msg: AgentIPCMessage| serde_json::to_string(&msg).unwrap() + "\n";
    let mut writer = UnixStream::connect(socket_path).await.expect("connect writer");
    writer
        .write_all(
            line(AgentIPCMessage::Register {
                id: None,
                params: RegisterParams {
                    agent_id: AgentId("golden-1".to_string()),
                    agent_type: "pi".to_string(),
                    project_id: ProjectId("golden-proj".to_string()),
                    pid: 4242,
                    app_id: "agent.pi.golden-1".to_string(),
                    parent_agent_id: None,
                },
            })
            .as_bytes(),
        )
        .await
        .unwrap();
    writer.flush().await.unwrap();

    // Subscribe. The first snapshot goes out on the next broadcast, so trigger one.
    let diff = UnixStream::connect(socket_path).await.expect("connect diff");
    let (r, mut w) = diff.into_split();
    w.write_all(b"{\"method\":\"daemon.subscribe_diff\"}\n").await.unwrap();
    let mut lines = BufReader::new(r).lines();
    writer
        .write_all(
            line(AgentIPCMessage::UpdateStatus {
                params: UpdateStatusParams { agent_id: AgentId("golden-1".to_string()), status: AgentStatus::Thinking },
            })
            .as_bytes(),
        )
        .await
        .unwrap();
    writer.flush().await.unwrap();
    let snapshot = timeout(Duration::from_secs(2), lines.next_line()).await.unwrap().unwrap().unwrap();
    println!("SNAPSHOT={snapshot}");

    // A delta: change the status, expect one upsert line.
    writer
        .write_all(
            line(AgentIPCMessage::UpdateStatus {
                params: UpdateStatusParams { agent_id: AgentId("golden-1".to_string()), status: AgentStatus::Completed },
            })
            .as_bytes(),
        )
        .await
        .unwrap();
    writer.flush().await.unwrap();
    let delta = timeout(Duration::from_secs(2), lines.next_line()).await.unwrap().unwrap().unwrap();
    println!("DELTA={delta}");

    assert_eq!(snapshot, EXPECTED_SNAPSHOT);
    assert_eq!(delta, EXPECTED_DELTA);

    listener.abort();
    daemon.abort();
    let _ = fs::remove_file(socket_path).await;
}

const EXPECTED_SNAPSHOT: &str = r#"{"remove":[],"snapshot":true,"upsert":[{"app_id":"agent.pi.golden-1","container_id":null,"id":"golden-1","mark":"mark_golden_1","parent_id":null,"project":"golden-proj","status":"thinking"}]}"#;
const EXPECTED_DELTA: &str = r#"{"remove":[],"snapshot":false,"upsert":[{"app_id":"agent.pi.golden-1","container_id":null,"id":"golden-1","mark":"mark_golden_1","parent_id":null,"project":"golden-proj","status":"completed"}]}"#;
