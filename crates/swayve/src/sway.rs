use anyhow::{Context, Result};
use futures_lite::stream::StreamExt;
use std::env;
use swayipc_async::{Connection, Event, EventType, WindowChange};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum SwayCommand {
    MarkWindow { con_id: i64, mark: String },
    SetBorderColor { mark: String, hex_color: String },
    FocusWindow { mark: String },
    MoveToWorkspace { con_id: i64, workspace: String },
    ApplyLayout { workspace: String, layout: String },
    SplitHorizontal { parent_mark: String },
    MoveToContainer { con_id: i64, target_mark: String },
    SetLayoutTabbed { mark: String },
}

impl SwayCommand {
    pub fn to_command_string(&self) -> String {
        match self {
            SwayCommand::MarkWindow { con_id, mark } => {
                format!("[con_id=\"{}\"] mark --add \"{}\"", con_id, mark)
            }
            SwayCommand::SetBorderColor { mark, .. } => {
                format!("[con_mark=\"{}\"] border pixel 3", mark)
            }
            SwayCommand::FocusWindow { mark } => {
                format!("[con_mark=\"{}\"] focus", mark)
            }
            SwayCommand::MoveToWorkspace { con_id, workspace } => {
                format!("[con_id=\"{}\"] move container to workspace {}", con_id, workspace)
            }
            SwayCommand::ApplyLayout { workspace, layout } => {
                format!("[workspace=\"{}\"] layout {}", workspace, layout)
            }
            SwayCommand::SplitHorizontal { parent_mark } => {
                format!("[con_mark=\"{}\"] focus, split horizontal", parent_mark)
            }
            SwayCommand::MoveToContainer { con_id, target_mark } => {
                format!("[con_id=\"{}\"] move container to mark \"{}\"", con_id, target_mark)
            }
            SwayCommand::SetLayoutTabbed { mark } => {
                format!("[con_mark=\"{}\"] layout tabbed", mark)
            }
        }
    }
}

pub struct SwayManager {
    dry_run: bool,
    command_rx: mpsc::Receiver<SwayCommand>,
}

impl SwayManager {
    pub fn new(dry_run: bool, command_rx: mpsc::Receiver<SwayCommand>) -> Self {
        Self { dry_run, command_rx }
    }

    pub async fn run_executor(mut self) -> Result<()> {
        let mut connection = if self.dry_run {
            println!("[sway] Operating in DRY-RUN mode. All commands will be logged.");
            None
        } else {
            // Verify SWAYSOCK is present or fail cleanly
            if env::var("SWAYSOCK").is_err() {
                anyhow::bail!("SWAYSOCK environment variable not set, and dry-run is disabled.");
            }
            let conn = Connection::new().await.context("Failed to connect to Sway IPC socket")?;
            Some(conn)
        };

        while let Some(cmd) = self.command_rx.recv().await {
            let command_str = cmd.to_command_string();

            if let Some(conn) = &mut connection {
                println!("[sway] Executing: {}", command_str);
                if let Err(e) = conn.run_command(&command_str).await {
                    eprintln!("[sway] Command execution failed for '{}': {:?}", command_str, e);
                }
            } else {
                println!("[sway] [DRY-RUN] Would execute: {}", command_str);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SwayEvent {
    WindowCreated { con_id: i64, app_id: String, pid: Option<i32> },
    WindowDestroyed { con_id: i64 },
    WindowFocused { con_id: i64 },
}

pub async fn run_event_listener(
    dry_run: bool,
    event_tx: mpsc::Sender<SwayEvent>,
) -> Result<()> {
    if dry_run {
        println!("[sway] Event listener disabled in DRY-RUN mode.");
        return Ok(());
    }

    if env::var("SWAYSOCK").is_err() {
        anyhow::bail!("SWAYSOCK environment variable not set. Event listener requires Sway running.");
    }

    let subs = [EventType::Window];
    let mut event_stream = Connection::new()
        .await
        .context("Failed to connect to Sway IPC for events")?
        .subscribe(subs)
        .await
        .context("Failed to subscribe to window events")?;

    println!("[sway] Subscribed to window events.");

    while let Some(event) = event_stream.next().await {
        if let Ok(Event::Window(win_event)) = event {
            match win_event.change {
                WindowChange::New => {
                    let con_id = win_event.container.id;
                    let app_id = win_event.container.app_id.clone().unwrap_or_default();
                    let pid = win_event.container.pid;
                    let _ = event_tx
                        .send(SwayEvent::WindowCreated { con_id, app_id, pid })
                        .await;
                }
                WindowChange::Close => {
                    let con_id = win_event.container.id;
                    let _ = event_tx.send(SwayEvent::WindowDestroyed { con_id }).await;
                }
                WindowChange::Focus => {
                    let con_id = win_event.container.id;
                    let _ = event_tx.send(SwayEvent::WindowFocused { con_id }).await;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_every_variant() {
        let s = |c: SwayCommand| c.to_command_string();
        assert_eq!(s(SwayCommand::MarkWindow { con_id: 7, mark: "m".into() }), "[con_id=\"7\"] mark --add \"m\"");
        assert_eq!(s(SwayCommand::SetBorderColor { mark: "m".into(), hex_color: "#fff".into() }), "[con_mark=\"m\"] border pixel 3");
        assert_eq!(s(SwayCommand::FocusWindow { mark: "m".into() }), "[con_mark=\"m\"] focus");
        assert_eq!(s(SwayCommand::MoveToWorkspace { con_id: 7, workspace: "w".into() }), "[con_id=\"7\"] move container to workspace w");
        assert_eq!(s(SwayCommand::ApplyLayout { workspace: "w".into(), layout: "l".into() }), "[workspace=\"w\"] layout l");
        assert_eq!(s(SwayCommand::SplitHorizontal { parent_mark: "p".into() }), "[con_mark=\"p\"] focus, split horizontal");
        assert_eq!(s(SwayCommand::MoveToContainer { con_id: 7, target_mark: "t".into() }), "[con_id=\"7\"] move container to mark \"t\"");
        assert_eq!(s(SwayCommand::SetLayoutTabbed { mark: "m".into() }), "[con_mark=\"m\"] layout tabbed");
    }
}
