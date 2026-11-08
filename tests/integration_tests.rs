use swayve::config::Config;
use swayve::daemon::{run_daemon_loops, start_daemon_ipc, DaemonEvent};
use swayve::sway::{SwayCommand, SwayEvent};
use swayve::types::{AgentId, AgentIPCMessage, AgentStatus, ProjectId, RegisterParams, UpdateStatusParams};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

async fn recv_command(rx: &mut mpsc::Receiver<SwayCommand>, timeout_ms: u64) -> Option<SwayCommand> {
    timeout(Duration::from_millis(timeout_ms), rx.recv()).await.ok().flatten()
}

async fn wait_for_socket(path: &str) {
    for _ in 0..200 {
        if UnixStream::connect(path).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("socket {path} not ready after 2s");
}

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
    wait_for_socket(socket_path).await;

    // Connect to the socket and send registration message
    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect to test socket");
    let register_msg = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("pi-wave1-12345".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("test-proj".to_string()),
            pid: 12345,
            app_id: "agent.pi.wave1.12345".to_string(),
            parent_agent_id: None,
        },
    };
    let payload = serde_json::to_string(&register_msg).unwrap() + "\n";
    stream.write_all(payload.as_bytes()).await.expect("Failed to write register message");

    // Send update status message
    let status_msg = AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: AgentId("pi-wave1-12345".to_string()),
            status: AgentStatus::AwaitingHumanInput,
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
            assert_eq!(mark, "mark_pi_wave1_12345");
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
            assert_eq!(mark, "mark_pi_wave1_12345");
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

#[tokio::test]
async fn test_subagent_layout_routing() {
    let socket_path = "/tmp/swayve_test_sub.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx_listener).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    // 1. Register Parent
    let reg_parent = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("parent-1".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-x".to_string()),
            pid: 123,
            app_id: "agent.pi.parent-1".to_string(),
            parent_agent_id: None,
        },
    };
    stream.write_all((serde_json::to_string(&reg_parent).unwrap() + "\n").as_bytes()).await.unwrap();

    // 2. Register Child 1
    let reg_child1 = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("child-1".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-x".to_string()),
            pid: 124,
            app_id: "agent.pi.child-1".to_string(),
            parent_agent_id: Some(AgentId("parent-1".to_string())),
        },
    };
    stream.write_all((serde_json::to_string(&reg_child1).unwrap() + "\n").as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    // Allow time for daemon socket task to process registration messages
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate window creation events in daemon loop
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated { con_id: 1000, app_id: "agent.pi.parent-1".to_string(), pid: None })).await;
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated { con_id: 1001, app_id: "agent.pi.child-1".to_string(), pid: None })).await;

    // Check SwayCommands for Child 1 (should focus parent and split, mark children container, and set layout to tabbed)
    let mut split_received = false;
    let mut mark_children_received = false;
    let mut layout_tabbed_received = false;

    // Expecting:
    // MarkWindow for parent, MoveToWorkspace for parent, MarkWindow for child1, MoveToWorkspace for child1
    // And layout routing: SplitHorizontal, MarkWindow (for children group), SetLayoutTabbed
    for _ in 0..7 {
        if let Some(cmd) = recv_command(&mut sway_cmd_rx, 500).await {
            match cmd {
                SwayCommand::SplitHorizontal { parent_mark } => {
                    assert_eq!(parent_mark, "mark_parent_1");
                    split_received = true;
                }
                SwayCommand::MarkWindow { con_id, mark } => {
                    if mark == "mark_children_parent_1" {
                        assert_eq!(con_id, 1001);
                        mark_children_received = true;
                    }
                }
                SwayCommand::SetLayoutTabbed { mark }
                    if mark == "mark_children_parent_1" => {
                        layout_tabbed_received = true;
                    }
                _ => {}
            }
        }
    }

    assert!(split_received, "SplitHorizontal not received");
    assert!(mark_children_received, "MarkWindow for child container not received");
    assert!(layout_tabbed_received, "SetLayoutTabbed not received");

    // 3. Register Child 2
    let reg_child2 = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("child-2".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-x".to_string()),
            pid: 125,
            app_id: "agent.pi.child-2".to_string(),
            parent_agent_id: Some(AgentId("parent-1".to_string())),
        },
    };
    stream.write_all((serde_json::to_string(&reg_child2).unwrap() + "\n").as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    // Allow time for daemon socket task to process registration message
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Window creation for child 2
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated { con_id: 1002, app_id: "agent.pi.child-2".to_string(), pid: None })).await;

    // Expecting MoveToContainer command for child 2
    let mut move_to_container_received = false;
    for _ in 0..5 {
        if let Some(cmd) = recv_command(&mut sway_cmd_rx, 500).await
            && let SwayCommand::MoveToContainer { con_id, target_mark } = cmd
                && con_id == 1002 && target_mark == "mark_children_parent_1" {
                    move_to_container_received = true;
                }
    }
    assert!(move_to_container_received, "MoveToContainer not received for child-2");

    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

