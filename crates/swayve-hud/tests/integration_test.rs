use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use swayve_hud::client::{
    connect_and_subscribe, process_stream, render_offline, run_client_with_options, ClientOptions,
    SUBSCRIBE_DIFF_MSG,
};
use swayve_hud::types::{AgentStatus, AgentWindow};

/// RAII guard to create and clean up temporary Unix socket paths.
struct TempSocket {
    path: PathBuf,
}

impl TempSocket {
    /// Generates a unique temporary socket path and removes any preexisting socket file.
    fn new(test_name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("swayve_hud_{test_name}_{pid}_{id}.sock"));
        let _ = fs::remove_file(&path);
        Self { path }
    }

    /// Returns a reference to the temporary socket file path.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempSocket {
    /// Removes the socket file from the filesystem on drop.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Helper to read and verify the subscribe_diff handshake from a stream.
fn verify_handshake(stream: &UnixStream) {
    let mut reader = BufReader::new(stream.try_clone().expect("failed to clone stream"));
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("failed to read handshake");
    assert_eq!(line, SUBSCRIBE_DIFF_MSG);
}

/// Tests mock daemon handshake transmission and processing of snapshot, upsert, and remove diff frames.
#[test]
fn test_mock_daemon_handshake_and_frames() {
    let temp_socket = TempSocket::new("test_handshake_and_frames");
    let listener = UnixListener::bind(temp_socket.path()).expect("failed to bind mock listener");

    let server_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("failed to accept connection");
        verify_handshake(&stream);

        // 1. Initial snapshot frame
        let snapshot = r#"{"snapshot":true,"upsert":[{"id":"agent-1","project":"proj1","app_id":"ghostty","mark":"mark1","status":"thinking"}],"remove":[]}"#;
        writeln!(stream, "{snapshot}").expect("failed to write snapshot");
        stream.flush().expect("failed to flush snapshot");

        // 2. Incremental upsert frame
        let upsert = r#"{"snapshot":false,"upsert":[{"id":"agent-2","project":"proj2","app_id":"foot","mark":"mark2","status":"idle"}],"remove":[]}"#;
        writeln!(stream, "{upsert}").expect("failed to write upsert");
        stream.flush().expect("failed to flush upsert");

        // 3. Remove frame
        let remove = r#"{"snapshot":false,"upsert":[],"remove":["agent-1"]}"#;
        writeln!(stream, "{remove}").expect("failed to write remove");
        stream.flush().expect("failed to flush remove");

        // Close connection on EOF
    });

    let client_stream =
        connect_and_subscribe(temp_socket.path()).expect("failed to connect and subscribe");
    let mut map = HashMap::new();
    let mut observed_states = Vec::new();

    let frames_processed = process_stream(
        BufReader::new(client_stream),
        &mut map,
        |current_map| {
            observed_states.push(current_map.clone());
            Ok(())
        },
    )
    .expect("stream processing failed");

    assert_eq!(frames_processed, 3);
    assert_eq!(observed_states.len(), 3);

    // State after snapshot
    assert_eq!(observed_states[0].len(), 1);
    assert!(observed_states[0].contains_key("agent-1"));
    assert_eq!(
        observed_states[0]["agent-1"].status,
        AgentStatus::Thinking
    );

    // State after upsert
    assert_eq!(observed_states[1].len(), 2);
    assert!(observed_states[1].contains_key("agent-1"));
    assert!(observed_states[1].contains_key("agent-2"));
    assert_eq!(observed_states[1]["agent-2"].status, AgentStatus::Idle);

    // State after remove
    assert_eq!(observed_states[2].len(), 1);
    assert!(!observed_states[2].contains_key("agent-1"));
    assert!(observed_states[2].contains_key("agent-2"));

    // Final map state matches last update
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("agent-2"));

    server_handle.join().expect("server thread failed");
}

