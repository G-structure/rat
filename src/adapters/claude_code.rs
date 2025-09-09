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

use super::{
    agent_installer::{AgentCommand, AgentInstaller},
    traits::{AgentAdapter, AgentCapabilities, AgentHealth},
};
use crate::acp::Session;
use crate::acp::{AcpClient, Message, SessionId};
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
    installer: AgentInstaller,
    command: Option<AgentCommand>,
}

impl ClaudeCodeAdapter {
    pub async fn new(
        config: ClaudeCodeConfig,
        message_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Result<Self> {
        let installer = AgentInstaller::new().context("Failed to create agent installer")?;

        Ok(Self {
            name: "claude-code".to_string(),
            config,
            client: None,
            sessions: HashMap::new(),
            message_tx,
            health: AgentHealth::Disconnected,
            last_health_check: None,
            installer,
            command: None,
        })
    }

    async fn get_or_install_command(&mut self) -> Result<&AgentCommand> {
        if self.command.is_none() {
            info!("Getting or installing Claude Code agent...");
            let command = self
                .installer
                .get_or_install_claude_code()
                .await
                .context("Failed to get or install Claude Code")?;

            // Skip verification for now to avoid hanging
            // TODO: Implement non-blocking verification
            info!("Skipping command verification to avoid hanging");

            self.command = Some(command);
        }

        Ok(self.command.as_ref().unwrap())
    }

    async fn check_environment(&self) -> Result<()> {
        // Claude Code handles authentication internally through login flow
        // No environment variables required
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

#[async_trait(?Send)]
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

        // Store values we need to avoid borrowing conflicts
        let name = self.name.clone();
        let message_tx = self.message_tx.clone();

        // Get or install the command
        let command = self.get_or_install_command().await?;

        // Create and start ACP client (include args/env)
        let mut client = AcpClient::new(
            &name,
            command.path.to_str().unwrap(),
            command.args.clone(),
            command.env.clone(),
            message_tx,
        );

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

        let session = Session::new(session_id.clone());
        self.sessions.insert(session_id.clone(), session);

        debug!("Created Claude Code session: {}", session_id.0);
        Ok(session_id)
    }

    async fn send_message(&mut self, session_id: &SessionId, content: String) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        client
            .send_message(session_id, content)
            .await
            .context("Failed to send message")?;

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