fn get_test_ppid() -> u32 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
    let r_paren = stat.rfind(')').unwrap();
    let rest = &stat[r_paren + 1..];
    let fields: Vec<&str> = rest.split_whitespace().collect();
    fields[1].parse::<u32>().unwrap()
}

#[tokio::test]
async fn test_dynamic_process_tree_auditing() {
    let socket_path = "/tmp/swayve_test_audit.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx_listener).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    // Get current test process parent PPID
    let test_ppid = get_test_ppid();

    // 1. Register Parent with cargo's PID (the parent of our test process)
    let reg_parent = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("parent-audit".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-y".to_string()),
            pid: test_ppid,
            app_id: "agent.pi.parent-audit".to_string(),
            parent_agent_id: None,
        },
    };
    stream.write_all((serde_json::to_string(&reg_parent).unwrap() + "\n").as_bytes()).await.unwrap();

    // 2. Register Child (with parent_agent_id = None so that it relies on dynamic process tree audit)
    let reg_child = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("child-audit".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-y".to_string()),
            pid: std::process::id(), // Test process PID
            app_id: "agent.pi.child-audit".to_string(),
            parent_agent_id: None,
        },
    };
    stream.write_all((serde_json::to_string(&reg_child).unwrap() + "\n").as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    // Allow time for daemon socket task to process registration messages
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Window creation for parent
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated { con_id: 2000, app_id: "agent.pi.parent-audit".to_string(), pid: Some(test_ppid as i32) })).await;
    // Window creation for child (pass child PID = std::process::id())
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated { con_id: 2001, app_id: "agent.pi.child-audit".to_string(), pid: Some(std::process::id() as i32) })).await;

    // Expecting parent split, and tab layout on child
    let mut split_received = false;
    let mut mark_children_received = false;

    for _ in 0..7 {
        if let Some(cmd) = recv_command(&mut sway_cmd_rx, 500).await {
            match cmd {
                SwayCommand::SplitHorizontal { parent_mark } => {
                    if parent_mark == "mark_parent_audit" {
                        split_received = true;
                    }
                }
                SwayCommand::MarkWindow { con_id, mark }
                    if mark == "mark_children_parent_audit" && con_id == 2001 => {
                        mark_children_received = true;
                    }
                _ => {}
            }
        }
    }

    assert!(split_received, "SplitHorizontal not received for parent-audit");
    assert!(mark_children_received, "MarkWindow for child-audit not received");

    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

