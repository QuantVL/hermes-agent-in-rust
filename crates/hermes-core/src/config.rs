//! Configuration types mirroring the Electron Hermes `config.yaml` schema.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced while loading or saving configuration.
///
/// Note: each source error type appears exactly once. `thiserror` derives one
/// `From<E>` implementation per `#[from]` field; repeating a source type makes
/// the generated impls conflict (E0119).
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config directory could not be determined")]
    NoConfigDir,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub font_size: u16,
    pub font_family: String,
    pub line_height: f32,
    pub vim_mode: bool,
    pub tab_size: u8,
    pub word_wrap: bool,
    pub minimap: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: 14,
            font_family: "JetBrains Mono".into(),
            line_height: 1.6,
            vim_mode: false,
            tab_size: 2,
            word_wrap: true,
            minimap: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub max_message_size: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 0,
            timeout_ms: 60_000,
            max_message_size: 16 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub name: String,
    pub kind: String,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    pub default_model: String,
    pub providers: Vec<ModelProvider>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_model: "hermes-3-70b".into(),
            providers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolsConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    pub auto_approve: Vec<String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            enabled: vec!["terminal".into(), "file".into(), "web_search".into()],
            disabled: Vec::new(),
            auto_approve: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProfileConfig {
    pub name: String,
    pub gateway: Option<GatewayConfig>,
    pub models: Option<ModelConfig>,
    pub tools: Option<ToolsConfig>,
    pub editor: Option<EditorConfig>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            name: "default".into(),
            gateway: None,
            models: None,
            tools: None,
            editor: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub profile: String,
    pub theme: ThemeMode,
    pub editor: EditorConfig,
    pub gateway: GatewayConfig,
    pub models: ModelConfig,
    pub tools: ToolsConfig,
    pub profiles: HashMap<String, ProfileConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("default".into(), ProfileConfig::default());
        Self {
            profile: "default".into(),
            theme: ThemeMode::System,
            editor: EditorConfig::default(),
            gateway: GatewayConfig::default(),
            models: ModelConfig::default(),
            tools: ToolsConfig::default(),
            profiles,
        }
    }
}

impl Config {
    /// Load configuration from the platform config directory, or return the
    /// defaults when no config file exists yet.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_yaml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Persist configuration to the platform config directory.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let base = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(base.join("hermes-agent-rust").join("config.yaml"))
    }

    pub fn data_dir() -> Result<PathBuf, ConfigError> {
        let base = dirs::data_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(base.join("hermes-agent-rust"))
    }

    /// Active profile config, falling back to the `default` profile, then to
    /// a fresh default value. Returns an owned value so no temporary reference
    /// escapes the function (E0515).
    pub fn active_profile(&self) -> ProfileConfig {
        self.profiles
            .get(&self.profile)
            .cloned()
            .or_else(|| self.profiles.get("default").cloned())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_default_profile() {
        let cfg = Config::default();
        assert_eq!(cfg.profile, "default");
        assert!(cfg.profiles.contains_key("default"));
        assert_eq!(cfg.active_profile().name, "default");
    }

    #[test]
    fn config_round_trips_through_yaml() {
        let cfg = Config::default();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.profile, cfg.profile);
        assert_eq!(parsed.theme, cfg.theme);
    }
}
