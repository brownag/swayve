use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::fold::fold_diff_line;
use crate::types::AgentWindow;

/// The diff subscription handshake command line.
pub const SUBSCRIBE_DIFF_MSG: &str = "{\"method\":\"daemon.subscribe_diff\"}\n";

/// Default initial reconnect backoff delay (500ms).
pub const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_millis(500);

/// Default maximum reconnect backoff delay cap (5000ms).
pub const DEFAULT_MAX_BACKOFF: Duration = Duration::from_millis(5000);

/// Resolves the Swayve daemon socket path according to environment rules:
/// 1. `$SWAYVE_SOCKET` if set and non-empty.
/// 2. `$XDG_RUNTIME_DIR/swayve/swayve.sock` if `$XDG_RUNTIME_DIR` is set and non-empty.
///
/// Returns an error message if neither can be determined.
pub fn resolve_socket_path() -> Result<PathBuf, String> {
    resolve_socket_path_with(|key| env::var(key).ok())
}

/// Resolves socket path using a custom environment variable lookup function (testable).
pub fn resolve_socket_path_with<F>(var_lookup: F) -> Result<PathBuf, String>
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(socket) = var_lookup("SWAYVE_SOCKET")
        && !socket.trim().is_empty()
    {
        return Ok(PathBuf::from(socket));
    }

    if let Some(xdg) = var_lookup("XDG_RUNTIME_DIR")
        && !xdg.trim().is_empty()
    {
        return Ok(PathBuf::from(xdg).join("swayve").join("swayve.sock"));
    }

    Err("Could not determine socket path: neither SWAYVE_SOCKET nor XDG_RUNTIME_DIR is set and non-empty".to_string())
}

/// Connects to the given Unix socket and transmits the `daemon.subscribe_diff` handshake.
pub fn connect_and_subscribe(path: &Path) -> io::Result<UnixStream> {
    let mut stream = UnixStream::connect(path)?;
    stream.write_all(SUBSCRIBE_DIFF_MSG.as_bytes())?;
    stream.flush()?;
    Ok(stream)
}

/// Configuration options for the reconnect loop.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub max_reconnect_attempts: Option<usize>,
    pub reset_backoff_on_frame: bool,
}

impl Default for ClientOptions {
    /// Returns default reconnect options with standard initial and maximum backoff.
    fn default() -> Self {
        Self {
            initial_backoff: DEFAULT_INITIAL_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
            max_reconnect_attempts: None,
            reset_backoff_on_frame: true,
        }
    }
}

/// Renders the folded agent map to `out` with ANSI clear `\x1b[2J\x1b[H`.
pub fn render_hud<W: Write>(agents: &HashMap<String, AgentWindow>, out: &mut W) -> io::Result<()> {
    write!(out, "\x1b[2J\x1b[H")?;
    writeln!(out, "=== Swayve HUD (Live Agents: {}) ===", agents.len())?;

    if agents.is_empty() {
        writeln!(out, "No active agents.")?;
        out.flush()?;
        return Ok(());
    }

    let mut sorted_agents: Vec<&AgentWindow> = agents.values().collect();
    sorted_agents.sort_by(|a, b| a.id.cmp(&b.id));

    writeln!(
        out,
        "{:<24} {:<16} {:<16} {:<16} MARK",
        "STATUS", "ID", "APP_ID", "PROJECT"
    )?;
    writeln!(
        out,
        "{:<24} {:<16} {:<16} {:<16} ----",
        "------", "--", "------", "-------"
    )?;

    for agent in sorted_agents {
        writeln!(
            out,
            "{:<24} {:<16} {:<16} {:<16} {}",
            agent.status.to_string(),
            agent.id,
            agent.app_id,
            agent.project,
            agent.mark
        )?;
    }

    out.flush()?;
    Ok(())
}

/// Renders the offline state shown while the daemon is unreachable.
pub fn render_offline<W: Write>(out: &mut W) -> io::Result<()> {
    write!(out, "\x1b[2J\x1b[H")?;
    writeln!(out, "=== Swayve HUD (daemon unreachable, retrying) ===")?;
    writeln!(out, "No active agents.")?;
    out.flush()?;
    Ok(())
}