#[tokio::test]
async fn test_cli_parent_flag_to_layout() {
    let socket_path = "/tmp/swayve_test_parent_cli.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx_listener).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    // Step 2: Register parent agent via socket
    let reg_parent = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("parent-cli".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-cli".to_string()),
            pid: 200,
            app_id: "agent.pi.parent-cli".to_string(),
            parent_agent_id: None,
        },
    };
    stream.write_all((serde_json::to_string(&reg_parent).unwrap() + "\n").as_bytes()).await.unwrap();

    // Step 3: Register child agent via socket with parent_agent_id
    let reg_child = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: AgentId("child-cli".to_string()),
            agent_type: "pi".to_string(),
            project_id: ProjectId("proj-cli".to_string()),
            pid: 201,
            app_id: "agent.pi.child-cli".to_string(),
            parent_agent_id: Some(AgentId("parent-cli".to_string())),
        },
    };
    stream.write_all((serde_json::to_string(&reg_child).unwrap() + "\n").as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Step 4: Send synthetic window-created event for parent
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated {
        con_id: 3000,
        app_id: "agent.pi.parent-cli".to_string(),
        pid: None,
    })).await;

    // Step 5: Send synthetic window-created event for child
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated {
        con_id: 3001,
        app_id: "agent.pi.child-cli".to_string(),
        pid: None,
    })).await;

    // Step 6: Verify LayoutManager hierarchy commands
    let mut split_received = false;
    let mut mark_children_received = false;
    let mut layout_tabbed_received = false;

    for _ in 0..7 {
        if let Some(cmd) = recv_command(&mut sway_cmd_rx, 500).await {
            match cmd {
                SwayCommand::SplitHorizontal { parent_mark } => {
                    if parent_mark == "mark_parent_cli" {
                        split_received = true;
                    }
                }
                SwayCommand::MarkWindow { con_id, mark } => {
                    if mark == "mark_children_parent_cli" && con_id == 3001 {
                        mark_children_received = true;
                    }
                }
                SwayCommand::SetLayoutTabbed { mark }
                    if mark == "mark_children_parent_cli" => {
                        layout_tabbed_received = true;
                    }
                _ => {}
            }
        }
    }

    assert!(split_received, "SplitHorizontal not received for parent-cli");
    assert!(mark_children_received, "MarkWindow for child-cli not received");
    assert!(layout_tabbed_received, "SetLayoutTabbed not received for children container");

    // Step 7: Clean up
    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

#[tokio::test]
async fn test_focus_survives_parent_destruction() {
    let socket_path = "/tmp/swayve_test_focus_orphan.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx_listener).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    let agents = [
        ("parent-doomed", None),
        ("child-x", Some("parent-doomed")),
        ("child-y", Some("parent-doomed")),
    ];
    for (idx, (agent_id, parent)) in agents.iter().enumerate() {
        let reg = AgentIPCMessage::Register {
            id: None,
            params: RegisterParams {
                agent_id: AgentId(agent_id.to_string()),
                agent_type: "pi".to_string(),
                project_id: ProjectId("proj-orphan".to_string()),
                pid: 400 + idx as u32,
                app_id: format!("agent.pi.{}", agent_id),
                parent_agent_id: parent.map(|p| AgentId(p.to_string())),
            },
        };
        stream.write_all((serde_json::to_string(&reg).unwrap() + "\n").as_bytes()).await.unwrap();
    }
    stream.flush().await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    for (idx, (agent_id, _)) in agents.iter().enumerate() {
        let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated {
            con_id: 5000 + idx as i64,
            app_id: format!("agent.pi.{}", agent_id),
            pid: None,
        })).await;
    }

    // The parent's window dies; its children are left behind
    let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowDestroyed { con_id: 5000 })).await;

    while recv_command(&mut sway_cmd_rx, 300).await.is_some() {}

    // Focus must still move, and only ever onto the surviving children
    for _ in 0..2 {
        let payload = serde_json::to_string(&AgentIPCMessage::Focus).unwrap() + "\n";
        stream.write_all(payload.as_bytes()).await.unwrap();
    }
    stream.flush().await.unwrap();

    let mut focused = Vec::new();
    for _ in 0..2 {
        match recv_command(&mut sway_cmd_rx, 500).await {
            Some(SwayCommand::FocusWindow { mark }) => focused.push(mark),
            other => panic!("focus stopped working after the parent died: got {:?}", other),
        }
    }

    assert_eq!(focused, vec!["mark_child_x", "mark_child_y"]);

    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

