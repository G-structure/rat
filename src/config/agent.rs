use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub claude_code: ClaudeCodeConfig,
    pub gemini: GeminiConfig,
    pub default_agent: String,
    pub auto_connect: Vec<String>,
    pub connection_timeout_seconds: u64,
    pub max_concurrent_agents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub enabled: bool,
    pub command_path: Option<PathBuf>,
    pub api_key_env: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout_seconds: u64,
    pub auto_install: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub enabled: bool,
    pub command_path: Option<PathBuf>,
    pub api_key_env: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout_seconds: u64,
    pub auto_install: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            claude_code: ClaudeCodeConfig::default(),
            gemini: GeminiConfig::default(),
            default_agent: "claude-code".to_string(),
            auto_connect: vec!["claude-code".to_string()],
            connection_timeout_seconds: 30,
            max_concurrent_agents: 5,
        }
    }
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            command_path: None, // Will auto-detect
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            model: Some("claude-3-5-sonnet-20241022".to_string()),
            max_tokens: Some(8192),
            temperature: Some(0.7),
            timeout_seconds: 300,
            auto_install: true,
        }
    }
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            command_path: None, // Will auto-detect
            api_key_env: "GOOGLE_API_KEY".to_string(),
            model: Some("gemini-2.0-flash-exp".to_string()),
            max_tokens: Some(8192),
            temperature: Some(0.7),
            timeout_seconds: 300,
            auto_install: true,
        }
    }
}

impl AgentConfig {
    pub fn validate(&self) -> Result<()> {
        if self.connection_timeout_seconds == 0 {
            return Err(anyhow::anyhow!(
                "connection_timeout_seconds must be greater than 0"
            ));
        }

        if self.max_concurrent_agents == 0 {
            return Err(anyhow::anyhow!(
                "max_concurrent_agents must be greater than 0"
            ));
        }

        let valid_agents = ["claude-code", "gemini"];
        if !valid_agents.contains(&self.default_agent.as_str()) {
            return Err(anyhow::anyhow!(
                "default_agent must be one of: {:?}",
                valid_agents
            ));
        }

        for agent in &self.auto_connect {
            if !valid_agents.contains(&agent.as_str()) {
                return Err(anyhow::anyhow!(
                    "auto_connect agent '{}' must be one of: {:?}",
                    agent,
                    valid_agents
                ));
            }
        }

        self.claude_code.validate().context("claude_code config")?;
        self.gemini.validate().context("gemini config")?;

        Ok(())
    }

    pub fn merge_with(&mut self, other: AgentConfig) {
        self.claude_code.merge_with(other.claude_code);
        self.gemini.merge_with(other.gemini);

        if other.default_agent != AgentConfig::default().default_agent {
            self.default_agent = other.default_agent;
        }
        if !other.auto_connect.is_empty() {
            self.auto_connect = other.auto_connect;
        }
        if other.connection_timeout_seconds != AgentConfig::default().connection_timeout_seconds {
            self.connection_timeout_seconds = other.connection_timeout_seconds;
        }
        if other.max_concurrent_agents != AgentConfig::default().max_concurrent_agents {
            self.max_concurrent_agents = other.max_concurrent_agents;
        }
    }

    pub fn get_agent_command_path(&self, agent_name: &str) -> Option<PathBuf> {
        match agent_name {
            "claude-code" => self.claude_code.get_command_path(),
            "gemini" => self.gemini.get_command_path(),
            _ => None,
        }
    }

    pub fn is_agent_enabled(&self, agent_name: &str) -> bool {
        match agent_name {
            "claude-code" => self.claude_code.enabled,
            "gemini" => self.gemini.enabled,
            _ => false,
        }
    }

    pub fn get_api_key_env(&self, agent_name: &str) -> Option<String> {
        match agent_name {
            "claude-code" => Some(self.claude_code.api_key_env.clone()),
            "gemini" => Some(self.gemini.api_key_env.clone()),
            _ => None,
        }
    }

    pub fn get_enabled_agents(&self) -> Vec<String> {
        let mut enabled = Vec::new();
        if self.claude_code.enabled {
            enabled.push("claude-code".to_string());
        }
        if self.gemini.enabled {
            enabled.push("gemini".to_string());
        }
        enabled
    }
}

impl ClaudeCodeConfig {
    pub fn validate(&self) -> Result<()> {
        if self.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("timeout_seconds must be greater than 0"));
        }

        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(anyhow::anyhow!("temperature must be between 0.0 and 2.0"));
            }
        }

        if let Some(tokens) = self.max_tokens {
            if tokens == 0 {
                return Err(anyhow::anyhow!("max_tokens must be greater than 0"));
            }
        }

        Ok(())
    }

    pub fn merge_with(&mut self, other: ClaudeCodeConfig) {
        if other.enabled != ClaudeCodeConfig::default().enabled {
            self.enabled = other.enabled;
        }
        if other.command_path.is_some() {
            self.command_path = other.command_path;
        }
        if other.api_key_env != ClaudeCodeConfig::default().api_key_env {
            self.api_key_env = other.api_key_env;
        }
        if other.model.is_some() {
            self.model = other.model;
        }
        if other.max_tokens.is_some() {
            self.max_tokens = other.max_tokens;
        }
        if other.temperature.is_some() {
            self.temperature = other.temperature;
        }
        if other.timeout_seconds != ClaudeCodeConfig::default().timeout_seconds {
            self.timeout_seconds = other.timeout_seconds;
        }
        if other.auto_install != ClaudeCodeConfig::default().auto_install {
            self.auto_install = other.auto_install;
        }
    }

    pub fn get_command_path(&self) -> Option<PathBuf> {
        self.command_path.clone().or_else(|| {
            // Try to find claude-code-acp in PATH
            which::which("claude-code-acp").ok()
        })
    }
}

impl GeminiConfig {
    pub fn validate(&self) -> Result<()> {
        if self.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("timeout_seconds must be greater than 0"));
        }

        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(anyhow::anyhow!("temperature must be between 0.0 and 2.0"));
            }
        }

        if let Some(tokens) = self.max_tokens {
            if tokens == 0 {
                return Err(anyhow::anyhow!("max_tokens must be greater than 0"));
            }
        }

        Ok(())
    }

    pub fn merge_with(&mut self, other: GeminiConfig) {
        if other.enabled != GeminiConfig::default().enabled {
            self.enabled = other.enabled;
        }
        if other.command_path.is_some() {
            self.command_path = other.command_path;
        }
        if other.api_key_env != GeminiConfig::default().api_key_env {
            self.api_key_env = other.api_key_env;
        }
        if other.model.is_some() {
            self.model = other.model;
        }
        if other.max_tokens.is_some() {
            self.max_tokens = other.max_tokens;
        }
        if other.temperature.is_some() {
            self.temperature = other.temperature;
        }
        if other.timeout_seconds != GeminiConfig::default().timeout_seconds {
            self.timeout_seconds = other.timeout_seconds;
        }
        if other.auto_install != GeminiConfig::default().auto_install {
            self.auto_install = other.auto_install;
        }
    }

    pub fn get_command_path(&self) -> Option<PathBuf> {
        self.command_path.clone().or_else(|| {
            // Try to find gemini in PATH
            which::which("gemini").ok()
        })
    }
}
