use anyhow::Result;
use dirs::config_dir;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub overlay: OverlayConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub socket: SocketConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverlayConfig {
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default)]
    pub custom_x: i32,
    #[serde(default)]
    pub custom_y: i32,
    #[serde(default = "default_max_participants")]
    pub max_participants: usize,
    #[serde(default = "default_avatar_size")]
    pub avatar_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_pulse_ms")]
    pub speaking_pulse_ms: u64,
    #[serde(default = "default_true")]
    pub show_names: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocketConfig {
    #[serde(default = "default_socket_path")]
    pub path: String,
}

fn default_position() -> String {
    "top-right".into()
}
fn default_max_participants() -> usize {
    10
}
fn default_avatar_size() -> i32 {
    40
}
fn default_theme() -> String {
    "auto".into()
}
fn default_pulse_ms() -> u64 {
    1000
}
fn default_true() -> bool {
    true
}
fn default_socket_path() -> String {
    std::env::var("XDG_RUNTIME_DIR")
        .map(|dir| format!("{}/vesktop-voice-overlay.sock", dir))
        .unwrap_or_else(|_| format!("/tmp/vesktop-voice-overlay-{}.sock", std::process::id()))
}

static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vesktop-voice-overlay")
        .join("config.toml")
});

impl Config {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(&*CONFIG_PATH)?;
        let config: Config = toml::from_str(&content)?;
        debug!("Loaded config from {:?}", CONFIG_PATH);
        Ok(config)
    }

    #[expect(dead_code)]
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = CONFIG_PATH.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&*CONFIG_PATH, content)?;
        debug!("Saved config to {:?}", CONFIG_PATH);
        Ok(())
    }

    pub fn socket_path(&self) -> &str {
        &self.socket.path
    }
}