#[tokio::test]
async fn test_focus_follows_sway_focus() {
    let socket_path = "/tmp/swayve_test_focus_follows.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx_listener).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    let agents = [
        ("parent-f", None),
        ("child-a", Some("parent-f")),
        ("child-b", Some("parent-f")),
    ];
    for (idx, (agent_id, parent)) in agents.iter().enumerate() {
        let reg = AgentIPCMessage::Register {
            id: None,
            params: RegisterParams {
                agent_id: AgentId(agent_id.to_string()),
                agent_type: "pi".to_string(),
                project_id: ProjectId("proj-follow".to_string()),
                pid: 500 + idx as u32,
                app_id: format!("agent.pi.{}", agent_id),
                parent_agent_id: parent.map(|p| AgentId(p.to_string())),
            },
        };
        stream.write_all((serde_json::to_string(&reg).unwrap() + "\n").as_bytes()).await.unwrap();
    }
    stream.flush().await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    for (idx, (agent_id, _)) in agents.iter().enumerate() {
        let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated {
            con_id: 6000 + idx as i64,
            app_id: format!("agent.pi.{}", agent_id),
            pid: None,
        })).await;
    }

    while recv_command(&mut sway_cmd_rx, 300).await.is_some() {}

    let focus_payload = serde_json::to_string(&AgentIPCMessage::Focus).unwrap() + "\n";
    let mut focused = Vec::new();
    for step in 0..4 {
        match step {
            // Manual focus on child-b: the cycle continues from there
            2 => {
                let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowFocused { con_id: 6002 })).await;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            // Not an agent: ignored
            3 => {
                let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowFocused { con_id: 9999 })).await;
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            _ => {}
        }
        stream.write_all(focus_payload.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
        match recv_command(&mut sway_cmd_rx, 500).await {
            Some(SwayCommand::FocusWindow { mark }) => focused.push(mark),
            other => panic!("step {}: expected FocusWindow, got {:?}", step, other),
        }
    }

    assert_eq!(
        focused,
        vec!["mark_parent_f", "mark_child_a", "mark_parent_f", "mark_child_a"]
    );

    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

#[tokio::test]
async fn test_cli_focus_with_hierarchy() {
    let socket_path = "/tmp/swayve_test_focus_hierarchy.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx_listener).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    // A parent with two children, registered over the socket like the CLI does
    let agents = [
        ("parent-focus", None),
        ("child-a-focus", Some("parent-focus")),
        ("child-b-focus", Some("parent-focus")),
    ];
    for (idx, (agent_id, parent)) in agents.iter().enumerate() {
        let reg = AgentIPCMessage::Register {
            id: None,
            params: RegisterParams {
                agent_id: AgentId(agent_id.to_string()),
                agent_type: "pi".to_string(),
                project_id: ProjectId("proj-focus".to_string()),
                pid: 300 + idx as u32,
                app_id: format!("agent.pi.{}", agent_id),
                parent_agent_id: parent.map(|p| AgentId(p.to_string())),
            },
        };
        stream.write_all((serde_json::to_string(&reg).unwrap() + "\n").as_bytes()).await.unwrap();
    }
    stream.flush().await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Give each agent a window, ascending con_id so focus order is deterministic
    for (idx, (agent_id, _)) in agents.iter().enumerate() {
        let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowCreated {
            con_id: 4000 + idx as i64,
            app_id: format!("agent.pi.{}", agent_id),
            pid: None,
        })).await;
    }

    // Discard the layout commands; this test is only about focus
    while recv_command(&mut sway_cmd_rx, 300).await.is_some() {}

    // Four focus requests over the socket, exercising the daemon.focus route
    for _ in 0..4 {
        let payload = serde_json::to_string(&AgentIPCMessage::Focus).unwrap() + "\n";
        stream.write_all(payload.as_bytes()).await.unwrap();
    }
    stream.flush().await.unwrap();

    let mut focused = Vec::new();
    for _ in 0..4 {
        match recv_command(&mut sway_cmd_rx, 500).await {
            Some(SwayCommand::FocusWindow { mark }) => focused.push(mark),
            other => panic!("expected FocusWindow, got {:?}", other),
        }
    }

    // Parent descends to its first child, cycles siblings, then wraps back to the parent
    assert_eq!(
        focused,
        vec![
            "mark_parent_focus",
            "mark_child_a_focus",
            "mark_child_b_focus",
            "mark_parent_focus",
        ]
    );

    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