/// Reads lines from `reader`, folding valid diff frames into `map`.
/// Calls `on_update` only when a valid diff frame modifies or updates `map`.
/// Skips non-JSON or invalid lines silently without modifying state or calling `on_update`.
/// Returns the number of successfully processed frames on EOF.
pub fn process_stream<R, F>(
    reader: R,
    map: &mut HashMap<String, AgentWindow>,
    mut on_update: F,
) -> io::Result<usize>
where
    R: BufRead,
    F: FnMut(&HashMap<String, AgentWindow>) -> io::Result<()>,
{
    let mut frames_processed = 0;
    for line_res in reader.lines() {
        let line = line_res?;
        if line.trim().is_empty() {
            continue;
        }
        if fold_diff_line(map, &line) {
            frames_processed += 1;
            on_update(map)?;
        }
    }
    Ok(frames_processed)
}

/// Runs the client reconnect loop with the specified options and update handler.
pub fn run_client_with_options<F, G>(
    socket_path: &Path,
    options: &ClientOptions,
    mut on_update: F,
    mut on_offline: G,
) -> Result<(), String>
where
    F: FnMut(&HashMap<String, AgentWindow>) -> io::Result<()>,
    G: FnMut() -> io::Result<()>,
{
    let mut map = HashMap::new();
    let mut backoff = options.initial_backoff;
    let mut reconnect_count: usize = 0;

    loop {
        match connect_and_subscribe(socket_path) {
            Ok(stream) => {
                let reader = BufReader::new(stream);
                let mut frame_received = false;
                let mut output_err: Option<io::Error> = None;

                let _ = process_stream(reader, &mut map, |agents| {
                    if options.reset_backoff_on_frame && !frame_received {
                        backoff = options.initial_backoff;
                        frame_received = true;
                    }
                    on_update(agents).map_err(|e| {
                        output_err = Some(e);
                        io::Error::other("output closed")
                    })
                });
                if let Some(e) = output_err {
                    return Err(e.to_string());
                }
                map.clear();
                on_offline().map_err(|e| e.to_string())?;
            }
            Err(_) => {
                // Connection failed
                map.clear();
                on_offline().map_err(|e| e.to_string())?;
            }
        }

        reconnect_count += 1;
        if let Some(max_attempts) = options.max_reconnect_attempts
            && reconnect_count >= max_attempts
        {
            return Ok(());
        }

        std::thread::sleep(backoff);
        backoff = (backoff * 2).min(options.max_backoff);
    }
}

/// Runs the client loop using default reconnect options and standard ANSI output.
pub fn run_client(socket_path: &Path) -> Result<(), String> {
    let options = ClientOptions::default();
    run_client_with_options(
        socket_path,
        &options,
        |agents| render_hud(agents, &mut io::stdout()),
        || render_offline(&mut io::stdout()),
    )
}

