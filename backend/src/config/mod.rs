pub mod agent;
pub mod project;
pub mod ui;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use agent::AgentConfig;
pub use project::ProjectConfig;
pub use ui::UiConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agents: AgentConfig,
    pub ui: UiConfig,
    pub project: ProjectConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_level: String,
    pub auto_save_sessions: bool,
    pub max_session_history: usize,
    pub permission_timeout_seconds: u64,
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            agents: AgentConfig::default(),
            ui: UiConfig::default(),
            project: ProjectConfig::default(),
            general: GeneralConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            auto_save_sessions: true,
            max_session_history: 1000,
            permission_timeout_seconds: 300, // 5 minutes
            config_dir: None,
            data_dir: None,
        }
    }
}

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path.as_ref())
            .await
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path.as_ref()))?;

        Ok(config)
    }

    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        tokio::fs::write(path.as_ref(), content)
            .await
            .with_context(|| format!("Failed to write config file: {:?}", path.as_ref()))?;

        Ok(())
    }

    pub fn get_config_dir() -> Result<PathBuf> {
        if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(config_home).join("rat"))
        } else if let Some(home) = dirs::home_dir() {
            Ok(home.join(".config").join("rat"))
        } else {
            Err(anyhow::anyhow!("Could not determine config directory"))
        }
    }

    pub fn get_data_dir() -> Result<PathBuf> {
        if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
            Ok(PathBuf::from(data_home).join("rat"))
        } else if let Some(home) = dirs::home_dir() {
            Ok(home.join(".local").join("share").join("rat"))
        } else {
            Err(anyhow::anyhow!("Could not determine data directory"))
        }
    }

    pub fn get_default_config_file() -> Result<PathBuf> {
        Ok(Self::get_config_dir()?.join("config.toml"))
    }

    pub async fn load_or_create_default() -> Result<(Self, PathBuf)> {
        let config_file = Self::get_default_config_file()?;

        if config_file.exists() {
            let config = Self::from_file(&config_file).await?;
            Ok((config, config_file))
        } else {
            let mut config = Self::default();

            // Set directories in config
            config.general.config_dir = Some(Self::get_config_dir()?);
            config.general.data_dir = Some(Self::get_data_dir()?);

            // Create default config file
            config.save_to_file(&config_file).await?;

            Ok((config, config_file))
        }
    }

    pub fn validate(&self) -> Result<()> {
        // Validate agent configurations
        self.agents.validate()?;

        // Validate UI configuration
        self.ui.validate()?;

        // Validate general configuration
        if self.general.max_session_history == 0 {
            return Err(anyhow::anyhow!(
                "max_session_history must be greater than 0"
            ));
        }

        if self.general.permission_timeout_seconds == 0 {
            return Err(anyhow::anyhow!(
                "permission_timeout_seconds must be greater than 0"
            ));
        }

        Ok(())
    }

    pub fn merge_with(&mut self, other: Config) {
        // Merge configurations, with other taking precedence
        self.agents.merge_with(other.agents);
        self.ui.merge_with(other.ui);
        self.project.merge_with(other.project);

        // For general config, replace non-default values
        if other.general.log_level != GeneralConfig::default().log_level {
            self.general.log_level = other.general.log_level;
        }
        if other.general.auto_save_sessions != GeneralConfig::default().auto_save_sessions {
            self.general.auto_save_sessions = other.general.auto_save_sessions;
        }
        if other.general.max_session_history != GeneralConfig::default().max_session_history {
            self.general.max_session_history = other.general.max_session_history;
        }
        if other.general.permission_timeout_seconds
            != GeneralConfig::default().permission_timeout_seconds
        {
            self.general.permission_timeout_seconds = other.general.permission_timeout_seconds;
        }
        if other.general.config_dir.is_some() {
            self.general.config_dir = other.general.config_dir;
        }
        if other.general.data_dir.is_some() {
            self.general.data_dir = other.general.data_dir;
        }
    }

    pub fn get_effective_config_dir(&self) -> PathBuf {
        self.general
            .config_dir
            .clone()
            .unwrap_or_else(|| Self::get_config_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    pub fn get_effective_data_dir(&self) -> PathBuf {
        self.general
            .data_dir
            .clone()
            .unwrap_or_else(|| Self::get_data_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}
