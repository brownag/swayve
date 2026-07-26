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
            let agent_id = format!("{}-{}-{}", agent, wave, now % 100000);
            let app_id = format!("agent.{}.{}", agent, wave);
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
                        capabilities: None,
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
                let replaced = arg
                    .replace("{app_id}", &app_id)
                    .replace("{title}", &title)
                    .replace("{command}", &actual_command);
                term_args.push(replaced);
            }

            println!(
                "[spawn] Launching terminal: {} {:?}",
                config.terminal.default_emulator, term_args
            );

            if !dry_run {
                Command::new(&config.terminal.default_emulator)
                    .args(&term_args)
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
    }

    Ok(())
}
