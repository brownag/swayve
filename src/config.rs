use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub border_colors: BorderColorsConfig,
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct HooksConfig {
    pub on_agent_spawn: Option<String>,
    pub on_agent_status_change: Option<String>,
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
    #[serde(default = "default_auto_monitor")]
    pub auto_monitor: bool,
}

fn default_max_ancestry_depth() -> usize {
    32
}

fn default_process_tree_audit_enabled() -> bool {
    true
}

fn default_auto_monitor() -> bool {
    false
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            max_ancestry_depth: 32,
            process_tree_audit_enabled: true,
            auto_monitor: false,
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
            hooks: HooksConfig::default(),
        }
    }
}

pub fn parse_config_file(path: &std::path::Path) -> Result<Option<Config>, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {}", path.display(), e)),
    };
    toml::from_str::<Config>(&content)
        .map(Some)
        .map_err(|e| format!("{}: {}", path.display(), e))
}

/// Pure: `xdg` is `XDG_CONFIG_HOME`, `home` is `HOME`. A relative or empty
/// `xdg` is ignored per the XDG spec.
pub fn config_path_from(xdg: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    match (xdg, home) {
        (Some(x), _) if x.starts_with('/') => Some(PathBuf::from(x).join("swayve/config.toml")),
        (_, Some(h)) => Some(PathBuf::from(h).join(".config/swayve/config.toml")),
        _ => None,
    }
}

/// Pure: warning text when `XDG_CONFIG_HOME` is set but relative or empty.
pub fn xdg_config_warning(xdg: Option<&str>) -> Option<&'static str> {
    match xdg {
        Some(x) if x.is_empty() || !x.starts_with('/') => {
            Some("XDG_CONFIG_HOME is relative or empty; ignoring and using $HOME/.config")
        }
        _ => None,
    }
}

pub fn load_config() -> Config {
    let xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let home = std::env::var("HOME").ok();
    if let Some(msg) = xdg_config_warning(xdg.as_deref()) {
        eprintln!("[config] {}", msg);
    }
    let Some(path) = config_path_from(xdg.as_deref(), home.as_deref()) else {
        return Config::default();
    };
    match parse_config_file(&path) {
        Ok(Some(c)) => c,
        Ok(None) => Config::default(),
        Err(msg) => {
            eprintln!("[config] {}; using defaults", msg);
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_config_default() {
        let config = Config::default();
        assert_eq!(config.hooks.on_agent_spawn, None);
        assert_eq!(config.hooks.on_agent_status_change, None);
    }

    #[test]
    fn test_parse_config_file_malformed_reports_path() {
        let path = std::env::temp_dir()
            .join(format!("swayve-config-test-{}.toml", std::process::id()));
        fs::write(&path, "daemon = [").unwrap();
        let result = parse_config_file(&path);
        fs::remove_file(&path).unwrap();
        let msg = result.unwrap_err();
        assert!(msg.contains(&path.display().to_string()), "{msg}");
    }

    #[test]
    fn test_hooks_config_parsing() {
        let toml_str = r##"
            [border_colors]
            idle = "#111111"
            thinking = "#222222"
            executing_tool = "#333333"
            awaiting_human_input = "#444444"
            completed = "#555555"
            failed = "#666666"

            [terminal]
            default_emulator = "foot"
            args = ["-e", "{command}"]

            [hooks]
            on_agent_spawn = "echo spawn"
            on_agent_status_change = "echo status"
        "##;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.hooks.on_agent_spawn, Some("echo spawn".to_string()));
        assert_eq!(
            config.hooks.on_agent_status_change,
            Some("echo status".to_string())
        );
    }

    #[test]
    fn test_daemon_auto_monitor_config_parsing() {
        let config_default = Config::default();
        assert!(!config_default.daemon.auto_monitor);

        let toml_str = r##"
            [border_colors]
            idle = "#111111"
            thinking = "#222222"
            executing_tool = "#333333"
            awaiting_human_input = "#444444"
            completed = "#555555"
            failed = "#666666"

            [terminal]
            default_emulator = "foot"
            args = ["-e", "{command}"]

            [daemon]
            auto_monitor = true
        "##;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.daemon.auto_monitor);
    }

    #[test]
    fn test_config_path_absolute_xdg_wins() {
        assert_eq!(
            config_path_from(Some("/x/cfg"), Some("/home/u")),
            Some(PathBuf::from("/x/cfg/swayve/config.toml"))
        );
    }

    #[test]
    fn test_config_path_relative_xdg_falls_back_to_home() {
        assert_eq!(
            config_path_from(Some("rel/cfg"), Some("/home/u")),
            Some(PathBuf::from("/home/u/.config/swayve/config.toml"))
        );
    }

    #[test]
    fn test_config_path_empty_xdg_falls_back_to_home() {
        assert_eq!(
            config_path_from(Some(""), Some("/home/u")),
            Some(PathBuf::from("/home/u/.config/swayve/config.toml"))
        );
    }

    #[test]
    fn test_config_path_home_only() {
        assert_eq!(
            config_path_from(None, Some("/home/u")),
            Some(PathBuf::from("/home/u/.config/swayve/config.toml"))
        );
        assert_eq!(config_path_from(None, None), None);
    }

    #[test]
    fn test_xdg_warning_absolute_is_none() {
        assert_eq!(xdg_config_warning(Some("/x")), None);
    }

    #[test]
    fn test_xdg_warning_relative_is_some() {
        assert!(xdg_config_warning(Some("rel/dir")).is_some());
    }

    #[test]
    fn test_xdg_warning_empty_is_some() {
        assert!(xdg_config_warning(Some("")).is_some());
    }

    #[test]
    fn test_xdg_warning_unset_is_none() {
        assert_eq!(xdg_config_warning(None), None);
    }
}

