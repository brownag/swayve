use crate::config::load_config;
use crate::daemon::{run_daemon_loops, start_daemon_ipc_with_limits, DaemonEvent};
use crate::sway::{run_event_listener, SwayCommand, SwayEvent, SwayManager};
use crate::types::{AgentId, AgentIPCMessage, ProjectId, RegisterParams};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::process::Command;
use swayipc_async::Connection;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;

#[derive(Parser, Clone, Debug)]
#[command(name = "swayve")]
#[command(about = "Sway Agent Orchestrator & CLI Tool", long_about = None)]
pub struct Cli {
    #[arg(long, global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    /// Output shell prompt hook configuration for bash/zsh
    InitShell,
    /// Start the swayve background daemon
    Daemon,
    /// Spawn an agent terminal inside Sway
    Spawn {
        #[arg(long)]
        wave: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        project: String,
        #[arg(long)]
        command: Option<String>,
        #[arg(long, short)]
        prompt: Option<String>,
        /// Full agent ID of the parent (its SWAYVE_AGENT_ID, e.g. A-c1-14404), not the bare name
        #[arg(long)]
        parent: Option<String>,
    },
    /// Toggle visibility of the Orchestrator panel
    Panel {
        #[command(subcommand)]
        action: PanelAction,
    },
    /// View the status of active agents
    Status {
        #[arg(long)]
        json: bool,
        #[arg(long, short)]
        watch: bool,
        #[arg(long, requires = "watch")]
        diff: bool,
    },
    /// Focus navigation commands
    Focus {
        #[arg(long)]
        cycle: bool,
    },
}

#[derive(Subcommand, Clone, Debug)]
pub enum PanelAction {
    Toggle,
}

/// Pure: `xdg` is the value of `XDG_RUNTIME_DIR`.
pub fn runtime_dir_from(xdg: Option<&str>) -> Result<std::path::PathBuf> {
    match xdg {
        Some(x) if !x.is_empty() => Ok(std::path::Path::new(x).join("swayve")),
        _ => anyhow::bail!("XDG_RUNTIME_DIR is not set; refusing to use a shared location"),
    }
}

pub fn swayve_runtime_dir() -> Result<std::path::PathBuf> {
    runtime_dir_from(env::var("XDG_RUNTIME_DIR").ok().as_deref())
}

/// Current effective uid (Linux: owner of /proc/self).
pub fn current_uid() -> Result<u32> {
    use std::os::unix::fs::MetadataExt;
    Ok(std::fs::metadata("/proc/self")
        .context("Failed to determine current uid via /proc/self")?
        .uid())
}

/// Create `p` with mode 0700, or verify an existing dir is ours with no group/other bits.
pub fn ensure_private_dir(p: &std::path::Path) -> Result<()> {
    ensure_private_dir_for(p, current_uid()?)
}

/// Like `ensure_private_dir`, but verifies ownership against `expected_uid`.
pub fn ensure_private_dir_for(p: &std::path::Path, expected_uid: u32) -> Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(p)
        .with_context(|| format!("Failed to create directory {}", p.display()))?;
    let md = std::fs::metadata(p)?;
    if md.uid() != expected_uid || md.mode() & 0o077 != 0 {
        anyhow::bail!(
            "{} must be owned by the current user with mode 0700",
            p.display()
        );
    }
    Ok(())
}

