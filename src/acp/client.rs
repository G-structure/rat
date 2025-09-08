use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::{Message, Session, SessionId};
use crate::app::AppMessage;
use agent_client_protocol::{self as acp, Agent};

// Dummy connection for testing without ACP blocking
struct DummyConnection;

impl DummyConnection {
    fn new() -> Self {
        Self
    }
}

/// Our implementation of the ACP Client trait
pub struct RatClient {
    agent_name: String,
    message_tx: mpsc::UnboundedSender<AppMessage>,
}

impl RatClient {
    pub fn new(agent_name: String, message_tx: mpsc::UnboundedSender<AppMessage>) -> Self {
        Self {
            agent_name,
            message_tx,
        }
    }
}

impl acp::Client for RatClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> Result<acp::RequestPermissionResponse, acp::Error> {
        info!(
            "Permission requested for session {} - tool call: {:?}",
            args.session_id.0, args.tool_call
        );

        // For now, we'll automatically approve all permissions
        // TODO: Implement proper user permission dialog
        if let Some(option) = args.options.first() {
            Ok(acp::RequestPermissionResponse {
                outcome: acp::RequestPermissionOutcome::Selected {
                    option_id: option.id.clone(),
                },
            })
        } else {
            Ok(acp::RequestPermissionResponse {
                outcome: acp::RequestPermissionOutcome::Cancelled,
            })
        }
    }

    async fn write_text_file(&self, args: acp::WriteTextFileRequest) -> Result<(), acp::Error> {
        info!("Writing file: {:?}", args.path);

        match tokio::fs::write(&args.path, &args.content).await {
            Ok(()) => {
                debug!("Successfully wrote file: {:?}", args.path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to write file {:?}: {}", args.path, e);
                Err(acp::Error::internal_error())
            }
        }
    }

    async fn read_text_file(
        &self,
        args: acp::ReadTextFileRequest,
    ) -> Result<acp::ReadTextFileResponse, acp::Error> {
        info!("Reading file: {:?}", args.path);

        match tokio::fs::read_to_string(&args.path).await {
            Ok(content) => {
                let mut result_content = content;

                // Handle line-based reading if requested
                if let Some(start_line) = args.line {
                    let lines: Vec<&str> = result_content.lines().collect();
                    let start_idx = (start_line as usize).saturating_sub(1);

                    if start_idx < lines.len() {
                        let end_idx = if let Some(limit) = args.limit {
                            std::cmp::min(start_idx + limit as usize, lines.len())
                        } else {
                            lines.len()
                        };

                        result_content = lines[start_idx..end_idx].join("\n");
                    } else {
                        result_content = String::new();
                    }
                }

                debug!("Successfully read file: {:?}", args.path);
                Ok(acp::ReadTextFileResponse {
                    content: result_content,
                })
            }
            Err(e) => {
                error!("Failed to read file {:?}: {}", args.path, e);
                Err(acp::Error::internal_error())
            }
        }
    }

    async fn session_notification(&self, args: acp::SessionNotification) -> Result<(), acp::Error> {
        debug!(
            "Session notification for {}: {:?}",
            args.session_id.0, args.update
        );

        let session_id = SessionId(args.session_id.0.to_string());
        let message = Message::from_session_update(session_id.clone(), args.update);

        let app_message = AppMessage::AgentMessage {
            agent_name: self.agent_name.clone(),
            message,
        };

        if let Err(e) = self.message_tx.send(app_message) {
            error!("Failed to send session notification: {}", e);
            return Err(acp::Error::internal_error());
        }

        Ok(())
    }
}

/// ACP client that manages connection to an agent process
pub struct AcpClient {
    agent_name: String,
    command_path: String,
    process: Option<Child>,
    connection: Option<DummyConnection>,
    sessions: HashMap<SessionId, Session>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
    client: RatClient,
}

impl AcpClient {
    pub fn new(
        agent_name: &str,
        command_path: &str,
        message_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Self {
        let client = RatClient::new(agent_name.to_string(), message_tx.clone());

        Self {
            agent_name: agent_name.to_string(),
            command_path: command_path.to_string(),
            process: None,
            connection: None,
            sessions: HashMap::new(),
            message_tx,
            client,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting ACP agent: {}", self.agent_name);

        // Start the agent process
        let mut child = Command::new(&self.command_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
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

        // Clone the client for the local set
        let client_clone = self.client.clone();

        // For now, let's use a simple dummy implementation to avoid hanging
        // TODO: Implement proper ACP connection without LocalSet blocking
        warn!("Using dummy ACP implementation - agent connection will be mock");

        // Keep the process alive but don't try to establish ACP connection yet
        // This prevents hanging while we debug the ACP integration
        let connection = DummyConnection::new();
        info!("Dummy ACP connection created (agent process is running)");

        self.process = Some(child);
        self.connection = Some(connection);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping ACP agent: {}", self.agent_name);

        // Close connection first
        self.connection = None;

        // Kill the process
        if let Some(mut process) = self.process.take() {
            if let Err(e) = process.kill().await {
                warn!("Failed to kill agent process: {}", e);
            }
        }

        self.sessions.clear();
        Ok(())
    }

    pub async fn create_session(&mut self) -> Result<SessionId> {
        let _connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        // Create a dummy session ID for now
        let session_id = SessionId(format!("dummy-session-{}", uuid::Uuid::new_v4()));
        let session = Session::new(session_id.clone());
        self.sessions.insert(session_id.clone(), session);

        info!("Created new session: {}", session_id.0);
        Ok(session_id)
    }

    pub async fn send_prompt(
        &self,
        session_id: &SessionId,
        _prompt: Vec<acp::ContentBlock>,
    ) -> Result<()> {
        let _connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        debug!(
            "Mock sending prompt to session {}: (dummy implementation)",
            session_id.0,
        );

        // For now, just log the message - no actual sending
        info!("Would send prompt to session {}", session_id.0);
        Ok(())
    }

    pub async fn send_message(&self, session_id: &SessionId, content: String) -> Result<()> {
        let prompt = vec![acp::ContentBlock::Text(acp::TextContent {
            text: content,
            annotations: Default::default(),
        })];
        self.send_prompt(session_id, prompt).await
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

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn agent_name(&self) -> &str {
        &self.agent_name
    }

    // Note: ACP doesn't currently support session cancellation in the public API

    pub fn get_session_count(&self) -> usize {
        self.sessions.len()
    }
}

// We need to implement Clone for RatClient to use it with acp::ClientSideConnection
impl Clone for RatClient {
    fn clone(&self) -> Self {
        Self {
            agent_name: self.agent_name.clone(),
            message_tx: self.message_tx.clone(),
        }
    }
}
