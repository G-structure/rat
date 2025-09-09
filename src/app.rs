use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use log::{debug, error, info, warn};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use std::time::Duration as StdDuration;

use crate::acp::{AcpClient, Message, SessionId};
use crate::adapters::AgentManager;
use crate::config::Config;
use crate::ui::TuiManager;

// Messages sent from UI layer to App layer
pub enum UiToApp {
    CreateSession {
        agent_name: String,
        respond_to: oneshot::Sender<anyhow::Result<SessionId>>,
    },
    ConnectAgent {
        agent_name: String,
    },
}

pub struct App {
    config: Config,
    tui_manager: TuiManager,
    should_quit: bool,
    last_tick: Instant,
    message_rx: Option<mpsc::UnboundedReceiver<AppMessage>>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
    ui_cmd_rx: Option<mpsc::UnboundedReceiver<UiToApp>>,
    ui_cmd_tx: mpsc::UnboundedSender<UiToApp>,
    manager_tx: mpsc::UnboundedSender<ManagerCmd>,
    manager_rx: Option<mpsc::UnboundedReceiver<ManagerCmd>>,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    AgentMessage {
        agent_name: String,
        message: Message,
    },
    AgentConnected {
        agent_name: String,
    },
    AgentDisconnected {
        agent_name: String,
    },
    SessionCreated {
        agent_name: String,
        session_id: SessionId,
    },
    Error {
        error: String,
    },
    Quit,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing application");

        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (ui_cmd_tx, ui_cmd_rx) = mpsc::unbounded_channel();
        let (manager_tx, manager_rx) = mpsc::unbounded_channel();

        let tui_manager = TuiManager::new(config.ui.clone(), ui_cmd_tx.clone())?;

