use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use super::traits::{AgentAdapter, AgentCapabilities, AgentHealth};
use crate::acp::{AcpClient, Message, Session, SessionId};
use crate::app::AppMessage;
use crate::config::agent::ClaudeCodeConfig;

pub struct ClaudeCodeAdapter {
    name: String,
    config: ClaudeCodeConfig,
    client: Option<AcpClient>,
    sessions: HashMap<SessionId, Session>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
    health: AgentHealth,
    last_health_check: Option<std::time::Instant>,
    command_path: PathBuf,
}

impl ClaudeCodeAdapter {
    pub async fn new(
        command_path: PathBuf,
        config: ClaudeCodeConfig,
        message_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Result<Self> {
        let adapter = Self {
            name: "claude-code".to_string(),
            config,
            client: None,
            sessions: HashMap::new(),
            message_tx,
            health: AgentHealth::Disconnected,
            last_health_check: None,
            command_path,
        };

        // Verify command exists
        adapter.verify_command().await?;

        Ok(adapter)
    }

    async fn verify_command(&self) -> Result<()> {
        if !self.command_path.exists() {
            if self.config.auto_install {
                info!("Claude Code command not found, attempting auto-install");
                self.auto_install().await?;
            } else {
                return Err(anyhow::anyhow!(
                    "Claude Code command not found: {:?}",
                    self.command_path
                ));
            }
        }

        // Test command execution
        let output = timeout(
            Duration::from_secs(10),
            Command::new(&self.command_path).arg("--version").output(),
        )
        .await
        .context("Command verification timeout")?
        .context("Failed to execute command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command verification failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Claude Code command verified: {:?}", self.command_path);
        Ok(())
    }

    async fn auto_install(&self) -> Result<()> {
        info!("Auto-installing claude-code-acp via npm");

        let output = Command::new("npm")
            .args(&["install", "-g", "@zed-industries/claude-code-acp"])
            .output()
            .await
            .context("Failed to run npm install")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to install claude-code-acp: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        info!("Successfully installed claude-code-acp");
        Ok(())
    }

    async fn check_environment(&self) -> Result<()> {
        // Check for required environment variables
        if std::env::var(&self.config.api_key_env).is_err() {
            return Err(anyhow::anyhow!(
                "Environment variable '{}' not set",
                self.config.api_key_env
            ));
        }

        Ok(())
    }

    async fn update_health(&mut self) {
        let now = std::time::Instant::now();

        // Only check health every 30 seconds
        if let Some(last_check) = self.last_health_check {
            if now.duration_since(last_check) < Duration::from_secs(30) {
                return;
            }
        }

        self.last_health_check = Some(now);

        let new_health = if self.client.is_none() {
            AgentHealth::Disconnected
        } else if let Err(e) = self.check_environment().await {
            AgentHealth::Unhealthy {
                reason: e.to_string(),
            }
        } else {
            // Could add more health checks here (memory usage, response time, etc.)
            AgentHealth::Healthy
        };

        if self.health != new_health {
            debug!(
                "Claude Code health changed: {} -> {}",
                self.health, new_health
            );
            self.health = new_health;
        }
    }
}

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_connected(&self) -> bool {
        self.client
            .as_ref()
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting Claude Code adapter");

        // Check environment first
        self.check_environment()
            .await
            .context("Environment check failed")?;

        // Create and start ACP client
        let mut client = AcpClient::new(&self.name, self.command_path.to_str().unwrap())?;

        client.start().await.context("Failed to start ACP client")?;

        self.client = Some(client);
        self.health = AgentHealth::Healthy;

        info!("Claude Code adapter started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping Claude Code adapter");

        if let Some(mut client) = self.client.take() {
            client.stop().await.context("Failed to stop ACP client")?;
        }

        self.sessions.clear();
        self.health = AgentHealth::Disconnected;

        info!("Claude Code adapter stopped");
        Ok(())
    }

    async fn create_session(&mut self) -> Result<SessionId> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        let session_id = client
            .create_session()
            .await
            .context("Failed to create session")?;

        let session = Session::with_agent(session_id.clone(), self.name.clone());
        self.sessions.insert(session_id.clone(), session);

        debug!("Created Claude Code session: {}", session_id.0);
        Ok(session_id)
    }

    async fn send_message(&mut self, session_id: &SessionId, content: String) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        // Convert string content to ACP Content
        let acp_content = vec![agent_client_protocol::Content::Text(content.clone())];

        client
            .send_prompt(session_id, acp_content)
            .await
            .context("Failed to send message")?;

        // Add user message to session
        if let Some(session) = self.sessions.get_mut(session_id) {
            let user_message = Message::user_prompt(
                session_id.clone(),
                vec![agent_client_protocol::Content::Text(content)],
            );
            session.add_message(user_message);
        }

        Ok(())
    }

    fn get_session_ids(&self) -> Vec<SessionId> {
        self.sessions.keys().cloned().collect()
    }

    fn get_session(&self, session_id: &SessionId) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    fn get_session_mut(&mut self, session_id: &SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    async fn tick(&mut self) -> Result<()> {
        // Update health status
        self.update_health().await;

        // Handle any incoming messages from the ACP client
        if let Some(client) = &mut self.client {
            // Process any notifications or updates
            // The actual message handling would be done through the client's message receiver
        }

        Ok(())
    }

    fn health_status(&self) -> AgentHealth {
        self.health.clone()
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities::claude_code()
    }
}

impl Drop for ClaudeCodeAdapter {
    fn drop(&mut self) {
        if self.is_connected() {
            warn!("ClaudeCodeAdapter dropped while still connected");
        }
    }
}
