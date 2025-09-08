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
use tokio::sync::mpsc;

use crate::acp::{AcpClient, Message, SessionId};
use crate::adapters::AgentManager;
use crate::config::Config;
use crate::ui::TuiManager;

pub struct App {
    config: Config,
    agent_manager: AgentManager,
    tui_manager: TuiManager,
    should_quit: bool,
    last_tick: Instant,
    message_rx: Option<mpsc::UnboundedReceiver<AppMessage>>,
    message_tx: mpsc::UnboundedSender<AppMessage>,
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

        let agent_manager = AgentManager::new(config.agents.clone(), message_tx.clone()).await?;
        let tui_manager = TuiManager::new(config.ui.clone())?;

        Ok(Self {
            config,
            agent_manager,
            tui_manager,
            should_quit: false,
            last_tick: Instant::now(),
            message_rx: Some(message_rx),
            message_tx,
        })
    }

    pub async fn connect_agent(&mut self, agent_name: &str) -> Result<()> {
        info!("Connecting to agent: {}", agent_name);

        if !self.config.agents.is_agent_enabled(agent_name) {
            return Err(anyhow::anyhow!("Agent '{}' is not enabled", agent_name));
        }

        self.agent_manager.connect_agent(agent_name).await?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting RAT application");

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

        // Auto-connect configured agents
        let auto_connect_agents = self.config.agents.auto_connect.clone();
        for agent_name in auto_connect_agents {
            if let Err(e) = self.connect_agent(&agent_name).await {
                warn!("Failed to auto-connect agent '{}': {}", agent_name, e);
                let _ = self.message_tx.send(AppMessage::Error {
                    error: format!("Failed to connect to {}: {}", agent_name, e),
                });
            }
        }

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

            // Tick
            if last_tick.elapsed() >= tick_rate {
                self.tick().await?;
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
        // Update agent manager
        self.agent_manager.tick().await?;

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
        self.agent_manager.disconnect_all().await?;

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

    pub fn get_agent_names(&self) -> Vec<String> {
        self.agent_manager.get_agent_names()
    }

    pub fn get_active_sessions(&self) -> HashMap<String, Vec<SessionId>> {
        self.agent_manager.get_active_sessions()
    }

    pub async fn create_session(&mut self, agent_name: &str) -> Result<SessionId> {
        self.agent_manager.create_session(agent_name).await
    }

    pub async fn send_message(
        &mut self,
        agent_name: &str,
        session_id: &SessionId,
        content: String,
    ) -> Result<()> {
        self.agent_manager
            .send_message(agent_name, session_id, content)
            .await
    }
}