/// Resolves socket path and runs the live HUD.
pub fn run_hud() -> Result<(), String> {
    let socket_path = resolve_socket_path()?;
    run_client(&socket_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentId, AgentStatus, ProjectId};
    use std::io::Cursor;

    #[test]
    fn test_resolve_socket_path_prefers_swayve_socket() {
        let lookup = |key: &str| match key {
            "SWAYVE_SOCKET" => Some("/run/user/1000/custom.sock".to_string()),
            "XDG_RUNTIME_DIR" => Some("/run/user/1000".to_string()),
            _ => None,
        };

        let resolved = resolve_socket_path_with(lookup).unwrap();
        assert_eq!(resolved, PathBuf::from("/run/user/1000/custom.sock"));
    }

    #[test]
    fn test_resolve_socket_path_falls_back_when_swayve_socket_empty() {
        let lookup = |key: &str| match key {
            "SWAYVE_SOCKET" => Some("   ".to_string()),
            "XDG_RUNTIME_DIR" => Some("/run/user/1000".to_string()),
            _ => None,
        };

        let resolved = resolve_socket_path_with(lookup).unwrap();
        assert_eq!(
            resolved,
            PathBuf::from("/run/user/1000").join("swayve").join("swayve.sock")
        );
    }

    #[test]
    fn test_resolve_socket_path_falls_back_when_swayve_socket_unset() {
        let lookup = |key: &str| match key {
            "SWAYVE_SOCKET" => None,
            "XDG_RUNTIME_DIR" => Some("/run/user/1000".to_string()),
            _ => None,
        };

        let resolved = resolve_socket_path_with(lookup).unwrap();
        assert_eq!(
            resolved,
            PathBuf::from("/run/user/1000").join("swayve").join("swayve.sock")
        );
    }

    #[test]
    fn test_resolve_socket_path_errors_when_neither_set() {
        let lookup_none = |_: &str| None;
        assert!(resolve_socket_path_with(lookup_none).is_err());

        let lookup_empty = |_: &str| Some("".to_string());
        assert!(resolve_socket_path_with(lookup_empty).is_err());
    }

    #[test]
    fn test_render_hud_empty() {
        let map = HashMap::new();
        let mut buf = Vec::new();
        render_hud(&map, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("\x1b[2J\x1b[H"));
        assert!(output.contains("=== Swayve HUD (Live Agents: 0) ==="));
        assert!(output.contains("No active agents."));
    }

    #[test]
    fn test_render_hud_sorted_agents() {
        let mut map = HashMap::new();
        map.insert(
            "agent-z".to_string(),
            AgentWindow {
                id: AgentId("agent-z".to_string()),
                project: ProjectId("proj-z".to_string()),
                parent_id: None,
                container_id: None,
                app_id: "foot".to_string(),
                mark: "mark_z".to_string(),
                status: AgentStatus::Completed,
            },
        );
        map.insert(
            "agent-a".to_string(),
            AgentWindow {
                id: AgentId("agent-a".to_string()),
                project: ProjectId("proj-a".to_string()),
                parent_id: None,
                container_id: None,
                app_id: "ghostty".to_string(),
                mark: "mark_a".to_string(),
                status: AgentStatus::Thinking,
            },
        );

        let mut buf = Vec::new();
        render_hud(&map, &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("\x1b[2J\x1b[H"));
        assert!(output.contains("=== Swayve HUD (Live Agents: 2) ==="));
        assert!(output.contains("STATUS"));
        assert!(output.contains("ID"));
        assert!(output.contains("APP_ID"));
        assert!(output.contains("PROJECT"));
        assert!(output.contains("MARK"));

        let idx_a = output.find("agent-a").unwrap();
        let idx_z = output.find("agent-z").unwrap();
        assert!(idx_a < idx_z, "agent-a should be rendered before agent-z");
    }

    #[test]
    fn test_process_stream_folds_and_skips_invalid() {
        let input = [
            // 1. Snapshot with agent-1
            r#"{"snapshot":true,"upsert":[{"id":"agent-1","project":"p1","app_id":"term","mark":"m1","status":"idle"}],"remove":[]}"#,
            // 2. Non-JSON line (must be skipped, no callback)
            r#"NOT VALID JSON"#,
            // 3. Upsert updating agent-1 and adding agent-2
            r#"{"snapshot":false,"upsert":[{"id":"agent-1","project":"p1","app_id":"term","mark":"m1","status":"thinking"},{"id":"agent-2","project":"p1","app_id":"term","mark":"m2","status":"executing_tool"}],"remove":[]}"#,
            // 4. Invalid DiffFrame (missing required fields)
            r#"{"snapshot":false,"upsert":"invalid_array"}"#,
            // 5. Remove agent-1
            r#"{"snapshot":false,"upsert":[],"remove":["agent-1"]}"#,
        ]
        .join("\n");

        let mut map = HashMap::new();
        let mut update_count = 0;

        let processed = process_stream(Cursor::new(input), &mut map, |_current_map| {
            update_count += 1;
            Ok(())
        })
        .unwrap();

        assert_eq!(processed, 3);
        assert_eq!(update_count, 3);
        assert_eq!(map.len(), 1);
        assert!(!map.contains_key("agent-1"));
        assert!(map.contains_key("agent-2"));
    }

    #[test]
    fn test_run_client_with_options_max_attempts() {
        let non_existent_path = std::env::temp_dir().join("non_existent_swayve_test.sock");
        let options = ClientOptions {
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(2),
            max_reconnect_attempts: Some(2),
            reset_backoff_on_frame: true,
        };

        let mut offline_count = 0;
        let result = run_client_with_options(&non_existent_path, &options, |_| Ok(()), || {
            offline_count += 1;
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(offline_count, 2);
    }

    #[test]
    fn test_run_client_with_options_offline_error_propagates() {
        let non_existent_path = PathBuf::from("/tmp/non_existent_swayve_test.sock");
        let options = ClientOptions {
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(2),
            max_reconnect_attempts: Some(5),
            reset_backoff_on_frame: true,
        };

        let result = run_client_with_options(&non_existent_path, &options, |_| Ok(()), || {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        });
        assert!(result.is_err());
    }
}
