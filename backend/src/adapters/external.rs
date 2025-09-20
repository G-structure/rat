use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{info, warn};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::acp::{AcpClient, Session, SessionId};
use crate::app::AppMessage;

use super::traits::{AgentAdapter, AgentCapabilities, AgentHealth};

#[derive(Debug, Clone)]
pub struct ExternalAgentSpec {
    pub name: String,
    pub path: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

pub struct ExternalCmdAdapter {
    spec: ExternalAgentSpec,
    client: Option<AcpClient>,
    sessions: HashMap<SessionId, Session>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
    health: AgentHealth,
}

impl ExternalCmdAdapter {
    pub fn new(spec: ExternalAgentSpec, message_tx: mpsc::UnboundedSender<AppMessage>) -> Self {
        Self {
            spec,
            client: None,
            sessions: HashMap::new(),
            message_tx,
            health: AgentHealth::Disconnected,
        }
    }
}

#[async_trait(?Send)]
impl AgentAdapter for ExternalCmdAdapter {
    fn name(&self) -> &str {
        &self.spec.name
    }

    fn is_connected(&self) -> bool {
        self.client
            .as_ref()
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting external agent: {}", self.spec.name);

        let mut client = AcpClient::new(
            &self.spec.name,
            &self.spec.path,
            self.spec.args.clone(),
            self.spec.env.clone(),
            self.message_tx.clone(),
            None,
        );
        client.start().await.context("Failed to start ACP client")?;

        self.client = Some(client);
        self.health = AgentHealth::Healthy;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(mut client) = self.client.take() {
            client.stop().await.context("Failed to stop ACP client")?;
        }
        self.sessions.clear();
        self.health = AgentHealth::Disconnected;
        Ok(())
    }

    async fn create_session(&mut self) -> Result<SessionId> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;
        let session_id = client.create_session().await?;
        self.sessions
            .insert(session_id.clone(), Session::new(session_id.clone()));
        Ok(session_id)
    }

    async fn send_message(&mut self, session_id: &SessionId, content: String) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;
        client.send_message(session_id, content).await
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
        Ok(())
    }

    fn health_status(&self) -> AgentHealth {
        self.health.clone()
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities::default()
    }
}

