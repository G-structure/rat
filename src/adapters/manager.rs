use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use tokio::sync::mpsc;

use super::{claude_code::ClaudeCodeAdapter, gemini::GeminiAdapter, AgentAdapter};
use crate::acp::{Message, SessionId};
use crate::app::AppMessage;
use crate::config::AgentConfig;

pub struct AgentManager {
    config: AgentConfig,
    agents: HashMap<String, Box<dyn AgentAdapter>>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
}

impl AgentManager {
    pub async fn new(
        config: AgentConfig,
        message_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Result<Self> {
        let mut manager = Self {
            config,
            agents: HashMap::new(),
            message_tx,
        };

        manager.initialize_agents().await?;
        Ok(manager)
    }

    async fn initialize_agents(&mut self) -> Result<()> {
        info!("Initializing agent adapters");

        // Initialize Claude Code adapter if enabled
        if self.config.claude_code.enabled {
            match self.create_claude_code_adapter().await {
                Ok(adapter) => {
                    info!("Claude Code adapter initialized");
                    self.agents.insert("claude-code".to_string(), adapter);
                }
                Err(e) => {
                    warn!("Failed to initialize Claude Code adapter: {}", e);
                    // Don't treat this as a fatal error - we'll try to install when connecting
                    let _ = self.message_tx.send(AppMessage::Error {
                        error: format!(
                            "Claude Code adapter available but not immediately ready: {}",
                            e
                        ),
                    });
                }
            }
        }

        // Initialize Gemini adapter if enabled
        if self.config.gemini.enabled {
            match self.create_gemini_adapter().await {
                Ok(adapter) => {
                    info!("Gemini adapter initialized");
                    self.agents.insert("gemini".to_string(), adapter);
                }
                Err(e) => {
                    warn!("Failed to initialize Gemini adapter: {}", e);
                    let _ = self.message_tx.send(AppMessage::Error {
                        error: format!("Failed to initialize Gemini: {}", e),
                    });
                }
            }
        }

        Ok(())
    }

    async fn create_claude_code_adapter(&self) -> Result<Box<dyn AgentAdapter>> {
        let adapter =
            ClaudeCodeAdapter::new(self.config.claude_code.clone(), self.message_tx.clone())
                .await?;

        Ok(Box::new(adapter))
    }

    async fn create_gemini_adapter(&self) -> Result<Box<dyn AgentAdapter>> {
        let adapter =
            GeminiAdapter::new(self.config.gemini.clone(), self.message_tx.clone()).await?;

        Ok(Box::new(adapter))
    }

    pub async fn connect_agent(&mut self, agent_name: &str) -> Result<()> {
        info!("Connecting to agent: {}", agent_name);

        let agent = self
            .agents
            .get_mut(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        if agent.is_connected() {
            debug!("Agent '{}' is already connected", agent_name);
            return Ok(());
        }

        agent
            .start()
            .await
            .with_context(|| format!("Failed to start agent '{}'", agent_name))?;

        let _ = self.message_tx.send(AppMessage::AgentConnected {
            agent_name: agent_name.to_string(),
        });

        info!("Successfully connected to agent: {}", agent_name);
        Ok(())
    }

    pub async fn disconnect_agent(&mut self, agent_name: &str) -> Result<()> {
        info!("Disconnecting from agent: {}", agent_name);

        let agent = self
            .agents
            .get_mut(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        agent
            .stop()
            .await
            .with_context(|| format!("Failed to stop agent '{}'", agent_name))?;

        let _ = self.message_tx.send(AppMessage::AgentDisconnected {
            agent_name: agent_name.to_string(),
        });

        info!("Successfully disconnected from agent: {}", agent_name);
        Ok(())
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        info!("Disconnecting all agents");

        let agent_names: Vec<String> = self.agents.keys().cloned().collect();
        for agent_name in agent_names {
            if let Err(e) = self.disconnect_agent(&agent_name).await {
                error!("Failed to disconnect agent '{}': {}", agent_name, e);
            }
        }

        Ok(())
    }

    pub async fn create_session(&mut self, agent_name: &str) -> Result<SessionId> {
        debug!("Creating session for agent: {}", agent_name);

        let agent = self
            .agents
            .get_mut(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        if !agent.is_connected() {
            info!("Agent '{}' not connected; attempting to connect...", agent_name);
            if let Err(e) = agent.start().await {
                let msg = format!("Failed to start agent '{}': {}", agent_name, e);
                let _ = self.message_tx.send(AppMessage::Error { error: msg.clone() });
                return Err(anyhow::anyhow!(msg));
            }
            let _ = self.message_tx.send(AppMessage::AgentConnected {
                agent_name: agent_name.to_string(),
            });
        }

        match agent.create_session().await {
            Ok(session_id) => {
                let _ = self.message_tx.send(AppMessage::SessionCreated {
                    agent_name: agent_name.to_string(),
                    session_id: session_id.clone(),
                });

                info!("Created session {} for agent {}", session_id.0, agent_name);
                Ok(session_id)
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to create session for agent '{}': {}",
                    agent_name, e
                );
                let _ = self.message_tx.send(AppMessage::Error {
                    error: error_msg.clone(),
                });
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    pub async fn send_message(
        &mut self,
        agent_name: &str,
        session_id: &SessionId,
        content: String,
    ) -> Result<()> {
        debug!(
            "Sending message to agent '{}' session '{}'",
            agent_name, session_id.0
        );

        let agent = self
            .agents
            .get_mut(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        agent
            .send_message(session_id, content)
            .await
            .with_context(|| format!("Failed to send message to agent '{}'", agent_name))?;

        Ok(())
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Update all agents
        for (agent_name, agent) in &mut self.agents {
            if let Err(e) = agent.tick().await {
                warn!("Agent '{}' tick error: {}", agent_name, e);
            }
        }

        Ok(())
    }

    pub fn get_agent_names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    pub fn get_connected_agents(&self) -> Vec<String> {
        self.agents
            .iter()
            .filter(|(_, agent)| agent.is_connected())
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn get_active_sessions(&self) -> HashMap<String, Vec<SessionId>> {
        self.agents
            .iter()
            .map(|(name, agent)| (name.clone(), agent.get_session_ids()))
            .collect()
    }

    pub fn is_agent_connected(&self, agent_name: &str) -> bool {
        self.agents
            .get(agent_name)
            .map(|agent| agent.is_connected())
            .unwrap_or(false)
    }

    pub fn get_agent_health(&self, agent_name: &str) -> Option<super::traits::AgentHealth> {
        self.agents
            .get(agent_name)
            .map(|agent| agent.health_status())
    }

    pub fn get_agent_capabilities(
        &self,
        agent_name: &str,
    ) -> Option<super::traits::AgentCapabilities> {
        self.agents
            .get(agent_name)
            .map(|agent| agent.capabilities())
    }

    pub fn get_all_agent_health(&self) -> HashMap<String, super::traits::AgentHealth> {
        self.agents
            .iter()
            .map(|(name, agent)| (name.clone(), agent.health_status()))
            .collect()
    }

    #[cfg(test)]
    pub fn insert_agent_for_test(
        &mut self,
        name: String,
        adapter: Box<dyn AgentAdapter>,
    ) {
        self.agents.insert(name, adapter);
    }

    pub fn register_agent(&mut self, name: String, adapter: Box<dyn AgentAdapter>) {
        self.agents.insert(name, adapter);
    }

    pub async fn auto_connect_agents(&mut self) -> Result<()> {
        info!("Auto-connecting configured agents");

        let auto_connect_list = self.config.auto_connect.clone();
        for agent_name in auto_connect_list {
            if self.agents.contains_key(&agent_name) {
                if let Err(e) = self.connect_agent(&agent_name).await {
                    warn!("Failed to auto-connect agent '{}': {}", agent_name, e);
                    let _ = self.message_tx.send(AppMessage::Error {
                        error: format!("Failed to auto-connect {}: {}", agent_name, e),
                    });
                }
            } else {
                warn!("Auto-connect agent '{}' not available", agent_name);
            }
        }

        Ok(())
    }

    pub fn get_session_count(&self) -> usize {
        self.agents
            .values()
            .map(|agent| agent.get_session_ids().len())
            .sum()
    }

    pub fn get_max_concurrent_agents(&self) -> usize {
        self.config.max_concurrent_agents
    }

    pub fn get_connected_count(&self) -> usize {
        self.agents
            .values()
            .filter(|agent| agent.is_connected())
            .count()
    }

    pub fn can_connect_more_agents(&self) -> bool {
        self.get_connected_count() < self.get_max_concurrent_agents()
    }

    pub async fn handle_agent_message(&self, agent_name: String, message: Message) -> Result<()> {
        let _ = self.message_tx.send(AppMessage::AgentMessage {
            agent_name,
            message,
        });
        Ok(())
    }
}
