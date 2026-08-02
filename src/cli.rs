use crate::config::load_config;
use crate::daemon::{run_daemon_loops, start_daemon_ipc, DaemonEvent};
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

pub async fn run_cli() -> Result<()> {
    let args = Cli::parse();
    let config = load_config();
    let socket_path = "/tmp/swayve.sock";

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
            tokio::spawn(async move {
                if let Err(e) = start_daemon_ipc(&socket_path_str, event_tx).await {
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
            let base_command = command.unwrap_or_else(|| agent.clone());
            let actual_command = if let Some(ref p) = prompt {
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
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let suffix = now % 100000;
            let agent_id = format!("{}-{}-{}", agent, wave, suffix);
            let app_id = format!("agent.{}.{}.{}", agent, wave, suffix);
            let title = format!("Agent [{}]: {}", agent, wave);

            println!("[spawn] Registering agent {} with daemon...", agent_id);

            // Register with daemon over Unix socket
            if let Ok(mut stream) = UnixStream::connect(socket_path).await {
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
                let _ = stream.write_all(payload.as_bytes()).await;
            } else {
                println!("[spawn] Warning: Could not connect to daemon socket at {}. Running in standalone mode.", socket_path);
            }

            // Template terminal command
            let mut term_args = Vec::new();
            for arg in &config.terminal.args {
                if arg == "{command}" {
                    if let Some(split_cmds) = shlex::split(&actual_command) {
                        for sub_arg in split_cmds {
                            term_args.push(sub_arg);
                        }
                    } else {
                        term_args.push(actual_command.clone());
                    }
                } else {
                    let replaced = arg
                        .replace("{app_id}", &app_id)
                        .replace("{title}", &title);
                    term_args.push(replaced);
                }
            }

            println!(
                "[spawn] Launching terminal: {} {:?}",
                config.terminal.default_emulator, term_args
            );

            if !dry_run {
                std::fs::create_dir_all("/tmp/swayve/logs")
                    .context("Failed to create log directory /tmp/swayve/logs")?;
                let log_path = format!("/tmp/swayve/logs/{}.log", agent_id);
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
                    if let Some(app_id) = &node.app_id {
                        if app_id == "agent.orchestrator" {
                            return true;
                        }
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
                    let status_cmd = format!("swayve status");
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

        Commands::Status { json } => {
            if json {
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
                    if let Some(app_id) = &node.app_id {
                        if app_id.starts_with("agent.") {
                            let title = node.name.clone().unwrap_or_else(|| "Unknown".to_string());
                            agents.push((app_id.clone(), title));
                        }
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

/// Cycle focus across agent windows by talking to Sway directly, ignoring hierarchy.
/// Used only when the daemon is not reachable.
async fn focus_cycle_standalone() -> Result<()> {
    if env::var("SWAYSOCK").is_err() {
        anyhow::bail!("Sway is not running.");
    }

    let mut conn = Connection::new().await?;
    let tree = conn.get_tree().await?;

    fn collect_agents_focus(node: &swayipc_async::Node, agents: &mut Vec<(i64, bool)>) {
        if let Some(app_id) = &node.app_id {
            if app_id.starts_with("agent.") {
                agents.push((node.id, node.focused));
            }
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

    if agents.is_empty() {
        println!("No active agent windows found to focus.");
        return Ok(());
    }

    let focused_idx = agents.iter().position(|&(_, focused)| focused);
    let next_idx = match focused_idx {
        Some(idx) => (idx + 1) % agents.len(),
        None => 0,
    };
    let (target_id, _) = agents[next_idx];
    conn.run_command(&format!("[con_id=\"{}\"] focus", target_id)).await?;
    println!("[focus] Focused container {}", target_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_init_shell_cmd_parsing() {
        let cli = Cli::try_parse_from(["swayve", "init-shell"]).unwrap();
        assert!(matches!(cli.command, Commands::InitShell));
    }
}