        Ok(Self {
            config,
            tui_manager,
            should_quit: false,
            last_tick: Instant::now(),
            message_rx: Some(message_rx),
            message_tx,
            ui_cmd_rx: Some(ui_cmd_rx),
            ui_cmd_tx,
            manager_tx,
            manager_rx: Some(manager_rx),
        })
    }

    pub async fn connect_agent(&mut self, agent_name: &str) -> Result<()> {
        info!("Connecting to agent: {}", agent_name);

        if !self.config.agents.is_agent_enabled(agent_name) {
            return Err(anyhow::anyhow!("Agent '{}' is not enabled", agent_name));
        }
        let _ = self
            .manager_tx
            .send(ManagerCmd::ConnectAgent { agent_name: agent_name.to_string() });

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting RAT application");
        // Run the main loop inside a LocalSet so we can use spawn_local for non-Send tasks
        let local = tokio::task::LocalSet::new();
        local.run_until(self.run_inner()).await
    }

    async fn run_inner(&mut self) -> Result<()> {
        
        // Setup terminal
        crossterm::terminal::enable_raw_mode().context("Failed to enable raw mode")?;
        crossterm::execute!(
            io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        // Take the message receiver
        let mut message_rx = self
            .message_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("Message receiver already taken"))?;

        // Take the UI command receiver
        let mut ui_cmd_rx = self
            .ui_cmd_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("UI command receiver already taken"))?;

        // Start the manager worker now that we're inside LocalSet
        let manager_rx = self
            .manager_rx
            .take()
            .ok_or_else(|| anyhow::anyhow!("Manager receiver already taken"))?;
        let message_tx = self.message_tx.clone();
        let agent_config = self.config.agents.clone();
        tokio::task::spawn_local(async move {
            match AgentManager::new(agent_config, message_tx.clone()).await {
                Ok(manager) => manager_worker(manager, manager_rx).await,
                Err(e) => {
                    let _ = message_tx.send(AppMessage::Error {
                        error: format!("Failed to start AgentManager: {}", e),
                    });
                }
            }
        });

        // Auto-connect configured agents via manager worker (non-blocking)
        for agent_name in self.config.agents.auto_connect.clone() {
            let _ = self.manager_tx.send(ManagerCmd::ConnectAgent { agent_name });
        }

        // Manager worker handles its own periodic tick.

        // Main event loop
        let tick_rate = Duration::from_millis(50); // 20 FPS
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // Handle events - simplified approach
            if crossterm::event::poll(timeout)? {
                if let Ok(event) = crossterm::event::read() {
                    match event {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            info!("Raw key event detected: {:?}", key);
                            if self.handle_key_event(key).await? {
                                break;
                            }
                        }
                        _ => {
                            // Ignore other events for now
                        }
                    }
                }
            }

            // Handle application messages
            while let Ok(msg) = message_rx.try_recv() {
                self.handle_app_message(msg).await?;
            }

            // Handle at most one UI -> App command per frame to keep UI responsive
            if let Ok(cmd) = ui_cmd_rx.try_recv() {
                match cmd {
                    UiToApp::CreateSession {
                        agent_name,
                        respond_to,
                    } => {
                        // Offload session creation to background to avoid blocking UI loop
                        // Forward to manager worker which will respond via oneshot
                        let _ = self.manager_tx.send(ManagerCmd::CreateSession {
                            agent_name,
                            respond_to,
                        });
                    }
                    UiToApp::ConnectAgent { agent_name } => {
                        let _ = self.manager_tx.send(ManagerCmd::ConnectAgent { agent_name });
                    }
                }
            }

            // Tick
            if last_tick.elapsed() >= tick_rate {
                // Only tick the TUI here; agent manager ticks in its own background task
                self.tui_manager.tick().await?;
                last_tick = Instant::now();
            }

            // Render
            terminal.draw(|f| {
                if let Err(e) = self.render(f) {
                    error!("Render error: {}", e);
                }
            })?;

            if self.should_quit {
                break;
            }
        }

        // Cleanup
        self.cleanup().await?;

        // Restore terminal
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;

        info!("RAT application terminated");
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        info!("Key event received: {:?}", key);

        // Global keybindings
        if let Some(quit_key) = self.config.ui.get_keybinding("quit") {
            info!("Quit key configured as: {}", quit_key);
            if key.code == KeyCode::Char(quit_key.chars().next().unwrap_or('q')) {
                info!("Quit key pressed! Setting should_quit = true");
                self.should_quit = true;
                return Ok(true);
            }
        }

        // Handle Ctrl+C
        if key.code == KeyCode::Char('c')
            && key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.should_quit = true;
            return Ok(true);
        }

        // Pass to TUI manager for specific handling
        self.tui_manager.handle_key_event(key).await?;

        Ok(false)
    }

    async fn handle_app_message(&mut self, message: AppMessage) -> Result<()> {
        debug!("App message: {:?}", message);

        match message {
            AppMessage::AgentMessage {
                agent_name,
                message,
            } => {
                self.tui_manager.add_message(&agent_name, message).await?;
            }
            AppMessage::AgentConnected { agent_name } => {
                info!("Agent connected: {}", agent_name);
                self.tui_manager
                    .set_agent_status(&agent_name, "Connected".to_string());
            }
            AppMessage::AgentDisconnected { agent_name } => {
                warn!("Agent disconnected: {}", agent_name);
                self.tui_manager
                    .set_agent_status(&agent_name, "Disconnected".to_string());
            }
            AppMessage::SessionCreated {
                agent_name,
                session_id,
            } => {
                info!("Session created for {}: {}", agent_name, session_id.0);
                self.tui_manager.add_session(&agent_name, session_id)?;
            }
            AppMessage::Error { error } => {
                error!("Application error: {}", error);
                self.tui_manager.show_error(error);
            }
            AppMessage::Quit => {
                self.should_quit = true;
            }
        }

        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Update TUI manager
        self.tui_manager.tick().await?;

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) -> Result<()> {
        self.tui_manager.render(frame)
    }

    async fn cleanup(&mut self) -> Result<()> {
        info!("Cleaning up application");

        // Disconnect all agents
        let (tx, rx) = oneshot::channel();
        let _ = self.manager_tx.send(ManagerCmd::DisconnectAll { respond_to: tx });
        let _ = rx.await; // ignore errors during shutdown

        // Save configuration if auto-save is enabled
        if self.config.general.auto_save_sessions {
            if let Err(e) = self.save_state().await {
                warn!("Failed to save application state: {}", e);
            }
        }

        Ok(())
    }

    async fn save_state(&self) -> Result<()> {
        // Save sessions, preferences, etc.
        // Implementation would depend on persistence requirements
        info!("Application state saved");
        Ok(())
    }

    pub async fn create_session(&mut self, agent_name: &str) -> Result<SessionId> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .manager_tx
            .send(ManagerCmd::CreateSession {
                agent_name: agent_name.to_string(),
                respond_to: tx,
            });
        rx.await.unwrap_or_else(|_| Err(anyhow::anyhow!("manager task ended")))
    }

    pub async fn send_message(
        &mut self,
        agent_name: &str,
        session_id: &SessionId,
        content: String,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self.manager_tx.send(ManagerCmd::SendMessage {
            agent_name: agent_name.to_string(),
            session_id: session_id.clone(),
            content,
            respond_to: tx,
        });
        rx.await.unwrap_or_else(|_| Err(anyhow::anyhow!("manager task ended")))
    }
}

// Commands to the single-threaded manager worker
pub enum ManagerCmd {
    ConnectAgent { agent_name: String },
    CreateSession {
        agent_name: String,
        respond_to: oneshot::Sender<anyhow::Result<SessionId>>,
    },
    SendMessage {
        agent_name: String,
        session_id: SessionId,
        content: String,
        respond_to: oneshot::Sender<anyhow::Result<()>>,
    },
    DisconnectAll {
        respond_to: oneshot::Sender<()>,
    },
}

pub async fn manager_worker(
    mut manager: AgentManager,
    mut rx: mpsc::UnboundedReceiver<ManagerCmd>,
) {
    let mut interval = tokio::time::interval(StdDuration::from_millis(50));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = manager.tick().await {
                    warn!("Agent manager tick error: {}", e);
                }
            }
            cmd = rx.recv() => {
                match cmd {
                    Some(ManagerCmd::ConnectAgent { agent_name }) => {
                        if let Err(e) = manager.connect_agent(&agent_name).await {
                            warn!("Failed to connect agent '{}': {}", agent_name, e);
                        }
                    }
                    Some(ManagerCmd::CreateSession { agent_name, respond_to }) => {
                        let _ = respond_to.send(manager.create_session(&agent_name).await);
                    }
                    Some(ManagerCmd::SendMessage { agent_name, session_id, content, respond_to }) => {
                        let _ = respond_to.send(manager.send_message(&agent_name, &session_id, content).await);
                    }
                    Some(ManagerCmd::DisconnectAll { respond_to }) => {
                        let _ = manager.disconnect_all().await;
                        let _ = respond_to.send(());
                    }
                    None => break,
                }
            }
        }
    }
}