#[tokio::test]
async fn test_multiple_agents_same_wave_unique_app_id() {
    let socket_path = "/tmp/swayve_test_unique_appid.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, mut sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect to test socket");

    // Agent 1 spawned in wave1
    let agent1_id = AgentId("worker-wave1-10001".to_string());
    let reg1 = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: agent1_id.clone(),
            agent_type: "worker".to_string(),
            project_id: ProjectId("test-proj".to_string()),
            pid: 11001,
            app_id: "agent.worker.wave1.10001".to_string(),
            parent_agent_id: None,
        },
    };

    // Agent 2 spawned in wave1 (distinct suffix and app_id)
    let agent2_id = AgentId("worker-wave1-10002".to_string());
    let reg2 = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: agent2_id.clone(),
            agent_type: "worker".to_string(),
            project_id: ProjectId("test-proj".to_string()),
            pid: 11002,
            app_id: "agent.worker.wave1.10002".to_string(),
            parent_agent_id: None,
        },
    };

    stream.write_all((serde_json::to_string(&reg1).unwrap() + "\n").as_bytes()).await.expect("Failed to write reg1");
    stream.write_all((serde_json::to_string(&reg2).unwrap() + "\n").as_bytes()).await.expect("Failed to write reg2");

    // Update status for Agent 1 and Agent 2
    let status1 = AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: agent1_id.clone(),
            status: AgentStatus::AwaitingHumanInput,
        },
    };
    let status2 = AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: agent2_id.clone(),
            status: AgentStatus::AwaitingHumanInput,
        },
    };

    stream.write_all((serde_json::to_string(&status1).unwrap() + "\n").as_bytes()).await.expect("Failed to write status1");
    stream.write_all((serde_json::to_string(&status2).unwrap() + "\n").as_bytes()).await.expect("Failed to write status2");
    stream.flush().await.unwrap();

    let mut border_marks = Vec::new();
    let mut focus_marks = Vec::new();

    for _ in 0..4 {
        if let Some(cmd) = recv_command(&mut sway_cmd_rx, 500).await {
            match cmd {
                SwayCommand::SetBorderColor { mark, hex_color: _ } => {
                    border_marks.push(mark);
                }
                SwayCommand::FocusWindow { mark } => {
                    focus_marks.push(mark);
                }
                _ => {}
            }
        }
    }

    assert!(border_marks.contains(&"mark_worker_wave1_10001".to_string()));
    assert!(border_marks.contains(&"mark_worker_wave1_10002".to_string()));
    assert!(focus_marks.contains(&"mark_worker_wave1_10001".to_string()));
    assert!(focus_marks.contains(&"mark_worker_wave1_10002".to_string()));

    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}

