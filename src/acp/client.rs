use anyhow::{Context, Result};
use futures::StreamExt;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use uuid::Uuid;

use agent_client_protocol::{self as acp, Client};

use super::{Message, Session, SessionId};

pub struct AcpClient {
    process: Option<Child>,
    client: Option<acp::client::Client>,
    sessions: HashMap<SessionId, Session>,
    message_tx: mpsc::UnboundedSender<Message>,
    message_rx: Option<mpsc::UnboundedReceiver<Message>>,
    agent_name: String,
    command_path: String,
}

impl AcpClient {
    pub fn new(agent_name: &str, command_path: &str) -> Result<Self> {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Ok(AcpClient {
            process: None,
            client: None,
            sessions: HashMap::new(),
            message_tx,
            message_rx: Some(message_rx),
            agent_name: agent_name.to_string(),
            command_path: command_path.to_string(),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting ACP agent: {}", self.agent_name);

        // Start the agent process
        let mut child = Command::new(&self.command_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start agent: {}", self.command_path))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin handle"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout handle"))?;

        // Create ACP client
        let client = acp::client::Client::new(stdin, stdout);

        // Initialize the connection
        let init_response = client
            .initialize(acp::InitializeRequest {
                client_name: "rat".to_string(),
                client_version: env!("CARGO_PKG_VERSION").to_string(),
            })
            .await?;

        info!(
            "ACP agent initialized: protocol_version={:?}, capabilities={:?}",
            init_response.protocol_version, init_response.agent_capabilities
        );

        self.process = Some(child);
        self.client = Some(client);

        // Start message processing loop
        self.start_message_loop().await?;

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping ACP agent: {}", self.agent_name);

        if let Some(mut process) = self.process.take() {
            if let Err(e) = process.kill().await {
                warn!("Failed to kill agent process: {}", e);
            }
        }

        self.client = None;
        self.sessions.clear();

        Ok(())
    }

    pub async fn create_session(&mut self) -> Result<SessionId> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        let session_response = client.new_session(acp::NewSessionRequest {}).await?;
        let session_id = SessionId(session_response.session_id.0.to_string());

        let session = Session::new(session_id.clone());
        self.sessions.insert(session_id.clone(), session);

        info!("Created new session: {}", session_id.0);
        Ok(session_id)
    }

    pub async fn send_prompt(
        &self,
        session_id: &SessionId,
        prompt: Vec<acp::Content>,
    ) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        debug!(
            "Sending prompt to session {}: {} content items",
            session_id.0,
            prompt.len()
        );

        let request = acp::PromptRequest {
            session_id: acp::SessionId(session_id.0.clone().into()),
            prompt,
        };

        client.prompt(request).await?;
        Ok(())
    }

    pub fn get_session(&self, session_id: &SessionId) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    pub fn get_session_mut(&mut self, session_id: &SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    pub fn list_sessions(&self) -> Vec<&SessionId> {
        self.sessions.keys().collect()
    }

    pub fn take_message_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<Message>> {
        self.message_rx.take()
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn agent_name(&self) -> &str {
        &self.agent_name
    }

    async fn start_message_loop(&self) -> Result<()> {
        // This would handle incoming messages from the agent
        // For now, we'll implement a basic structure
        info!("Message loop started for agent: {}", self.agent_name);
        Ok(())
    }

    pub async fn handle_notification(
        &mut self,
        notification: acp::SessionNotification,
    ) -> Result<()> {
        debug!(
            "Received notification for session: {}",
            notification.session_id.0
        );

        let session_id = SessionId(notification.session_id.0.to_string());
        let message = Message::from_session_update(session_id, notification.update);

        if let Err(e) = self.message_tx.send(message) {
            error!("Failed to send message to UI: {}", e);
        }

        Ok(())
    }
}
