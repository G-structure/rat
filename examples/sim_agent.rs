use anyhow::Result;
use agent_client_protocol::{self as acp, Client};
use clap::{ArgAction, Parser, ValueEnum};
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

#[derive(Clone, Debug, ValueEnum)]
enum Scenario {
    HappyPathEdit,
    FailurePath,
    ImagesAndThoughts,
    CommandsUpdate,
}

#[derive(Parser, Debug)]
#[command(name = "sim_agent", about = "Deterministic ACP simulator over stdio")]
struct Cli {
    #[arg(long, value_enum, default_value_t = Scenario::HappyPathEdit)]
    scenario: Scenario,

    #[arg(long, default_value = "fast")]
    speed: String,

    #[arg(long, default_value_t = 0)]
    seed: u64,

    #[arg(long, action = ArgAction::SetFalse)]
    loop_run: bool,
}

struct SimAgent {
    session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
    next_session_id: Cell<u64>,
    scenario: Scenario,
    speed_mult: f32,
    cancelling: Cell<bool>,
}

impl SimAgent {
    fn new(
        tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
        scenario: Scenario,
        speed_mult: f32,
    ) -> Self {
        Self {
            session_update_tx: tx,
            next_session_id: Cell::new(1),
            scenario,
            speed_mult,
            cancelling: Cell::new(false),
        }
    }

    fn parse_speed(s: &str) -> f32 {
        match s {
            "slomo" => 0.25,
            "normal" => 1.0,
            "fast" => 2.0,
            "max" => 100.0,
            _ => s.parse::<f32>().unwrap_or(1.0),
        }
    }

    async fn sleep_scaled(&self, ms: u64) {
        let scaled = if self.speed_mult <= 0.0 { ms } else { ((ms as f32) / self.speed_mult) as u64 };
        if scaled == 0 {
            tokio::task::yield_now().await;
        } else {
            tokio::time::sleep(Duration::from_millis(scaled)).await;
        }
    }