#[tokio::test]
async fn test_hooks_and_monitoring() {
    use std::os::unix::fs::PermissionsExt;

    let socket_path = "/tmp/swayve_test_hooks_mon.sock";
    let socket_p = Path::new(socket_path);
    if socket_p.exists() {
        let _ = fs::remove_file(socket_p).await;
    }

    let spawn_flag_path = "/tmp/swayve_test_spawn_flag.txt";
    let spawn_p = Path::new(spawn_flag_path);
    if spawn_p.exists() {
        let _ = fs::remove_file(spawn_p).await;
    }

    let monitor_flag_path = "/tmp/swayve_test_monitor_flag.txt";
    let monitor_p = Path::new(monitor_flag_path);
    if monitor_p.exists() {
        let _ = fs::remove_file(monitor_p).await;
    }

    let mock_term_path = "/tmp/swayve_test_mock_term.sh";
    let mock_term_p = Path::new(mock_term_path);
    if mock_term_p.exists() {
        let _ = fs::remove_file(mock_term_p).await;
    }

    // Create mock terminal emulator script that writes its arguments to monitor_flag_path
    let mock_term_content = format!("#!/bin/sh\necho \"$@\" > {}\n", monitor_flag_path);
    fs::write(mock_term_path, mock_term_content).await.expect("Failed to write mock term script");
    let mut perms = fs::metadata(mock_term_path).await.unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(mock_term_path, perms).await.expect("Failed to set mock term permissions");

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, _sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx).await
    });

    let mut config = Config::default();
    config.hooks.on_agent_spawn = Some(format!("echo \"$SWAYVE_AGENT_ID\" >> {}", spawn_flag_path));
    config.daemon.auto_monitor = true;
    config.terminal.default_emulator = mock_term_path.to_string();

    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect to socket");

    // 1. Register Parent Agent
    let parent_id = AgentId("parent-hooks-1".to_string());
    let reg_parent = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: parent_id.clone(),
            agent_type: "pi".to_string(),
            project_id: ProjectId("test-hooks-proj".to_string()),
            pid: 20001,
            app_id: "agent.pi.parent-hooks-1".to_string(),
            parent_agent_id: None,
        },
    };
    stream.write_all((serde_json::to_string(&reg_parent).unwrap() + "\n").as_bytes()).await.expect("Failed to write reg_parent");

    // 2. Register Subagent
    let subagent_id = AgentId("subagent-hooks-1".to_string());
    let reg_sub = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: subagent_id.clone(),
            agent_type: "pi".to_string(),
            project_id: ProjectId("test-hooks-proj".to_string()),
            pid: 20002,
            app_id: "agent.pi.subagent-hooks-1".to_string(),
            parent_agent_id: Some(parent_id.clone()),
        },
    };
    stream.write_all((serde_json::to_string(&reg_sub).unwrap() + "\n").as_bytes()).await.expect("Failed to write reg_sub");
    stream.flush().await.unwrap();

    // Wait a brief moment for spawn hooks and auto_monitor process execution
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Verify on_agent_spawn hook executed for both agents
    let spawn_flag_content = fs::read_to_string(spawn_flag_path)
        .await
        .expect("spawn flag file was not created by lifecycle hook");
    assert!(
        spawn_flag_content.contains("parent-hooks-1"),
        "Spawn flag missing parent-hooks-1"
    );
    assert!(
        spawn_flag_content.contains("subagent-hooks-1"),
        "Spawn flag missing subagent-hooks-1"
    );

    // Verify spawn_monitor_terminal launched mock terminal for subagent
    let monitor_flag_content = fs::read_to_string(monitor_flag_path)
        .await
        .expect("monitor flag file was not created by auto_monitor process");
    assert!(
        monitor_flag_content.contains("agent.monitor.subagent-hooks-1"),
        "Monitor flag content missing monitor app_id: {}",
        monitor_flag_content
    );
    assert!(
        monitor_flag_content.contains(&format!(
            "tail -f {}/swayve/logs/subagent-hooks-1.log",
            std::env::var("XDG_RUNTIME_DIR").unwrap()
        )),
        "Monitor flag content missing log tail command: {}",
        monitor_flag_content
    );

    // Clean up
    listener_handle.abort();
    daemon_handle.abort();
    let _ = fs::remove_file(socket_p).await;
    let _ = fs::remove_file(spawn_p).await;
    let _ = fs::remove_file(monitor_p).await;
    let _ = fs::remove_file(mock_term_p).await;
}

