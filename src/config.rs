use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub border_colors: BorderColorsConfig,
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderColorsConfig {
    pub idle: String,
    pub thinking: String,
    pub executing_tool: String,
    pub awaiting_human_input: String,
    pub completed: String,
    pub failed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub default_emulator: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default = "default_max_ancestry_depth")]
    pub max_ancestry_depth: usize,
    #[serde(default = "default_process_tree_audit_enabled")]
    pub process_tree_audit_enabled: bool,
}

fn default_max_ancestry_depth() -> usize {
    32
}

fn default_process_tree_audit_enabled() -> bool {
    true
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            max_ancestry_depth: 32,
            process_tree_audit_enabled: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            border_colors: BorderColorsConfig {
                idle: "#44475a".to_string(),
                thinking: "#ffb86c".to_string(),
                executing_tool: "#8be9fd".to_string(),
                awaiting_human_input: "#ff5555".to_string(),
                completed: "#50fa7b".to_string(),
                failed: "#ff79c6".to_string(),
            },
            terminal: TerminalConfig {
                default_emulator: "foot".to_string(),
                args: vec![
                    "--app-id".to_string(),
                    "{app_id}".to_string(),
                    "--title".to_string(),
                    "{title}".to_string(),
                    "-e".to_string(),
                    "{command}".to_string(),
                ],
            },
            daemon: DaemonConfig::default(),
        }
    }
}

pub fn load_config() -> Config {
    if let Some(home) = std::env::var("HOME").ok() {
        let path = PathBuf::from(format!("{}/.config/swayve/config.toml", home));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(parsed) = toml::from_str::<Config>(&content) {
                    return parsed;
                }
            }
        }
    }
    Config::default()
}