    async fn send_update(&self, session_id: &acp::SessionId, update: acp::SessionUpdate) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.session_update_tx
            .send((acp::SessionNotification { session_id: session_id.clone(), update }, tx))?;
        // Wait until the IO task forwards the notification to the client
        rx.await.map_err(|_| anyhow::anyhow!("session notification forwarding dropped"))?;
        Ok(())
    }

    async fn run_happy_path(&self, sid: &acp::SessionId) -> Result<()> {
        // Plan
        self.send_update(
            sid,
            acp::SessionUpdate::Plan(acp::Plan {
                entries: vec![
                    acp::PlanEntry { content: "Open file src/lib.rs".into(), priority: acp::PlanEntryPriority::Medium, status: acp::PlanEntryStatus::InProgress },
                    acp::PlanEntry { content: "Apply small refactor".into(), priority: acp::PlanEntryPriority::Low, status: acp::PlanEntryStatus::Pending },
                ],
            }),
        ).await?;
        self.sleep_scaled(120).await;

        // Tool call with diff - edit existing large file
        let tool_id = acp::ToolCallId(Arc::from("call_edit_1"));
        self.send_update(
            sid,
            acp::SessionUpdate::ToolCall(acp::ToolCall {
                id: tool_id.clone(),
                title: "Refactor main.rs error handling and add logging".into(),
                kind: acp::ToolKind::Edit,
                status: acp::ToolCallStatus::InProgress,
                content: vec![acp::ToolCallContent::Diff { diff: acp::Diff {
                    path: PathBuf::from("/workspace/src/main.rs"),
                    old_text: Some("use std::io;\nuse std::process;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    println!(\"Starting RAT application...\");\n\n    // Initialize configuration\n    let config = load_config()?;\n\n    // Start the main loop\n    run_app(config)?;\n\n    Ok(())\n}\n\nfn load_config() -> Result<Config, Box<dyn std::error::Error>> {\n    // Simple config loading\n    Ok(Config::default())\n}\n\nfn run_app(config: Config) -> Result<(), Box<dyn std::error::Error>> {\n    println!(\"Running with config: {:?}\", config);\n\n    // Main application logic here\n    println!(\"Application completed successfully\");\n\n    Ok(())\n}\n\n#[derive(Debug)]\nstruct Config {\n    debug: bool,\n    port: u16,\n}".into()),
                    new_text: "use std::io;\nuse std::process;\nuse log::{info, warn, error};\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n    // Initialize logging\n    env_logger::init();\n\n    info!(\"Starting RAT application...\");\n\n    // Initialize configuration with error handling\n    let config = match load_config() {\n        Ok(config) => {\n            info!(\"Configuration loaded successfully\");\n            config\n        },\n        Err(e) => {\n            error!(\"Failed to load configuration: {}\", e);\n            return Err(e.into());\n        }\n    };\n\n    // Start the main loop with proper error handling\n    if let Err(e) = run_app(config) {\n        error!(\"Application failed: {}\", e);\n        process::exit(1);\n    }\n\n    info!(\"Application completed successfully\");\n    Ok(())\n}\n\nfn load_config() -> Result<Config, io::Error> {\n    // Enhanced config loading with validation\n    let config = Config::default();\n\n    // Validate configuration\n    if config.port == 0 {\n        return Err(io::Error::new(io::ErrorKind::InvalidInput, \"Invalid port number\"));\n    }\n\n    Ok(config)\n}\n\nfn run_app(config: Config) -> Result<(), Box<dyn std::error::Error>> {\n    info!(\"Running with config: {:?}\", config);\n\n    // Enhanced application logic with logging\n    if config.debug {\n        warn!(\"Debug mode is enabled - this may impact performance\");\n    }\n\n    // Simulate some work\n    info!(\"Processing application logic...\");\n\n    // Check for any issues\n    if config.port < 1024 {\n        warn!(\"Using privileged port {}, consider using port > 1024\", config.port);\n    }\n\n    info!(\"Application logic completed successfully\");\n    Ok(())\n}\n\n#[derive(Debug)]\nstruct Config {\n    debug: bool,\n    port: u16,\n}\n\nimpl Default for Config {\n    fn default() -> Self {\n        Self {\n            debug: false,\n            port: 8080,\n        }\n    }\n}".into(),
                }}],
                locations: vec![acp::ToolCallLocation { path: PathBuf::from("/workspace/src/main.rs"), line: Some(1) }],
                raw_input: None,
                raw_output: None,
            }),
        ).await?;
        self.sleep_scaled(60).await;

        // Complete tool call (skipping permission request for now)
        self.send_update(
            sid,
            acp::SessionUpdate::ToolCallUpdate(acp::ToolCallUpdate {
                id: tool_id,
                fields: acp::ToolCallUpdateFields { status: Some(acp::ToolCallStatus::Completed), ..Default::default() },
            }),
        ).await?;
        self.sleep_scaled(80).await;

        // Agent message chunks
        self.send_update(
            sid,
            acp::SessionUpdate::AgentMessageChunk { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Applied the change to src/lib.rs. ".into() }) },
        ).await?;
        self.sleep_scaled(40).await;
        self.send_update(
            sid,
            acp::SessionUpdate::AgentMessageChunk { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Anything else?".into() }) },
        ).await?;

        Ok(())
    }

    async fn run_failure_path(&self, sid: &acp::SessionId) -> Result<()> {
        let tool_id = acp::ToolCallId(Arc::from("call_fail_1"));
        self.send_update(
            sid,
            acp::SessionUpdate::ToolCall(acp::ToolCall {
                id: tool_id.clone(),
                title: "Search project".into(),
                kind: acp::ToolKind::Search,
                status: acp::ToolCallStatus::InProgress,
                content: vec![acp::ToolCallContent::Content { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Searching...".into() }) }],
                locations: vec![],
                raw_input: None,
                raw_output: None,
            }),
        ).await?;
        self.sleep_scaled(60).await;
        self.send_update(
            sid,
            acp::SessionUpdate::ToolCallUpdate(acp::ToolCallUpdate {
                id: tool_id,
                fields: acp::ToolCallUpdateFields {
                    status: Some(acp::ToolCallStatus::Failed),
                    content: Some(vec![acp::ToolCallContent::Content { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "No matches found".into() }) }]),
                    ..Default::default()
                },
            }),
        ).await?;
        self.sleep_scaled(40).await;
        self.send_update(
            sid,
            acp::SessionUpdate::AgentMessageChunk { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Search failed; please refine your query.".into() }) },
        ).await?;
        Ok(())
    }

    async fn run_images_and_thoughts(&self, sid: &acp::SessionId) -> Result<()> {
        self.send_update(
            sid,
            acp::SessionUpdate::AgentThoughtChunk { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Hmm, outlining approach…".into() }) },
        ).await?;
        self.sleep_scaled(50).await;

        let img_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAuMB9s8O8d8AAAAASUVORK5CYII=";
        self.send_update(
            sid,
            acp::SessionUpdate::AgentMessageChunk { content: acp::ContentBlock::Image(acp::ImageContent { annotations: None, data: img_data.into(), mime_type: "image/png".into(), uri: None }) },
        ).await?;
        self.sleep_scaled(40).await;
        self.send_update(
            sid,
            acp::SessionUpdate::AgentMessageChunk { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Here\u{2019}s a quick sketch.".into() }) },
        ).await?;
        Ok(())
    }

    async fn run_commands_update(&self, sid: &acp::SessionId) -> Result<()> {
        // The AvailableCommandsUpdate session update is behind the `unstable` feature in ACP.
        // For now, just send a standard message chunk to indicate change.
        self.sleep_scaled(20).await;
        self.send_update(
            sid,
            acp::SessionUpdate::AgentMessageChunk { content: acp::ContentBlock::Text(acp::TextContent { annotations: None, text: "Commands updated.".into() }) },
        ).await?;
        Ok(())
    }
}

impl acp::Agent for SimAgent {
    async fn initialize(&self, _arguments: acp::InitializeRequest) -> Result<acp::InitializeResponse, acp::Error> {
        Ok(acp::InitializeResponse {
            protocol_version: acp::V1,
            agent_capabilities: acp::AgentCapabilities {
                load_session: true,
                prompt_capabilities: acp::PromptCapabilities { image: true, audio: false, embedded_context: true },
                mcp_capabilities: acp::McpCapabilities::default(),
            },
            auth_methods: vec![],
        })
    }

    async fn authenticate(&self, _arguments: acp::AuthenticateRequest) -> Result<(), acp::Error> {
        Ok(())
    }

    async fn new_session(&self, _arguments: acp::NewSessionRequest) -> Result<acp::NewSessionResponse, acp::Error> {
        let id = self.next_session_id.get();
        self.next_session_id.set(id + 1);
        Ok(acp::NewSessionResponse { session_id: acp::SessionId(Arc::from(format!("sim-{id}"))), modes: None })
    }

    async fn load_session(&self, _arguments: acp::LoadSessionRequest) -> Result<acp::LoadSessionResponse, acp::Error> {
        // Minimal: do nothing
        Ok(acp::LoadSessionResponse { modes: None })
    }

    async fn prompt(&self, arguments: acp::PromptRequest) -> Result<acp::PromptResponse, acp::Error> {
        self.cancelling.set(false);
        match self.scenario {
            Scenario::HappyPathEdit => self.run_happy_path(&arguments.session_id).await.map_err(|_| acp::Error::internal_error())?,
            Scenario::FailurePath => self.run_failure_path(&arguments.session_id).await.map_err(|_| acp::Error::internal_error())?,
            Scenario::ImagesAndThoughts => self.run_images_and_thoughts(&arguments.session_id).await.map_err(|_| acp::Error::internal_error())?,
            Scenario::CommandsUpdate => self.run_commands_update(&arguments.session_id).await.map_err(|_| acp::Error::internal_error())?,
        }
        let stop_reason = if self.cancelling.get() { acp::StopReason::Cancelled } else { acp::StopReason::EndTurn };
        Ok(acp::PromptResponse { stop_reason })
    }

    async fn cancel(&self, _args: acp::CancelNotification) -> Result<(), acp::Error> {
        self.cancelling.set(true);
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let speed_mult = SimAgent::parse_speed(&cli.speed);

    let (tx, mut rx) = mpsc::unbounded_channel();

    let outgoing = tokio::io::stdout().compat_write();
    let incoming = tokio::io::stdin().compat();

    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async move {
            let agent = SimAgent::new(tx, cli.scenario, speed_mult);
            let (conn, handle_io) = acp::AgentSideConnection::new(agent, outgoing, incoming, |fut| {
                tokio::task::spawn_local(fut);
            });

            tokio::task::spawn_local(async move {
                while let Some((session_notification, tx)) = rx.recv().await {
                    let result = conn.session_notification(session_notification).await;
                    if let Err(e) = result {
                        log::error!("{e}");
                        break;
                    }
                    let _ = tx.send(());
                }
            });

            handle_io.await
        })
        .await
}
