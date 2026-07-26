use swayve::config::Config;
use swayve::daemon::{run_daemon_loops, start_daemon_ipc, DaemonEvent};
use swayve::sway::SwayCommand;
use swayve::types::{AgentId, AgentIPCMessage, AgentStatus, ProjectId, RegisterParams, UpdateStatusParams};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::test]
async fn test_daemon_registration_and_status() {
    let socket_path = "/tmp/swayve_test.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    // Spawn Unix listener
    let socket_path_str = socket_path.to_string();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx).await
    });

    // Spawn central daemon loop
    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    // Wait a brief moment for socket to bind
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the socket and send registration message
    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect to test socket");
    let register_msg = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("test-agent".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("test-proj".to_string()),
            pid: 12345,
            app_id: "agent.pi.test-agent".to_string(),
            capabilities: None,
        },
    };
    let payload = serde_json::to_string(&register_msg).unwrap() + "\n";
    stream.write_all(payload.as_bytes()).await.expect("Failed to write register message");

    // Send update status message
    let status_msg = AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: AgentId("test-agent".to_string()),
            status: AgentStatus::AwaitingHumanInput,
            progress: Some(0.5),
            current_task: Some("Test task".to_string()),
            attention_reason: Some("input".to_string()),
        },
    };
    let payload2 = serde_json::to_string(&status_msg).unwrap() + "\n";
    stream.write_all(payload2.as_bytes()).await.expect("Failed to write status message");
    stream.flush().await.unwrap();

    // Verify command executor receives SetBorderColor and FocusWindow commands
    let cmd1 = timeout(Duration::from_secs(1), sway_cmd_rx.recv())
        .await
        .expect("Timeout waiting for first Sway command")
        .expect("Channel closed");

    match cmd1 {
        SwayCommand::SetBorderColor { mark, hex_color } => {
            assert_eq!(mark, "mark_test_agent");
            assert_eq!(hex_color, "#ff5555"); // AwaitingHumanInput border color (Dracula red)
        }
        _ => panic!("Expected SetBorderColor command, got {:?}", cmd1),
    }

    let cmd2 = timeout(Duration::from_secs(1), sway_cmd_rx.recv())
        .await
        .expect("Timeout waiting for second Sway command")
        .expect("Channel closed");

    match cmd2 {
        SwayCommand::FocusWindow { mark } => {
            assert_eq!(mark, "mark_test_agent");
        }
        _ => panic!("Expected FocusWindow command, got {:?}", cmd2),
    }

    // Clean up
    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}