#[tokio::test]
async fn test_dashboard_subscription_stream() {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use swayve::types::AgentWindow;

    let socket_path = "/tmp/swayve_test_dashboard_stream.sock";
    let path = Path::new(socket_path);
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, _sway_cmd_rx) = mpsc::channel::<SwayCommand>(32);

    let socket_path_str = socket_path.to_string();
    let listener_handle = tokio::spawn(async move {
        start_daemon_ipc(&socket_path_str, event_tx).await
    });

    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });

    wait_for_socket(socket_path).await;

    // Connect subscriber stream and subscribe to status updates
    let mut sub_stream = UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect subscriber stream to socket");
    let sub_msg = AgentIPCMessage::SubscribeStatus;
    let sub_payload = serde_json::to_string(&sub_msg).unwrap() + "\n";
    sub_stream
        .write_all(sub_payload.as_bytes())
        .await
        .expect("Failed to send SubscribeStatus");
    sub_stream.flush().await.unwrap();

    let (sub_reader, _) = sub_stream.into_split();
    let mut sub_lines = BufReader::new(sub_reader).lines();

    // 1. First broadcast line should be sent immediately upon subscription (empty list of agents)
    let line1 = timeout(Duration::from_secs(2), sub_lines.next_line())
        .await
        .expect("Timeout waiting for initial subscriber status broadcast")
        .expect("Stream closed unexpectedly")
        .expect("Failed to read line 1");

    let agents1: Vec<AgentWindow> = serde_json::from_str(&line1).expect("Failed to parse initial agent list JSON");
    assert!(agents1.is_empty(), "Initial agent list should be empty");

    // 2. Connect client stream and register an agent
    let mut client_stream = UnixStream::connect(socket_path)
        .await
        .expect("Failed to connect client stream to socket");

    let test_agent_id = AgentId("dash-agent-1".to_string());
    let reg_msg = AgentIPCMessage::Register {
        id: None,
        params: RegisterParams {
            agent_id: test_agent_id.clone(),
            agent_type: "pi".to_string(),
            project_id: ProjectId("dash-proj".to_string()),
            pid: 30001,
            app_id: "agent.pi.dash-agent-1".to_string(),
            parent_agent_id: None,
        },
    };
    let reg_payload = serde_json::to_string(&reg_msg).unwrap() + "\n";
    client_stream
        .write_all(reg_payload.as_bytes())
        .await
        .expect("Failed to write register message");
    client_stream.flush().await.unwrap();

    // 3. Second broadcast line should contain the newly registered agent
    let line2 = timeout(Duration::from_secs(2), sub_lines.next_line())
        .await
        .expect("Timeout waiting for registration subscriber status broadcast")
        .expect("Stream closed unexpectedly")
        .expect("Failed to read line 2");

    let agents2: Vec<AgentWindow> = serde_json::from_str(&line2).expect("Failed to parse registration agent list JSON");
    assert_eq!(agents2.len(), 1, "Should have exactly 1 registered agent");
    assert_eq!(agents2[0].id, test_agent_id);
    assert_eq!(agents2[0].status, AgentStatus::Idle);

    // 4. Update status for registered agent
    let status_msg = AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: test_agent_id.clone(),
            status: AgentStatus::AwaitingHumanInput,
        },
    };
    let status_payload = serde_json::to_string(&status_msg).unwrap() + "\n";
    client_stream
        .write_all(status_payload.as_bytes())
        .await
        .expect("Failed to write status message");
    client_stream.flush().await.unwrap();

    // 5. Third broadcast line should reflect the updated status
    let line3 = timeout(Duration::from_secs(2), sub_lines.next_line())
        .await
        .expect("Timeout waiting for status update subscriber broadcast")
        .expect("Stream closed unexpectedly")
        .expect("Failed to read line 3");

    let agents3: Vec<AgentWindow> = serde_json::from_str(&line3).expect("Failed to parse status update agent list JSON");
    assert_eq!(agents3.len(), 1, "Should have 1 agent after status update");
    assert_eq!(agents3[0].id, test_agent_id);
    assert_eq!(agents3[0].status, AgentStatus::AwaitingHumanInput);

    // Clean up
    listener_handle.abort();
    daemon_handle.abort();
    if path.exists() {
        let _ = fs::remove_file(path).await;
    }
}




