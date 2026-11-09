use std::collections::HashMap;
mod common;

use std::time::Duration;
use swayve::config::Config;
use swayve::daemon::{DaemonEvent, run_daemon_loops, start_daemon_ipc};
use swayve::sway::SwayCommand;
use swayve::types::{AgentIPCMessage, AgentId, AgentStatus, ProjectId, RegisterParams, UpdateStatusParams};
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

/// First line of a fresh `daemon.subscribe` stream, as a list sorted by id.
async fn daemon_list(path: &str) -> Vec<serde_json::Value> {
    let mut s = UnixStream::connect(path).await.expect("connect full client");
    s.write_all(b"{\"method\":\"daemon.subscribe\"}\n").await.unwrap();
    let (r, _) = s.into_split();
    let line = timeout(Duration::from_secs(2), BufReader::new(r).lines().next_line())
        .await
        .expect("Timeout waiting for full agent list")
        .expect("read error")
        .expect("full stream closed");
    let mut list: Vec<serde_json::Value> = serde_json::from_str(&line).expect("agent list JSON");
    list.sort_by_key(|a| a["id"].as_str().unwrap().to_string());
    list
}

#[tokio::test]
async fn test_stalled_diff_subscriber_recovers_with_snapshot() {
    let socket_path = common::unique_path("slow", ".sock");
    let _ = fs::remove_file(socket_path).await;

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(64);
    // The daemon awaits border-colour sends; drain so 300 updates cannot stall it.
    tokio::spawn(async move { while sway_cmd_rx.recv().await.is_some() {} });

    let listener_handle = tokio::spawn(async move { start_daemon_ipc(socket_path, event_tx).await });
    let config = Config::default();
    let daemon_handle =
        tokio::spawn(async move { run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await });
    wait_for_socket(socket_path).await;

    // Diff subscriber that does not read yet.
    let mut diff_stream = UnixStream::connect(socket_path).await.expect("connect diff client");
    diff_stream.write_all(b"{\"method\":\"daemon.subscribe_diff\"}\n").await.unwrap();
    let (diff_reader, _diff_writer) = diff_stream.into_split();
    let mut diff_lines = BufReader::new(diff_reader).lines();

    let line = |msg: AgentIPCMessage| serde_json::to_string(&msg).unwrap() + "\n";
    let update = |status: AgentStatus| {
        line(AgentIPCMessage::UpdateStatus {
            params: UpdateStatusParams { agent_id: AgentId("slow-a1".to_string()), status },
        })
    };

    // One agent with an ~8 KiB app_id, so every diff line is ~8 KiB.
    let mut client = UnixStream::connect(socket_path).await.expect("connect client");
    client
        .write_all(
            line(AgentIPCMessage::Register {
                id: None,
                params: RegisterParams {
                    agent_id: AgentId("slow-a1".to_string()),
                    agent_type: "pi".to_string(),
                    project_id: ProjectId("slow-proj".to_string()),
                    pid: 32000,
                    app_id: format!("agent.pi.slow-a1.{}", "x".repeat(8192)),
                    parent_agent_id: None,
                },
            })
            .as_bytes(),
        )
        .await
        .unwrap();
    // Split the 300 updates over two connections of 150 lines each, so no connection
    // trips the per-client IPC rate limit (burst 200). The second batch starts only
    // after the daemon has processed the first, so the update order is unchanged.
    client.flush().await.unwrap();
    for _ in 0..100 {
        if !daemon_list(socket_path).await.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut first = UnixStream::connect(socket_path).await.expect("connect first batch");
    for i in 0..150 {
        first.write_all(update(AgentStatus::Failed(format!("s{i}"))).as_bytes()).await.unwrap();
    }
    first.flush().await.unwrap();
    let mid = serde_json::json!(AgentStatus::Failed("s149".to_string()));
    let mut mid_processed = false;
    for _ in 0..100 {
        if daemon_list(socket_path).await.first().is_some_and(|a| a["status"] == mid) {
            mid_processed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(mid_processed, "daemon never processed the first batch");
    let mut second = UnixStream::connect(socket_path).await.expect("connect second batch");
    for i in 150..300 {
        second.write_all(update(AgentStatus::Failed(format!("s{i}"))).as_bytes()).await.unwrap();
    }
    second.flush().await.unwrap();

    // Ordering check: wait until the daemon has processed the last update.
    let last = serde_json::json!(AgentStatus::Failed("s299".to_string()));
    let mut processed = false;
    for _ in 0..100 {
        if daemon_list(socket_path).await.first().is_some_and(|a| a["status"] == last) {
            processed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(processed, "daemon never processed all status updates");

    // Resume reading. Bounded: at most 1000 reads in total.
    let mut map: HashMap<String, serde_json::Value> = HashMap::new();
    let mut snapshots = 0;
    let mut reads = 0;
    let mut triggered = false;
    let completed = serde_json::json!(AgentStatus::Completed);
    loop {
        assert!(reads < 1000, "too many reads; map = {map:?}");
        reads += 1;
        // 300 ms of silence means the backlog is drained; 2 s once the trigger is sent.
        let wait = if triggered { Duration::from_secs(2) } else { Duration::from_millis(300) };
        let Ok(next) = timeout(wait, diff_lines.next_line()).await else {
            assert!(!triggered, "timed out waiting for recovery; map = {map:?}");
            // The drop left needs_snapshot set; the next broadcast sends the snapshot.
            triggered = true;
            client.write_all(update(AgentStatus::Completed).as_bytes()).await.unwrap();
            client.flush().await.unwrap();
            continue;
        };
        let v: serde_json::Value = serde_json::from_str(&next.expect("read error").expect("stream closed"))
            .expect("diff line is JSON");
        if v["snapshot"].as_bool().expect("snapshot flag") {
            snapshots += 1;
            map.clear();
        }
        for a in v["upsert"].as_array().expect("upsert array") {
            map.insert(a["id"].as_str().unwrap().to_string(), a.clone());
        }
        for id in v["remove"].as_array().expect("remove array") {
            map.remove(id.as_str().unwrap());
        }
        if triggered && map.get("slow-a1").is_some_and(|a| a["status"] == completed) {
            break;
        }
    }

    assert!(snapshots >= 2, "expected a recovery snapshot after the first; saw {snapshots}");
    let mut client_list: Vec<serde_json::Value> = map.into_values().collect();
    client_list.sort_by_key(|a| a["id"].as_str().unwrap().to_string());
    assert_eq!(client_list, daemon_list(socket_path).await);

    listener_handle.abort();
    daemon_handle.abort();
    let _ = fs::remove_file(socket_path).await;
}