/// Tests reconnect loop recovery after daemon disconnection, verifying clean state reset with a fresh snapshot.
#[test]
fn test_disconnect_and_reconnect_recovery() {
    let temp_socket = TempSocket::new("test_reconnect_recovery");
    let listener = UnixListener::bind(temp_socket.path()).expect("failed to bind mock listener");

    let server_handle = thread::spawn(move || {
        // Connection 1: send snapshot then disconnect
        {
            let (mut stream1, _) = listener
                .accept()
                .expect("failed to accept first connection");
            verify_handshake(&stream1);

            let snap1 = r#"{"snapshot":true,"upsert":[{"id":"agent-old","project":"p1","app_id":"term","mark":"m1","status":"executing_tool"}],"remove":[]}"#;
            writeln!(stream1, "{snap1}").expect("failed to write snap1");
            stream1.flush().expect("failed to flush snap1");
            // stream1 drops here, closing connection to client
        }

        // Brief pause between connections
        thread::sleep(Duration::from_millis(50));

        // Connection 2: fresh connection after daemon restart
        {
            let (mut stream2, _) = listener
                .accept()
                .expect("failed to accept second connection");
            verify_handshake(&stream2);

            let snap2 = r#"{"snapshot":true,"upsert":[{"id":"agent-new","project":"p2","app_id":"ghostty","mark":"m2","status":"thinking"}],"remove":[]}"#;
            writeln!(stream2, "{snap2}").expect("failed to write snap2");
            stream2.flush().expect("failed to flush snap2");
            // stream2 drops here
        }
    });

    let options = ClientOptions {
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(50),
        max_reconnect_attempts: Some(3),
        reset_backoff_on_frame: true,
    };

    let history: Arc<Mutex<Vec<HashMap<String, AgentWindow>>>> =
        Arc::new(Mutex::new(Vec::new()));
    let history_clone = Arc::clone(&history);

    let client_res = run_client_with_options(
        temp_socket.path(),
        &options,
        move |current_map| {
            history_clone
                .lock()
                .expect("lock poisoned")
                .push(current_map.clone());
            Ok(())
        },
        || Ok(()),
    );

    assert!(client_res.is_ok());
    server_handle.join().expect("server thread failed");

    let updates = history.lock().expect("lock poisoned");
    assert!(
        updates.len() >= 2,
        "expected at least 2 updates across reconnects, got {}",
        updates.len()
    );

    // Initial connection saw agent-old
    assert!(updates[0].contains_key("agent-old"));
    assert_eq!(
        updates[0]["agent-old"].status,
        AgentStatus::ExecutingTool
    );

    // After reconnect and fresh snapshot, map cleared and has agent-new
    let final_state = updates.last().unwrap();
    assert!(
        !final_state.contains_key("agent-old"),
        "fresh snapshot must clear pre-existing agent-old"
    );
    assert!(
        final_state.contains_key("agent-new"),
        "fresh snapshot must include agent-new"
    );
    assert_eq!(final_state["agent-new"].status, AgentStatus::Thinking);
}

/// Tests that disconnecting from the daemon triggers offline state rendering with cleared active agents.
#[test]
fn test_disconnect_renders_offline_state() {
    let temp_socket = TempSocket::new("test_disconnect_offline");
    let listener = UnixListener::bind(temp_socket.path()).expect("failed to bind mock listener");

    let server_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("failed to accept connection");
        verify_handshake(&stream);

        let snap = r#"{"snapshot":true,"upsert":[{"id":"agent-old","project":"p1","app_id":"term","mark":"m1","status":"executing_tool"}],"remove":[]}"#;
        writeln!(stream, "{snap}").expect("failed to write snapshot");
        stream.flush().expect("failed to flush snapshot");
        // stream and listener drop here; later connects get ECONNREFUSED
    });

    let options = ClientOptions {
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(50),
        max_reconnect_attempts: Some(2),
        reset_backoff_on_frame: true,
    };

    let updates: Arc<Mutex<Vec<HashMap<String, AgentWindow>>>> =
        Arc::new(Mutex::new(Vec::new()));
    let offline_output: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    let updates_clone = Arc::clone(&updates);
    let offline_output_clone = Arc::clone(&offline_output);

    let client_res = run_client_with_options(
        temp_socket.path(),
        &options,
        move |current_map| {
            updates_clone
                .lock()
                .expect("lock poisoned")
                .push(current_map.clone());
            Ok(())
        },
        move || render_offline(&mut *offline_output_clone.lock().expect("lock poisoned")),
    );

    assert!(client_res.is_ok());
    server_handle.join().expect("server thread failed");

    let updates = updates.lock().expect("lock poisoned");
    assert!(
        updates.iter().any(|m| m.contains_key("agent-old")),
        "expected an update containing agent-old"
    );

    let output = String::from_utf8_lossy(&offline_output.lock().expect("lock poisoned")).into_owned();
    assert!(output.contains("daemon unreachable"));
    let last_render = output
        .rsplit("\x1b[2J\x1b[H")
        .next()
        .expect("no render output");
    assert!(!last_render.contains("agent-old"));
}

