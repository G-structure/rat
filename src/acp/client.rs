use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::{Message, Session, SessionId};
use crate::app::AppMessage;
use agent_client_protocol::{self as acp, Agent};

// Commands that can be sent to the ACP thread
#[derive(Debug)]
enum AcpCommand {
    CreateSession {
        respond_to: oneshot::Sender<Result<String>>,
    },
    SendPrompt {
        session_id: String,
        prompt: Vec<acp::ContentBlock>,
        respond_to: oneshot::Sender<Result<()>>,
    },
}

// Connection wrapper that communicates with ACP thread
struct RealAcpConnection {
    command_tx: mpsc::UnboundedSender<AcpCommand>,
}

impl RealAcpConnection {
    async fn create_session(&self) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(AcpCommand::CreateSession { respond_to: tx })
            .map_err(|_| anyhow::anyhow!("ACP thread disconnected"))?;
        rx.await
            .map_err(|_| anyhow::anyhow!("ACP thread response failed"))?
    }

    async fn send_prompt(&self, session_id: String, prompt: Vec<acp::ContentBlock>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(AcpCommand::SendPrompt {
                session_id,
                prompt,
                respond_to: tx,
            })
            .map_err(|_| anyhow::anyhow!("ACP thread disconnected"))?;
        rx.await
            .map_err(|_| anyhow::anyhow!("ACP thread response failed"))?
    }
}

// Main function for the ACP thread that runs in a single-threaded runtime with LocalSet
async fn acp_thread_main(
    agent_name: String,
    client: RatClient,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    mut command_rx: mpsc::UnboundedReceiver<AcpCommand>,
) {
    info!("ACP thread main starting for agent: {}", agent_name);

    // Convert tokio streams to compatibility layer for ACP
    let stdin_compat = stdin.compat_write();
    let stdout_compat = stdout.compat();

    // Create ACP connection using LocalSet (which requires single-threaded runtime)
    let (mut connection, io_task) =
        acp::ClientSideConnection::new(client, stdin_compat, stdout_compat, |fut| {
            tokio::task::spawn_local(fut);
        });

    info!("Successfully established ACP connection for {}", agent_name);

    // Start the IO task
    tokio::task::spawn_local(async move {
        if let Err(e) = io_task.await {
            error!("ACP IO task failed: {}", e);
        }
    });

    // Initialize the ACP connection with proper protocol version
    info!("Initializing ACP connection with protocol version");
    match connection
        .initialize(acp::InitializeRequest {
            protocol_version: acp::V1,
            client_capabilities: acp::ClientCapabilities {
                fs: acp::FileSystemCapability {
                    read_text_file: true,
                    write_text_file: true,
                },
                terminal: false,
            },
        })
        .await
    {
        Ok(response) => {
            info!(
                "ACP initialization successful, protocol version: {:?}",
                response.protocol_version
            );
        }
        Err(e) => {
            error!("ACP initialization failed: {}", e);
            return;
        }
    }

    let mut sessions: HashMap<String, acp::SessionId> = HashMap::new();

    // Process commands from main thread
    while let Some(command) = command_rx.recv().await {
        match command {
            AcpCommand::CreateSession { respond_to } => {
                info!("Creating new ACP session");
                match connection
                    .new_session(acp::NewSessionRequest {
                        cwd: std::env::current_dir().unwrap_or_else(|_| "/tmp".into()),
                        mcp_servers: vec![],
                    })
                    .await
                {
                    Ok(response) => {
                        let session_id_str = response.session_id.0.to_string();
                        sessions.insert(session_id_str.clone(), response.session_id);
                        info!("Created ACP session: {}", session_id_str);
                        let _ = respond_to.send(Ok(session_id_str));
                    }
                    Err(e) => {
                        error!("Failed to create ACP session: {}", e);
                        let _ = respond_to
                            .send(Err(anyhow::anyhow!("Failed to create session: {}", e)));
                    }
                }
            }
            AcpCommand::SendPrompt {
                session_id,
                prompt,
                respond_to,
            } => {
                info!("Sending prompt to session: {}", session_id);
                if let Some(acp_session_id) = sessions.get(&session_id) {
                    match connection
                        .prompt(acp::PromptRequest {
                            session_id: acp_session_id.clone(),
                            prompt: prompt,
                        })
                        .await
                    {
                        Ok(_response) => {
                            debug!("Successfully sent prompt to session {}", session_id);
                            let _ = respond_to.send(Ok(()));
                        }
                        Err(e) => {
                            error!("Failed to send prompt to session {}: {}", session_id, e);
                            let _ = respond_to
                                .send(Err(anyhow::anyhow!("Failed to send prompt: {}", e)));
                        }
                    }
                } else {
                    error!("Session not found: {}", session_id);
                    let _ =
                        respond_to.send(Err(anyhow::anyhow!("Session not found: {}", session_id)));
                }
            }
        }
    }

    info!("ACP thread main exiting for agent: {}", agent_name);
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
    command_args: Vec<String>,
    command_env: Option<HashMap<String, String>>,
    process: Option<Child>,
    connection: Option<RealAcpConnection>,
    acp_thread_handle: Option<thread::JoinHandle<()>>,
    sessions: HashMap<SessionId, Session>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
    client: RatClient,
}

