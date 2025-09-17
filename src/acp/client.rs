use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::thread;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::{Message, Session, SessionId};
use crate::app::AppMessage;
use agent_client_protocol::{self as acp, Agent};
use which::which;

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

#[derive(Clone, Debug)]
pub struct LoginCommand {
    pub path: PathBuf,
    pub args: Vec<String>,
}

// Best-effort interactive login runner.
async fn run_login_if_needed(login_cmd: &LoginCommand) -> Result<()> {
    let use_script = which("script").is_ok();
    let mut cmd = if use_script {
        let mut c = Command::new("script");
        c.arg("-q").arg("/dev/null").arg(&login_cmd.path);
        for a in &login_cmd.args {
            c.arg(a);
        }
        c
    } else {
        let mut c = Command::new(&login_cmd.path);
        c.args(&login_cmd.args);
        c
    };

    let mut child = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn login command")?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("no stdout from login process"))?;
    let stderr = child.stderr.take();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let tx_stdout = tx.clone();
    tokio::task::spawn_local(async move {
        let mut rdr = BufReader::new(stdout);
        let mut buf = String::new();
        loop {
            buf.clear();
            match rdr.read_line(&mut buf).await {
                Ok(0) => break,
                Ok(_) => {
                    let _ = tx_stdout.send(buf.clone());
                }
                Err(_) => break,
            }
        }
    });
    if let Some(stderr) = stderr {
        let tx2 = tx.clone();
        tokio::task::spawn_local(async move {
            let mut rdr = BufReader::new(stderr);
            let mut buf = String::new();
            loop {
                buf.clear();
                match rdr.read_line(&mut buf).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = tx2.send(buf.clone());
                    }
                    Err(_) => break,
                }
            }
        });
    }

    let success_markers = [
        "Login successful",
        "Already logged in",
        "You are logged in",
        "Logged in as",
        "Successfully logged in",
    ];

    let start = std::time::Instant::now();
    let max = tokio::time::Duration::from_secs(180);
    loop {
        if let Some(status) = child.try_wait().context("login try_wait failed")? {
            if status.success() {
                info!("Login process exited successfully");
                break;
            } else {
                return Err(anyhow!("login process failed: {}", status));
            }
        }

        match tokio::time::timeout(tokio::time::Duration::from_millis(1000), rx.recv()).await {
            Ok(Some(line)) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    info!("login: {}", trimmed);
                    if success_markers.iter().any(|m| trimmed.contains(m)) {
                        info!("Detected successful login marker");
                        break;
                    }
                }
            }
            Ok(None) => {}
            Err(_) => {
                if start.elapsed() > max {
                    let _ = child.kill().await;
                    return Err(anyhow!("login timeout"));
                }
            }
        }
    }

    let _ = child.kill().await;
    Ok(())
}