/// Tests that encountering a closed output/stdout stream halts the client immediately rather than looping.
#[test]
fn test_output_closed_stops_client() {
    let temp_socket = TempSocket::new("test_output_closed");
    let listener = UnixListener::bind(temp_socket.path()).expect("failed to bind mock listener");

    let server_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("failed to accept connection");
        verify_handshake(&stream);

        let snap = r#"{"snapshot":true,"upsert":[{"id":"agent-1","project":"p1","app_id":"term","mark":"m1","status":"idle"}],"remove":[]}"#;
        writeln!(stream, "{snap}").expect("failed to write snapshot");
        stream.flush().expect("failed to flush snapshot");
    });

    let options = ClientOptions {
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(50),
        max_reconnect_attempts: Some(5),
        reset_backoff_on_frame: true,
    };

    // Simulates stdout closed: every output write fails.
    let result = run_client_with_options(
        temp_socket.path(),
        &options,
        |_| Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "closed")),
        || Ok(()),
    );

    assert!(result.is_err(), "a closed output must stop the client, not retry");
    server_handle.join().expect("server thread failed");
}

/// Tests that invalid JSON and blank lines in the stream are silently ignored without disrupting valid diff frames.
#[test]
fn test_non_json_lines_ignored_by_mock_stream() {
    let temp_socket = TempSocket::new("test_non_json_ignored");
    let listener = UnixListener::bind(temp_socket.path()).expect("failed to bind mock listener");

    let server_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("failed to accept connection");
        verify_handshake(&stream);

        // Mix valid frames with invalid / non-JSON lines
        writeln!(stream, "plain non-json line").unwrap();
        writeln!(stream, "{{broken json: true").unwrap();
        writeln!(stream).unwrap();

        let snapshot = r#"{"snapshot":true,"upsert":[{"id":"agent-1","project":"p1","app_id":"term","mark":"m1","status":"idle"}],"remove":[]}"#;
        writeln!(stream, "{snapshot}").unwrap();

        writeln!(stream, "another invalid text line").unwrap();

        let upsert = r#"{"snapshot":false,"upsert":[{"id":"agent-2","project":"p1","app_id":"term","mark":"m2","status":"thinking"}],"remove":[]}"#;
        writeln!(stream, "{upsert}").unwrap();

        stream.flush().unwrap();
    });

    let client_stream =
        connect_and_subscribe(temp_socket.path()).expect("failed to connect and subscribe");
    let mut map = HashMap::new();
    let mut update_count = 0;

    let frames_processed = process_stream(
        BufReader::new(client_stream),
        &mut map,
        |_current_map| {
            update_count += 1;
            Ok(())
        },
    )
    .expect("stream processing failed");

    assert_eq!(frames_processed, 2);
    assert_eq!(update_count, 2);
    assert_eq!(map.len(), 2);
    assert!(map.contains_key("agent-1"));
    assert!(map.contains_key("agent-2"));

    server_handle.join().expect("server thread failed");
}