pub async fn run_cli() -> Result<()> {
    let args = Cli::parse();
    let config = load_config();
    // Resolved lazily: init-shell and panel never touch the socket.
    let socket_path = if matches!(args.command, Commands::InitShell | Commands::Panel { .. }) {
        String::new()
    } else if let Ok(p) = env::var("SWAYVE_SOCKET") {
        p
    } else {
        let dir = swayve_runtime_dir()?;
        ensure_private_dir(&dir)?;
        dir.join("swayve.sock").to_string_lossy().into_owned()
    };
    let socket_path = socket_path.as_str();

    // Set dry_run from CLI flag or environment
    let dry_run = args.dry_run || env::var("SWAYVE_DRY_RUN").is_ok();

    match args.command {
        Commands::InitShell => {
            let script = r#"# Swayve shell hook initialization for bash and zsh
_swayve_send_status() {
    [ -z "$SWAYVE_SOCKET" ] || [ -z "$SWAYVE_AGENT_ID" ] || [ ! -S "$SWAYVE_SOCKET" ] && return 0
    local status="$1"
    local msg="{\"method\":\"agent.update_status\",\"params\":{\"agent_id\":\"$SWAYVE_AGENT_ID\",\"status\":\"$status\"}}"
    if command -v nc >/dev/null 2>&1; then
        printf '%s\n' "$msg" | nc -U "$SWAYVE_SOCKET" 2>/dev/null || true
    elif command -v socat >/dev/null 2>&1; then
        printf '%s\n' "$msg" | socat - UNIX-CONNECT:"$SWAYVE_SOCKET" 2>/dev/null || true
    fi
}

_swayve_precmd() {
    _swayve_send_status "idle"
}

_swayve_preexec() {
    [ -n "$COMP_LINE" ] && return 0
    _swayve_send_status "thinking"
}

if [ -n "$ZSH_VERSION" ]; then
    autoload -Uz add-zsh-hook 2>/dev/null && {
        add-zsh-hook precmd _swayve_precmd
        add-zsh-hook preexec _swayve_preexec
    } || {
        precmd_functions+=(_swayve_precmd)
        preexec_functions+=(_swayve_preexec)
    }
elif [ -n "$BASH_VERSION" ]; then
    _swayve_in_precmd=0
    _swayve_bash_precmd() {
        _swayve_in_precmd=1
        _swayve_precmd
    }
    _swayve_bash_preexec() {
        if [ "$_swayve_in_precmd" -eq 1 ]; then
            _swayve_in_precmd=0
            return 0
        fi
        _swayve_preexec
    }
    if [[ ! "$PROMPT_COMMAND" =~ _swayve_bash_precmd ]]; then
        PROMPT_COMMAND="_swayve_bash_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
    fi
    trap '_swayve_bash_preexec' DEBUG
fi
"#;
            println!("{}", script);
        }

        Commands::Daemon => {
            println!("[daemon] Starting swayve daemon...");
            let (event_tx, event_rx) = mpsc::channel::<DaemonEvent>(128);
            let (sway_cmd_tx, sway_cmd_rx) = mpsc::channel::<SwayCommand>(64);

            let (sway_event_tx, mut sway_event_rx) = mpsc::channel::<SwayEvent>(64);
            let event_tx_forward = event_tx.clone();
            tokio::spawn(async move {
                while let Some(se) = sway_event_rx.recv().await {
                    let _ = event_tx_forward.send(DaemonEvent::Sway(se)).await;
                }
            });
            tokio::spawn(async move {
                if let Err(e) = run_event_listener(dry_run, sway_event_tx).await {
                    eprintln!("[daemon] Sway event listener error: {:?}", e);
                }
            });

            let manager = SwayManager::new(dry_run, sway_cmd_rx);
            tokio::spawn(async move {
                if let Err(e) = manager.run_executor().await {
                    eprintln!("[daemon] Sway command executor error: {:?}", e);
                }
            });

            let socket_path_str = socket_path.to_string();
            let ipc_lines_per_sec = config.daemon.ipc_lines_per_sec;
            let ipc_burst = config.daemon.ipc_burst;
            tokio::spawn(async move {
                let limits = crate::ratelimit::IpcLimits {
                    lines_per_sec: ipc_lines_per_sec,
                    burst: ipc_burst,
                };
                if let Err(e) = start_daemon_ipc_with_limits(&socket_path_str, event_tx, limits).await {
                    eprintln!("[daemon] Socket server error: {:?}", e);
                }
            });

            run_daemon_loops(dry_run, config, socket_path, event_rx, sway_cmd_tx).await?;
        }

        Commands::Spawn {
            wave,
            agent,
            project,
            command,
            prompt,
            parent,
        } => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let plan = spawn_plan(&agent, &wave, command, prompt.as_deref(), &config.terminal.args, now);
            let agent_id = plan.agent_id.clone();
            let app_id = plan.app_id.clone();

            println!("[spawn] Registering agent {} with daemon...", agent_id);

            // Register with daemon over Unix socket, and wait until it is listed (G52, D1).
            if UnixStream::connect(socket_path).await.is_ok() {
                let reg_msg = AgentIPCMessage::Register {
                    id: None,
                    params: RegisterParams {
                        agent_id: AgentId(agent_id.clone()),
                        agent_type: agent.clone(),
                        project_id: ProjectId(project.clone()),
                        pid: std::process::id(),
                        app_id: app_id.clone(),
                        parent_agent_id: parent.map(AgentId),
                    },
                };
                let payload = serde_json::to_string(&reg_msg)? + "\n";
                if !register_and_wait(socket_path, &payload, &agent_id, std::time::Duration::from_secs(2)).await {
                    println!("[spawn] Warning: daemon did not list {} within 2s; launching anyway", agent_id);
                }
            } else {
                println!("[spawn] Warning: Could not connect to daemon socket at {}. Running in standalone mode.", socket_path);
            }

            let term_args = plan.term_args;

            println!(
                "[spawn] Launching terminal: {} {:?}",
                config.terminal.default_emulator, term_args
            );

            if !dry_run {
                let runtime_dir = swayve_runtime_dir()?;
                ensure_private_dir(&runtime_dir)?;
                let logs_dir = runtime_dir.join("logs");
                ensure_private_dir(&logs_dir)?;
                let log_path = format!("{}/{}.log", logs_dir.display(), agent_id);
                let log_file = std::fs::File::create(&log_path)
                    .with_context(|| format!("Failed to create log file at {}", log_path))?;
                let err_file = log_file
                    .try_clone()
                    .with_context(|| format!("Failed to clone log file handle for {}", log_path))?;

                Command::new(&config.terminal.default_emulator)
                    .env("SWAYVE_SOCKET", socket_path)
                    .env("SWAYVE_AGENT_ID", &agent_id)
                    .env("SWAYVE_PROJECT_ID", &project)
                    .args(&term_args)
                    .stdout(log_file)
                    .stderr(err_file)
                    .spawn()
                    .context("Failed to launch agent terminal emulator")?;
            } else {
                println!("[spawn] [DRY-RUN] Would spawn terminal.");
            }
        }

        Commands::Panel { action } => match action {
            PanelAction::Toggle => {
                if dry_run {
                    println!("[panel] [DRY-RUN] Toggling panel visibility.");
                    return Ok(());
                }

                if env::var("SWAYSOCK").is_err() {
                    anyhow::bail!("Sway is not running.");
                }

                let mut conn = Connection::new().await?;
                let tree = conn.get_tree().await?;

                // Search for orchestrator window recursively
                fn find_orchestrator(node: &swayipc_async::Node) -> bool {
                    if let Some(app_id) = &node.app_id
                        && app_id == "agent.orchestrator" {
                            return true;
                        }
                    for child in &node.nodes {
                        if find_orchestrator(child) {
                            return true;
                        }
                    }
                    for child in &node.floating_nodes {
                        if find_orchestrator(child) {
                            return true;
                        }
                    }
                    false
                }

                let exists = find_orchestrator(&tree);
                if exists {
                    println!("[panel] Orchestrator panel exists. Toggling scratchpad.");
                    conn.run_command("[app_id=\"agent.orchestrator\"] scratchpad show").await?;
                } else {
                    println!("[panel] Orchestrator panel not found. Spawning...");
                    // Template args for orchestrator panel
                    let mut term_args = Vec::new();
                    let status_cmd = "swayve status --watch".to_string();
                    for arg in &config.terminal.args {
                        let replaced = arg
                            .replace("{app_id}", "agent.orchestrator")
                            .replace("{title}", "Swayve Orchestrator HUD")
                            .replace("{command}", &status_cmd);
                        term_args.push(replaced);
                    }
                    Command::new(&config.terminal.default_emulator)
                        .args(&term_args)
                        .spawn()
                        .context("Failed to spawn orchestrator terminal")?;
                }
            }
        },

        Commands::Status { json, watch, diff } => {
            if watch {
                if dry_run {
                    println!("[status] [DRY-RUN] HUD watch mode active (dry run)");
                    return Ok(());
                }
                run_status_watch(socket_path, diff).await?;
            } else if json {
                let summary = serde_json::json!({
                    "text": "Swayve: Active",
                    "tooltip": "Swayve is monitoring active agents.",
                    "class": "active",
                    "percentage": 100
                });
                println!("{}", serde_json::to_string(&summary)?);
            } else {
                if dry_run {
                    println!("[status] [DRY-RUN] Active agents: None (Dry-run mode)");
                    return Ok(());
                }

                if env::var("SWAYSOCK").is_err() {
                    println!("Sway is not running.");
                    return Ok(());
                }

                let mut conn = Connection::new().await?;
                let tree = conn.get_tree().await?;

                fn collect_agents(node: &swayipc_async::Node, agents: &mut Vec<(String, String)>) {
                    if let Some(app_id) = &node.app_id
                        && app_id.starts_with("agent.") {
                            let title = node.name.clone().unwrap_or_else(|| "Unknown".to_string());
                            agents.push((app_id.clone(), title));
                        }
                    for child in &node.nodes {
                        collect_agents(child, agents);
                    }
                    for child in &node.floating_nodes {
                        collect_agents(child, agents);
                    }
                }

                let mut agents = Vec::new();
                collect_agents(&tree, &mut agents);

                if agents.is_empty() {
                    println!("No active Swayve agent windows detected.");
                } else {
                    println!("Active Swayve Agent Windows:");
                    for (app_id, title) in agents {
                        println!(" - App ID: {} | Window: {}", app_id, title);
                    }
                }
            }
        }

        Commands::Focus { cycle } => {
            if cycle {
                if dry_run {
                    println!("[focus] [DRY-RUN] Would cycle focus.");
                    return Ok(());
                }

                // The daemon owns the agent hierarchy, so ask it to pick the next
                // window. Without it, fall back to a flat cycle over the Sway tree.
                match UnixStream::connect(socket_path).await {
                    Ok(mut stream) => {
                        let payload = serde_json::to_string(&AgentIPCMessage::Focus)? + "\n";
                        stream.write_all(payload.as_bytes()).await?;
                        println!("[focus] Requested focus cycle from daemon.");
                    }
                    Err(_) => {
                        println!("[focus] Daemon unreachable. Cycling focus directly.");
                        focus_cycle_standalone().await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Pick the next agent in tree order after the focused one (wrapping); first if none focused.
fn next_flat_agent_index(agents: &[(i64, bool)]) -> Option<usize> {
    if agents.is_empty() {
        return None;
    }
    Some(agents.iter().position(|&(_, f)| f).map_or(0, |i| (i + 1) % agents.len()))
}

/// Cycle focus across agent windows by talking to Sway directly, ignoring hierarchy.
/// Used only when the daemon is not reachable.
async fn focus_cycle_standalone() -> Result<()> {
    if env::var("SWAYSOCK").is_err() {
        anyhow::bail!("Sway is not running.");
    }

    let mut conn = Connection::new().await?;
    let tree = conn.get_tree().await?;

    fn collect_agents_focus(node: &swayipc_async::Node, agents: &mut Vec<(i64, bool)>) {
        if let Some(app_id) = &node.app_id
            && app_id.starts_with("agent.") {
                agents.push((node.id, node.focused));
            }
        for child in &node.nodes {
            collect_agents_focus(child, agents);
        }
        for child in &node.floating_nodes {
            collect_agents_focus(child, agents);
        }
    }

    let mut agents = Vec::new();
    collect_agents_focus(&tree, &mut agents);

    let Some(next_idx) = next_flat_agent_index(&agents) else {
        println!("No active agent windows found to focus.");
        return Ok(());
    };
    let (target_id, _) = agents[next_idx];
    conn.run_command(&format!("[con_id=\"{}\"] focus", target_id)).await?;
    println!("[focus] Focused container {}", target_id);

    Ok(())
}

/// Folds one `subscribe_diff` line into `map`. Returns `false`, changing nothing,
/// when the line is not JSON.
fn fold_diff_line(map: &mut std::collections::HashMap<String, crate::types::AgentWindow>, line: &str) -> bool {
    let Ok(frame) = serde_json::from_str::<crate::types::DiffFrame>(line) else {
        return false;
    };
    frame.fold_into(map);
    true
}

/// What `swayve spawn` will register and launch. Pure: the clock is passed in.
#[derive(Debug, PartialEq)]
pub struct SpawnPlan {
    pub agent_id: String,
    pub app_id: String,
    pub title: String,
    pub term_args: Vec<String>,
}

/// Builds the spawn plan. `term_template` is `config.terminal.args`. `{command}` is replaced by
/// the shell-split command; `{app_id}` and `{title}` are replaced by the derived values.
pub fn spawn_plan(
    agent: &str,
    wave: &str,
    command: Option<String>,
    prompt: Option<&str>,
    term_template: &[String],
    now_ms: u128,
) -> SpawnPlan {
    let base_command = command.unwrap_or_else(|| agent.to_string());
    let actual_command = if let Some(p) = prompt {
        let escaped_prompt = p.replace('\'', "'\\''");
        let cmd_templated = if base_command.contains("{prompt}") {
            base_command.replace("{prompt}", &escaped_prompt)
        } else {
            format!("{} \"{}\"", base_command, escaped_prompt)
        };
        format!("bash -c '{}'", cmd_templated)
    } else {
        base_command
    };
    let suffix = now_ms % 100000;
    let agent_id = format!("{}-{}-{}", agent, wave, suffix);
    let app_id = format!("agent.{}.{}.{}", agent, wave, suffix);
    let title = format!("Agent [{}]: {}", agent, wave);

    let mut term_args = Vec::new();
    for arg in term_template {
        if arg == "{command}" {
            if let Some(split_cmds) = shlex::split(&actual_command) {
                term_args.extend(split_cmds);
            } else {
                term_args.push(actual_command.clone());
            }
        } else {
            term_args.push(arg.replace("{app_id}", &app_id).replace("{title}", &title));
        }
    }
    SpawnPlan { agent_id, app_id, title, term_args }
}

/// Writes `reg_line` on one connection, then polls `daemon.subscribe` on fresh connections until
/// the first line lists `agent_id`. Returns `false` when the socket cannot be reached, or when
/// `timeout` passes first. Used by `spawn` so the window is not launched before the daemon has
/// applied the registration (G52, re-scoped; no wire change).
pub async fn register_and_wait(
    socket_path: &str,
    reg_line: &str,
    agent_id: &str,
    timeout: std::time::Duration,
) -> bool {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let Ok(mut stream) = UnixStream::connect(socket_path).await else {
        return false;
    };
    if stream.write_all(reg_line.as_bytes()).await.is_err() || stream.flush().await.is_err() {
        return false;
    }
    drop(stream);

    let needle = format!("\"id\":\"{}\"", agent_id);
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Ok(mut s) = UnixStream::connect(socket_path).await
            && s.write_all(b"{\"method\":\"daemon.subscribe\"}\n").await.is_ok()
        {
            let (r, _) = s.into_split();
            let first = tokio::time::timeout(
                std::time::Duration::from_millis(100),
                BufReader::new(r).lines().next_line(),
            )
            .await;
            if let Ok(Ok(Some(line))) = first
                && line.contains(&needle)
            {
                return true;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

async fn run_status_watch(socket_path: &str, diff: bool) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(e) => {
            println!("[status] Could not connect to daemon socket at {}: {:?}", socket_path, e);
            return Ok(());
        }
    };

    let sub_msg = if diff {
        AgentIPCMessage::SubscribeDiff
    } else {
        AgentIPCMessage::SubscribeStatus
    };
    let payload = serde_json::to_string(&sub_msg)? + "\n";
    stream.write_all(payload.as_bytes()).await?;

    let (reader, _) = stream.into_split();
    let buf_reader = BufReader::new(reader);
    let mut lines = buf_reader.lines();
    let mut map: std::collections::HashMap<String, crate::types::AgentWindow> = std::collections::HashMap::new();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let agents = if diff {
            if !fold_diff_line(&mut map, &line) {
                continue;
            }
            let mut agents: Vec<crate::types::AgentWindow> = map.values().cloned().collect();
            agents.sort_by(|a, b| a.id.0.cmp(&b.id.0));
            Some(agents)
        } else {
            serde_json::from_str::<Vec<crate::types::AgentWindow>>(&line).ok()
        };

        if let Some(agents) = agents {
            print!("\x1B[2J\x1B[H");
            use std::io::Write;
            let _ = std::io::stdout().flush();
            render_agent_tree(&agents);
        }
    }

    Ok(())
}

fn render_agent_tree(agents: &[crate::types::AgentWindow]) {
    println!("==================================================");
    println!("        SWAYVE AGENT HIERARCHY DASHBOARD          ");
    println!("==================================================");

    if agents.is_empty() {
        println!("No active agents registered.");
        return;
    }

    let agent_ids: std::collections::HashSet<&crate::types::AgentId> =
        agents.iter().map(|a| &a.id).collect();

    let mut parent_map: std::collections::HashMap<
        Option<crate::types::AgentId>,
        Vec<&crate::types::AgentWindow>,
    > = std::collections::HashMap::new();

    for agent in agents {
        let parent = match &agent.parent_id {
            Some(pid) if agent_ids.contains(pid) => Some(pid.clone()),
            _ => None,
        };
        parent_map.entry(parent).or_default().push(agent);
    }

    fn print_node(
        node: &crate::types::AgentWindow,
        prefix: &str,
        is_last: bool,
        parent_map: &std::collections::HashMap<
            Option<crate::types::AgentId>,
            Vec<&crate::types::AgentWindow>,
        >,
    ) {
        let connector = if is_last { "└── " } else { "├── " };
        let status_str = format_status(&node.status);
        println!(
            "{}{}{} [Status: {}] [Project: {}] [App ID: {}]",
            prefix, connector, node.id.0, status_str, node.project.0, node.app_id
        );

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        if let Some(children) = parent_map.get(&Some(node.id.clone())) {
            for (i, child) in children.iter().enumerate() {
                let child_is_last = i == children.len() - 1;
                print_node(child, &new_prefix, child_is_last, parent_map);
            }
        }
    }

    if let Some(roots) = parent_map.get(&None) {
        for (i, root) in roots.iter().enumerate() {
            let is_last = i == roots.len() - 1;
            print_node(root, "", is_last, &parent_map);
        }
    }
}

fn format_status(status: &crate::types::AgentStatus) -> String {
    match status {
        crate::types::AgentStatus::Idle => "\x1B[90mIdle\x1B[0m".to_string(),
        crate::types::AgentStatus::Thinking => "\x1B[38;5;208mThinking\x1B[0m".to_string(),
        crate::types::AgentStatus::ExecutingTool => "\x1B[36mExecutingTool\x1B[0m".to_string(),
        crate::types::AgentStatus::AwaitingHumanInput => {
            "\x1B[5;1;31mAwaitingInput\x1B[0m".to_string()
        }
        crate::types::AgentStatus::Completed => "\x1B[32mCompleted\x1B[0m".to_string(),
        crate::types::AgentStatus::Failed(err) => format!("\x1B[31mFailed({})\x1B[0m", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[tokio::test]
    async fn test_register_and_wait_false_when_no_socket() {
        let path = std::env::temp_dir().join(format!("swayve-{}-no-such.sock", std::process::id()));
        let line = "{\"method\":\"daemon.subscribe\"}\n";
        let start = std::time::Instant::now();
        let ok = register_and_wait(path.to_str().unwrap(), line, "x", std::time::Duration::from_millis(200)).await;
        assert!(!ok);
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
    }

    #[test]
    fn test_spawn_plan_escapes_single_quote_in_prompt() {
        let tmpl = vec!["{command}".to_string()];
        let plan = spawn_plan("pi", "w1", None, Some("it's"), &tmpl, 1);
        assert_eq!(plan.term_args, vec!["bash", "-c", "pi \"it's\""]);
    }

    #[test]
    fn test_spawn_plan_ids_follow_clock_suffix() {
        let plan = spawn_plan("pi", "w1", None, None, &[], 12345);
        assert_eq!(plan.agent_id, "pi-w1-12345");
        assert_eq!(plan.app_id, "agent.pi.w1.12345");
        assert_eq!(plan.title, "Agent [pi]: w1");
    }

    #[test]
    fn test_spawn_plan_distinct_clock_gives_distinct_ids() {
        let a = spawn_plan("pi", "w1", None, None, &[], 1);
        let b = spawn_plan("pi", "w1", None, None, &[], 2);
        assert_ne!(a.agent_id, b.agent_id);
        assert_ne!(a.app_id, b.app_id);
    }

    #[test]
    fn test_spawn_plan_substitutes_app_id_and_title() {
        let tmpl: Vec<String> = ["--app-id", "{app_id}", "--title", "{title}"].iter().map(|s| s.to_string()).collect();
        let plan = spawn_plan("pi", "w1", None, None, &tmpl, 7);
        assert_eq!(plan.term_args, vec!["--app-id", "agent.pi.w1.7", "--title", "Agent [pi]: w1"]);
    }

    fn diff_agent_json(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id, "project": "proj", "parent_id": null, "container_id": 1,
            "app_id": "app", "mark": "mark_x", "status": "thinking"
        })
    }

    fn diff_agent(id: &str) -> crate::types::AgentWindow {
        serde_json::from_value(diff_agent_json(id)).unwrap()
    }

    #[test]
    fn test_fold_diff_snapshot_clears_then_upserts() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), diff_agent("a"));
        let line = serde_json::json!({"snapshot": true, "upsert": [diff_agent_json("b")], "remove": []}).to_string();
        assert!(fold_diff_line(&mut map, &line));
        assert!(!map.contains_key("a"));
        assert!(map.contains_key("b"));
    }

    #[test]
    fn test_fold_diff_upsert_then_remove() {
        let mut map = std::collections::HashMap::new();
        let up = serde_json::json!({"snapshot": false, "upsert": [diff_agent_json("c")], "remove": []}).to_string();
        assert!(fold_diff_line(&mut map, &up));
        assert!(map.contains_key("c"));
        let rm = serde_json::json!({"snapshot": false, "upsert": [], "remove": ["c"]}).to_string();
        assert!(fold_diff_line(&mut map, &rm));
        assert!(map.is_empty());
    }

    #[test]
    fn test_fold_diff_skips_non_json() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), diff_agent("a"));
        let before = map.clone();
        assert!(!fold_diff_line(&mut map, "not json"));
        assert_eq!(map, before);
    }

    #[test]
    fn test_flat_fallback_cycles_agent_windows_in_tree_order() {
        assert_eq!(next_flat_agent_index(&[]), None);
        assert_eq!(next_flat_agent_index(&[(1, false), (2, true), (3, false)]), Some(2));
        assert_eq!(next_flat_agent_index(&[(1, false), (2, false), (3, true)]), Some(0));
        assert_eq!(next_flat_agent_index(&[(1, false), (2, false)]), Some(0));
        assert_eq!(next_flat_agent_index(&[(1, true)]), Some(0));
    }

    #[test]
    fn test_runtime_dir_requires_xdg_runtime_dir() {
        let msg = "XDG_RUNTIME_DIR is not set; refusing to use a shared location";
        assert_eq!(runtime_dir_from(None).unwrap_err().to_string(), msg);
        assert_eq!(runtime_dir_from(Some("")).unwrap_err().to_string(), msg);
        assert_eq!(
            runtime_dir_from(Some("/run/user/1000")).unwrap(),
            std::path::PathBuf::from("/run/user/1000/swayve")
        );
    }

    #[test]
    fn test_ensure_private_dir_creates_0700() {
        use std::os::unix::fs::MetadataExt;
        let p = std::env::temp_dir().join(format!("swayve_epd_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        let res = ensure_private_dir(&p);
        let mode = std::fs::metadata(&p).map(|m| m.mode() & 0o777);
        let _ = std::fs::remove_dir_all(&p);
        assert!(res.is_ok());
        assert_eq!(mode.unwrap(), 0o700);
    }

    #[test]
    fn test_ensure_private_dir_rejects_loose_mode() {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let p = std::env::temp_dir().join(format!("swayve_epd_loose_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::DirBuilder::new().mode(0o755).create(&p).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        let res = ensure_private_dir(&p);
        let _ = std::fs::remove_dir_all(&p);
        assert!(res.is_err());
    }

    fn epdf_dir(tag: &str) -> std::path::PathBuf {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let p = std::env::temp_dir().join(format!("swayve_epdf_{}_{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o700)).unwrap();
        p
    }

    #[test]
    fn test_ensure_private_dir_for_rejects_foreign_uid() {
        let p = epdf_dir("foreign");
        let res = ensure_private_dir_for(&p, current_uid().unwrap().wrapping_add(1));
        let _ = std::fs::remove_dir_all(&p);
        assert!(res.is_err());
    }

    #[test]
    fn test_ensure_private_dir_for_accepts_matching_uid() {
        let p = epdf_dir("ok");
        let res = ensure_private_dir_for(&p, current_uid().unwrap());
        let _ = std::fs::remove_dir_all(&p);
        assert!(res.is_ok());
    }

    #[test]
    fn test_init_shell_cmd_parsing() {
        let cli = Cli::try_parse_from(["swayve", "init-shell"]).unwrap();
        assert!(matches!(cli.command, Commands::InitShell));
    }

    #[test]
    fn test_status_watch_cmd_parsing() {
        let cli = Cli::try_parse_from(["swayve", "status", "--watch"]).unwrap();
        if let Commands::Status { watch, json, diff } = cli.command {
            assert!(watch);
            assert!(!json);
            assert!(!diff);
        } else {
            panic!("Expected Commands::Status");
        }
        let cli = Cli::try_parse_from(["swayve", "status", "--watch", "--diff"]).unwrap();
        if let Commands::Status { watch, diff, .. } = cli.command {
            assert!(watch && diff);
        } else {
            panic!("Expected Commands::Status");
        }
        assert!(Cli::try_parse_from(["swayve", "status", "--diff"]).is_err());
    }

    #[test]
    fn test_format_status_ansi_codes() {
        use crate::types::AgentStatus;
        assert_eq!(format_status(&AgentStatus::Idle), "\x1B[90mIdle\x1B[0m");
        assert_eq!(format_status(&AgentStatus::Thinking), "\x1B[38;5;208mThinking\x1B[0m");
        assert_eq!(format_status(&AgentStatus::ExecutingTool), "\x1B[36mExecutingTool\x1B[0m");
        assert_eq!(format_status(&AgentStatus::AwaitingHumanInput), "\x1B[5;1;31mAwaitingInput\x1B[0m");
        assert_eq!(format_status(&AgentStatus::Completed), "\x1B[32mCompleted\x1B[0m");
        assert_eq!(format_status(&AgentStatus::Failed("error".to_string())), "\x1B[31mFailed(error)\x1B[0m");
    }
}


