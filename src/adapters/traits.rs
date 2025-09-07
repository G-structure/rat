use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::acp::{Message, Session, SessionId};
use crate::app::AppMessage;

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Get the name of this agent
    fn name(&self) -> &str;

    /// Check if the agent is currently connected
    fn is_connected(&self) -> bool;

    /// Start the agent connection
    async fn start(&mut self) -> Result<()>;

    /// Stop the agent connection
    async fn stop(&mut self) -> Result<()>;

    /// Create a new session with this agent
    async fn create_session(&mut self) -> Result<SessionId>;

    /// Send a message to a specific session
    async fn send_message(&mut self, session_id: &SessionId, content: String) -> Result<()>;

    /// Get a list of active session IDs
    fn get_session_ids(&self) -> Vec<SessionId>;

    /// Get a specific session
    fn get_session(&self, session_id: &SessionId) -> Option<&Session>;

    /// Get a mutable reference to a specific session
    fn get_session_mut(&mut self, session_id: &SessionId) -> Option<&mut Session>;

    /// Handle periodic updates (called from the main loop)
    async fn tick(&mut self) -> Result<()>;

    /// Get the health status of the agent
    fn health_status(&self) -> AgentHealth;

    /// Get agent capabilities
    fn capabilities(&self) -> AgentCapabilities;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AgentCapabilities {
    pub supports_code_editing: bool,
    pub supports_file_operations: bool,
    pub supports_terminal: bool,
    pub supports_web_search: bool,
    pub supports_image_input: bool,
    pub max_context_tokens: Option<u32>,
    pub supported_languages: Vec<String>,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            supports_code_editing: true,
            supports_file_operations: true,
            supports_terminal: false,
            supports_web_search: false,
            supports_image_input: false,
            max_context_tokens: None,
            supported_languages: vec!["text".to_string()],
        }
    }
}

impl AgentCapabilities {
    pub fn claude_code() -> Self {
        Self {
            supports_code_editing: true,
            supports_file_operations: true,
            supports_terminal: true,
            supports_web_search: false,
            supports_image_input: true,
            max_context_tokens: Some(200_000),
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "html".to_string(),
                "css".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "toml".to_string(),
                "markdown".to_string(),
            ],
        }
    }

    pub fn gemini() -> Self {
        Self {
            supports_code_editing: true,
            supports_file_operations: true,
            supports_terminal: true,
            supports_web_search: true,
            supports_image_input: true,
            max_context_tokens: Some(1_000_000),
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
                "java".to_string(),
                "c".to_string(),
                "cpp".to_string(),
                "html".to_string(),
                "css".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "toml".to_string(),
                "markdown".to_string(),
            ],
        }
    }
}

impl std::fmt::Display for AgentHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentHealth::Healthy => write!(f, "Healthy"),
            AgentHealth::Degraded { reason } => write!(f, "Degraded: {}", reason),
            AgentHealth::Unhealthy { reason } => write!(f, "Unhealthy: {}", reason),
            AgentHealth::Disconnected => write!(f, "Disconnected"),
            AgentHealth::Unknown => write!(f, "Unknown"),
        }
    }
}