#[tokio::test]
async fn test_diff_stream_matches_daemon_state() {
    use std::collections::HashMap;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let socket_path = "/tmp/swayve_test_diff_stream.sock";
    let _ = fs::remove_file(socket_path).await;

    let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(32);
    let (sway_cmd_tx, _sway_cmd_rx) = mpsc::channel::<SwayCommand>(64);

    let ipc_tx = event_tx.clone();
    let listener_handle = tokio::spawn(async move { start_daemon_ipc(socket_path, ipc_tx).await });
    let config = Config::default();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_loops(true, config, socket_path, event_rx, sway_cmd_tx).await
    });
    wait_for_socket(socket_path).await;

    let mut diff_stream = UnixStream::connect(socket_path).await.expect("connect diff client");
    diff_stream
        .write_all(b"{\"method\":\"daemon.subscribe_diff\"}\n")
        .await
        .unwrap();
    let (diff_reader, _) = diff_stream.into_split();
    let mut diff_lines = BufReader::new(diff_reader).lines();

    let mut client = UnixStream::connect(socket_path).await.expect("connect client");
    let mut send = |msg: AgentIPCMessage| serde_json::to_string(&msg).unwrap() + "\n";
    for (i, n) in [1, 2, 3].iter().enumerate() {
        let id = format!("diff-a{n}");
        let payload = send(AgentIPCMessage::Register {
            id: None,
            params: RegisterParams {
                agent_id: AgentId(id.clone()),
                agent_type: "pi".to_string(),
                project_id: ProjectId("diff-proj".to_string()),
                pid: 31000 + i as u32,
                app_id: format!("agent.pi.{id}"),
                parent_agent_id: None,
            },
        });
        client.write_all(payload.as_bytes()).await.unwrap();
    }
    let payload = send(AgentIPCMessage::UpdateStatus {
        params: UpdateStatusParams {
            agent_id: AgentId("diff-a2".to_string()),
            status: AgentStatus::AwaitingHumanInput,
        },
    });
    client.write_all(payload.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    // Replay the diff stream until it converges (bounded; every read has a timeout).
    let mut map: HashMap<String, serde_json::Value> = HashMap::new();
    let mut converged = false;
    let mut removal_sent = false;
    for _ in 0..20 {
        let line = timeout(Duration::from_secs(2), diff_lines.next_line())
            .await
            .expect("Timeout waiting for diff line")
            .expect("read error")
            .expect("diff stream closed");
        let v: serde_json::Value = serde_json::from_str(&line).expect("diff line is JSON");
        if v["snapshot"].as_bool().expect("snapshot flag") {
            map.clear();
        }
        for a in v["upsert"].as_array().expect("upsert array") {
            map.insert(a["id"].as_str().unwrap().to_string(), a.clone());
        }
        for id in v["remove"].as_array().expect("remove array") {
            map.remove(id.as_str().unwrap());
        }
        // Once a3 is registered (seen in the stream), bind it to a window and destroy it.
        if !removal_sent && map.contains_key("diff-a3") {
            removal_sent = true;
            let _ = event_tx
                .send(DaemonEvent::Sway(SwayEvent::WindowCreated {
                    con_id: 3000,
                    app_id: "agent.pi.diff-a3".to_string(),
                    pid: None,
                }))
                .await;
            let _ = event_tx.send(DaemonEvent::Sway(SwayEvent::WindowDestroyed { con_id: 3000 })).await;
        }
        if map.len() == 2
            && !map.contains_key("diff-a3")
            && map.get("diff-a2").is_some_and(|a| a["status"] == serde_json::json!(AgentStatus::AwaitingHumanInput))
        {
            converged = true;
            break;
        }
    }
    assert!(converged, "diff stream never converged; map = {map:?}");

    // Daemon state: first line of a fresh full `daemon.subscribe` stream.
    let mut full_stream = UnixStream::connect(socket_path).await.expect("connect full client");
    full_stream.write_all(b"{\"method\":\"daemon.subscribe\"}\n").await.unwrap();
    let (full_reader, _) = full_stream.into_split();
    let mut full_lines = BufReader::new(full_reader).lines();
    let line = timeout(Duration::from_secs(2), full_lines.next_line())
        .await
        .expect("Timeout waiting for full agent list")
        .expect("read error")
        .expect("full stream closed");
    let mut daemon_list: Vec<serde_json::Value> = serde_json::from_str(&line).expect("agent list JSON");
    daemon_list.sort_by_key(|a| a["id"].as_str().unwrap().to_string());
    let mut client_list: Vec<serde_json::Value> = map.into_values().collect();
    client_list.sort_by_key(|a| a["id"].as_str().unwrap().to_string());
    assert_eq!(client_list, daemon_list);

    listener_handle.abort();
    daemon_handle.abort();
    let _ = fs::remove_file(socket_path).await;
}
