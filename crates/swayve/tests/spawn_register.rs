//! `spawn` waits for its registration to be applied (G52, re-scoped, D1). Uses a real daemon loop.
mod common;

use std::time::{Duration, Instant};
use swayve::cli::register_and_wait;
use swayve::config::Config;
use swayve::daemon::{DaemonEvent, run_daemon_loops, start_daemon_ipc};
use swayve::sway::SwayCommand;
use swayve::types::{AgentId, AgentIPCMessage, ProjectId, RegisterParams};
use tokio::fs;
use tokio::net::UnixStream;
use tokio::sync::mpsc;

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
async fn test_register_and_wait_sees_registration_and_times_out_otherwise() {
    let socket_path = common::unique_path("spawn_reg", ".sock");
    let _ = fs::remove_file(socket_path).await;

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(64);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(64);
    tokio::spawn(async move { while sway_cmd_rx.recv().await.is_some() {} });
    let listener = tokio::spawn(async move { start_daemon_ipc(socket_path, event_tx).await });
    let daemon = tokio::spawn(async move {
        run_daemon_loops(true, Config::default(), socket_path, event_rx, sway_cmd_tx).await
    });
    wait_for_socket(socket_path).await;

    let reg = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("spawn-1".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("p".to_string()),
            pid: 5151,
            app_id: "agent.pi.spawn-1".to_string(),
            parent_agent_id: None,
        },
    };
    let line = serde_json::to_string(&reg).unwrap() + "\n";

    // Registered: the wait returns true.
    assert!(register_and_wait(socket_path, &line, "spawn-1", Duration::from_secs(2)).await);

    // Never registered: the wait gives up at the timeout.
    let start = Instant::now();
    let never = "{\"method\":\"daemon.subscribe\"}\n";
    assert!(!register_and_wait(socket_path, never, "ghost-9", Duration::from_millis(300)).await);
    assert!(start.elapsed() < Duration::from_secs(1), "timeout took {:?}", start.elapsed());

    listener.abort();
    daemon.abort();
    let _ = fs::remove_file(socket_path).await;
}