impl AcpClient {
    pub fn new(
        agent_name: &str,
        command_path: &str,
        command_args: Vec<String>,
        command_env: Option<HashMap<String, String>>,
        message_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Self {
        let client = RatClient::new(agent_name.to_string(), message_tx.clone());

        Self {
            agent_name: agent_name.to_string(),
            command_path: command_path.to_string(),
            command_args,
            command_env,
            process: None,
            connection: None,
            acp_thread_handle: None,
            sessions: HashMap::new(),
            message_tx,
            client,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting ACP agent: {}", self.agent_name);

        // Start the agent process
        let mut cmd = Command::new(&self.command_path);
        if !self.command_args.is_empty() {
            cmd.args(&self.command_args);
        }
        if let Some(env) = &self.command_env {
            cmd.envs(env);
        }
        let mut child = cmd
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
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stderr handle"))?;

        // Create channel for communication with ACP thread
        let (command_tx, command_rx) = mpsc::unbounded_channel::<AcpCommand>();

        // Clone the client for the ACP thread
        let client_clone = self.client.clone();
        let agent_name = self.agent_name.clone();

        // Drain agent stderr in the background to prevent pipe backpressure deadlocks
        {
            let agent_name = self.agent_name.clone();
            tokio::task::spawn_local(async move {
                use tokio::io::AsyncBufReadExt;
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break,
                        Ok(_) => {
                            let trimmed = line.trim_end();
                            if !trimmed.is_empty() {
                                warn!("[{} stderr] {}", agent_name, trimmed);
                            }
                        }
                        Err(e) => {
                            warn!("Error reading [{}] stderr: {}", agent_name, e);
                            break;
                        }
                    }
                }
            });
        }

        // Spawn ACP thread with single-threaded runtime
        let acp_handle = thread::spawn(move || {
            info!("Starting ACP thread with single-threaded runtime");

            // Create single-threaded runtime for LocalSet compatibility
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create single-threaded runtime");

            rt.block_on(async {
                let local = tokio::task::LocalSet::new();
                local
                    .run_until(acp_thread_main(
                        agent_name,
                        client_clone,
                        stdin,
                        stdout,
                        command_rx,
                    ))
                    .await
            });

            info!("ACP thread exiting");
        });

        // Create connection wrapper
        let connection = RealAcpConnection { command_tx };

        self.process = Some(child);
        self.connection = Some(connection);
        self.acp_thread_handle = Some(acp_handle);

        info!("Real ACP connection established with threaded runtime");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping ACP agent: {}", self.agent_name);

        // Close connection first (this will drop command_tx and terminate ACP thread)
        self.connection = None;

        // Wait for ACP thread to finish
        if let Some(handle) = self.acp_thread_handle.take() {
            info!("Waiting for ACP thread to finish");
            if let Err(e) = handle.join() {
                warn!("ACP thread panicked: {:?}", e);
            }
        }

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
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        // Create session via ACP thread
        let acp_session_id = connection.create_session().await?;
        let session_id = SessionId(acp_session_id);
        let session = Session::new(session_id.clone());
        self.sessions.insert(session_id.clone(), session);

        info!("Created new ACP session: {}", session_id.0);
        Ok(session_id)
    }

    pub async fn send_prompt(
        &self,
        session_id: &SessionId,
        prompt: Vec<acp::ContentBlock>,
    ) -> Result<()> {
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Client not connected"))?;

        debug!("Sending prompt to session {}", session_id.0);

        // Send prompt via ACP thread
        connection.send_prompt(session_id.0.clone(), prompt).await?;

        info!("Successfully sent prompt to session {}", session_id.0);
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
