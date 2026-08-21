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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserDisplayMode {
    Always,
    #[default]
    SpeakingOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NameDisplayMode {
    Always,
    #[default]
    SpeakingOnly,
    Never,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AvatarSizeMode {
    #[default]
    Small,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
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
    #[serde(default)]
    pub user_display: UserDisplayMode,
    #[serde(default)]
    pub name_display: NameDisplayMode,
    #[serde(default)]
    pub avatar_size_mode: AvatarSizeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlaySettings {
    pub enabled: bool,
    pub position: String,
    pub custom_x: i32,
    pub custom_y: i32,
    pub user_display: UserDisplayMode,
    pub name_display: NameDisplayMode,
    pub avatar_size_mode: AvatarSizeMode,
}

impl OverlaySettings {
    pub fn is_valid(&self) -> bool {
        matches!(
            self.position.as_str(),
            "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center" | "custom"
        ) && (-32_768..=32_768).contains(&self.custom_x)
            && (-32_768..=32_768).contains(&self.custom_y)
    }
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            position: default_position(),
            custom_x: 0,
            custom_y: 0,
            max_participants: default_max_participants(),
            avatar_size: default_avatar_size(),
            user_display: UserDisplayMode::default(),
            name_display: NameDisplayMode::default(),
            avatar_size_mode: AvatarSizeMode::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_pulse_ms")]
    pub speaking_pulse_ms: u64,
    #[serde(default = "default_true")]
    pub show_names: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            speaking_pulse_ms: default_pulse_ms(),
            show_names: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketConfig {
    #[serde(default = "default_socket_path")]
    pub path: String,
}

impl Default for SocketConfig {
    fn default() -> Self {
        Self {
            path: default_socket_path(),
        }
    }
}

fn default_position() -> String {
    "top-right".into()
}
fn default_max_participants() -> usize {
    10
}
fn default_avatar_size() -> i32 {
    28
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

    pub fn apply_overlay_settings(&mut self, settings: OverlaySettings) {
        self.overlay.enabled = settings.enabled;
        self.overlay.position = settings.position;
        self.overlay.custom_x = settings.custom_x;
        self.overlay.custom_y = settings.custom_y;
        self.overlay.user_display = settings.user_display;
        self.overlay.name_display = settings.name_display;
        self.overlay.avatar_size_mode = settings.avatar_size_mode;
    }

    pub fn avatar_size_px(&self) -> i32 {
        match self.overlay.avatar_size_mode {
            AvatarSizeMode::Small => 28,
            AvatarSizeMode::Large => 40,
        }
    }
}