// Main function for the ACP thread that runs in a single-threaded runtime with LocalSet
async fn acp_thread_main(
    agent_name: String,
    client: RatClient,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    mut command_rx: mpsc::UnboundedReceiver<AcpCommand>,
    login_cmd: Option<LoginCommand>,
    app_tx: mpsc::UnboundedSender<crate::app::AppMessage>,
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
            if !response.auth_methods.is_empty() {
                let list: Vec<String> = response
                    .auth_methods
                    .iter()
                    .map(|m| m.id.0.to_string())
                    .collect();
                info!("Agent advertised auth methods: {:?}", list);

                // Do NOT call ACP authenticate here. For agents like claude-code-acp,
                // the advertised method (e.g., "claude-login") is not handled by the
                // authenticate RPC and must be satisfied via an external CLI login.
                // Instead, defer login to when an operation returns AUTH_REQUIRED and
                // then run the external login flow and retry (see session creation below).
            }
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
                let request_new_session = || async {
                    connection
                        .new_session(acp::NewSessionRequest {
                            cwd: std::env::current_dir().unwrap_or_else(|_| "/tmp".into()),
                            mcp_servers: vec![],
                        })
                        .await
                };

                match request_new_session().await {
                    Ok(response) => {
                        let session_id_str = response.session_id.0.to_string();
                        sessions.insert(session_id_str.clone(), response.session_id);
                        info!("Created ACP session: {}", session_id_str);
                        let _ = respond_to.send(Ok(session_id_str));
                    }
                    Err(e) => {
                        // If authentication required, try to run external login and retry once
                        let auth_required = e.code == acp::ErrorCode::AUTH_REQUIRED.code
                            || e.message.contains("Authentication required")
                            || e.message.contains("Please run /login");
                        if auth_required {
                            warn!("Session creation requires authentication; attempting external login...");
                            if let Some(cmd) = &login_cmd {
                                let _ = app_tx.send(crate::app::AppMessage::SuspendTui);
                                if let Err(le) = run_login_if_needed(cmd).await {
                                    warn!("Login flow failed: {}", le);
                                } else {
                                    // Retry once
                                    match request_new_session().await {
                                        Ok(response) => {
                                            let session_id_str = response.session_id.0.to_string();
                                            sessions.insert(session_id_str.clone(), response.session_id);
                                            info!("Created ACP session after login: {}", session_id_str);
                                            let _ = respond_to.send(Ok(session_id_str));
                                            continue;
                                        }
                                        Err(e2) => {
                                            error!("Failed to create session after login: {}", e2);
                                            let _ = respond_to
                                                .send(Err(anyhow::anyhow!("Failed to create session after login: {}", e2)));
                                            continue;
                                        }
                                    }
                                }
                                let _ = app_tx.send(crate::app::AppMessage::ResumeTui);
                            } else {
                                warn!("No login command configured; cannot authenticate");
                            }
                        }

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
    login_command: Option<LoginCommand>,
    sessions: HashMap<SessionId, Session>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
    client: RatClient,
}

impl AcpClient {
    // Build extra CLI args for Claude Code to ensure file edit/tools are enabled.
    // Defaults allow both ACP-bridged FS tools and Claude's built-in Edit/MultiEdit.
    // Users can override via env:
    // - RAT_PERMISSION_PROMPT_TOOL
    // - RAT_ALLOWED_TOOLS (comma-separated)
    // - RAT_DISALLOWED_TOOLS (comma-separated; empty to omit flag)
    fn build_claude_tool_args() -> Vec<String> {
        let mut args = Vec::new();

        let permission_tool = std::env::var("RAT_PERMISSION_PROMPT_TOOL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "mcp__acp__permission".to_string());
        args.push("--permission-prompt-tool".to_string());
        args.push(permission_tool);

        let allowed_default = "mcp__acp__read,mcp__acp__write,Read,Write,Edit,MultiEdit";
        let allowed = std::env::var("RAT_ALLOWED_TOOLS")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| allowed_default.to_string());
        args.push("--allowedTools".to_string());
        args.push(allowed);

        if let Ok(disallowed) = std::env::var("RAT_DISALLOWED_TOOLS") {
            if !disallowed.trim().is_empty() {
                args.push("--disallowedTools".to_string());
                args.push(disallowed);
            }
        }

        args
    }
    pub fn new(
        agent_name: &str,
        command_path: &str,
        command_args: Vec<String>,
        command_env: Option<HashMap<String, String>>,
        message_tx: mpsc::UnboundedSender<AppMessage>,
        login_command: Option<LoginCommand>,
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
            login_command,
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
        // For Claude Code specifically, append args that enable file edits and tool usage
        if self.agent_name == "claude-code" {
            cmd.args(Self::build_claude_tool_args());
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
            tokio::spawn(async move {
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
        let login_cmd = self.login_command.clone();
        let app_tx = self.message_tx.clone();
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
                        login_cmd,
                        app_tx,
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

#[cfg(test)]
mod tests {
    use super::AcpClient;

    fn with_env<T: FnOnce() -> ()>(kvs: &[(&str, &str)], f: T) {
        // Save old
        let saved: Vec<(String, Option<String>)> = kvs
            .iter()
            .map(|(k, _)| (k.to_string(), std::env::var(k).ok()))
            .collect();
        // Set new
        for (k, v) in kvs.iter() {
            std::env::set_var(k, v);
        }
        f();
        // Restore
        for (k, v) in saved.into_iter() {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }

    #[test]
    fn claude_tool_args_default_and_overrides() {
        // Default case (no env): should include permission tool and allowed list with Edit/MultiEdit
        std::env::remove_var("RAT_PERMISSION_PROMPT_TOOL");
        std::env::remove_var("RAT_ALLOWED_TOOLS");
        std::env::remove_var("RAT_DISALLOWED_TOOLS");
        let args = AcpClient::build_claude_tool_args();
        let joined = args.join(" ");
        assert!(joined.contains("--permission-prompt-tool mcp__acp__permission"));
        assert!(joined.contains("--allowedTools"));
        assert!(joined.contains("Edit"));
        assert!(joined.contains("MultiEdit"));
        assert!(!joined.contains("--disallowedTools"));

        // Override all via env
        with_env(
            &[
                ("RAT_PERMISSION_PROMPT_TOOL", "custom_perm"),
                ("RAT_ALLOWED_TOOLS", "Foo,Bar"),
                ("RAT_DISALLOWED_TOOLS", "Baz"),
            ],
            || {
                let args = AcpClient::build_claude_tool_args();
                let joined = args.join(" ");
                assert!(joined.contains("--permission-prompt-tool custom_perm"));
                assert!(joined.contains("--allowedTools Foo,Bar"));
                assert!(joined.contains("--disallowedTools Baz"));
            },
        );
    }
}
