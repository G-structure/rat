use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::info;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Tabs, BorderType},
};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use std::time::Instant;

use tachyonfx::{fx, Duration as FxDuration, EffectManager as FxManager, Interpolation};
use crate::effects::cyberpunk::{CyberTheme, neon_pulse_border, subtle_hsl_drift, sweep_in_attention, glitch_burst};
use crate::effects::startup::matrix_rain_morph_with_duration;
use tachyonfx::RefRect;
use tachyonfx::{ref_count, BufferRenderer};

use crate::acp::{Message, MessageContent, SessionId};
use crate::app::UiToApp;
use crate::config::UiConfig;
use crate::ui::{chat::ChatView, components::AgentSelector, statusbar::StatusBar};

pub struct TuiManager {
    config: UiConfig,
    active_tab: usize,
    tabs: Vec<Tab>,
    agent_selector: AgentSelector,
    status_bar: StatusBar,
    error_message: Option<String>,
    show_help: bool,
    ui_tx: mpsc::UnboundedSender<UiToApp>,
    theme: CyberTheme,
    fx: FxManager<&'static str>,
    last_fx_tick: Instant,
    ambient_fx_initialized: bool,
    // Startup animation state
    startup_effect: Option<tachyonfx::Effect>,
    startup_running: bool,
    startup_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub name: String,
    pub agent_name: String,
    pub session_id: Option<SessionId>,
    pub chat_view: ChatView,
    pub active: bool,
    pub chat_area_ref: RefRect,
}

impl TuiManager {
    pub fn new(config: UiConfig, ui_tx: mpsc::UnboundedSender<UiToApp>) -> Result<Self> {
        let startup_duration_ms = config.effects.startup.duration_ms;
        let startup_running = config.effects.enabled && config.effects.startup.enabled;
        Ok(Self {
            config,
            active_tab: 0,
            tabs: Vec::new(),
            agent_selector: AgentSelector::new(),
            status_bar: StatusBar::new(),
            error_message: None,
            show_help: false,
            ui_tx,
            theme: CyberTheme::default(),
            fx: FxManager::default(),
            last_fx_tick: Instant::now(),
            ambient_fx_initialized: false,
            startup_effect: None,
            startup_running,
            startup_duration_ms,
        })
    }

    pub fn render(&mut self, frame: &mut Frame) -> Result<()> {
        // Background
        let bg = Block::default().style(self.theme.background_style());
        frame.render_widget(bg, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Tab bar
                    Constraint::Min(1),    // Main content
                    Constraint::Length(1), // Status bar
                ]
                .as_ref(),
            )
            .split(frame.area());

        // Render tab bar if we have tabs
        if !self.tabs.is_empty() {
            self.render_tabs(frame, chunks[0]);

            // Render active tab content
            if let Some(active_tab) = self.tabs.get_mut(self.active_tab) {
                active_tab.chat_area_ref.set(chunks[1]);
                active_tab.chat_view.render(frame, chunks[1])?;
            }
        } else {
            // Show welcome screen
            self.render_welcome(frame, chunks[1]);
        }

        // Render status bar
        self.status_bar.render(frame, chunks[2])?;

        // Render error popup if present
        if let Some(ref error) = self.error_message {
            self.render_error_popup(frame, error);
        }

        // Render help if requested
        if self.show_help {
            self.render_help_popup(frame);
        }

        // Apply startup/ambient effects depending on config
        if self.config.effects.enabled {
            if self.startup_running {
                self.apply_startup_fx(frame);
            } else {
                // Ambient FX init happens in tick
                self.apply_fx(frame);
            }
        }

        Ok(())
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let tab_names: Vec<&str> = self.tabs.iter().map(|tab| tab.name.as_str()).collect();

        let tabs = Tabs::new(tab_names)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(self.theme.title_inactive())
            .highlight_style(self.theme.title_active())
            .select(self.active_tab);

        frame.render_widget(tabs, area);
    }

    fn render_welcome(&self, frame: &mut Frame, area: Rect) {
        let welcome_text = vec![
            Line::from("Welcome to RAT (Rust Agent Terminal)!"),
            Line::from(""),
            Line::from("Commands:"),
            Line::from("  n - Create new session with default agent"),
            Line::from("  a - Select agent"),
            Line::from("  ? - Show help"),
            Line::from("  q - Quit"),
            Line::from(""),
            Line::from("No active sessions. Press 'n' to start!"),
        ];

        let welcome = Paragraph::new(welcome_text)
            .block(Block::default().title("RAT").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(welcome, area);
    }

    fn render_error_popup(&self, frame: &mut Frame, error: &str) {
        let area = centered_rect(60, 20, frame.area());

        frame.render_widget(Clear, area);

        let error_text = vec![
            Line::from("Error"),
            Line::from(""),
            Line::from(error),
            Line::from(""),
            Line::from("Press any key to dismiss"),
        ];

        let popup = Paragraph::new(error_text)
            .block(
                Block::default()
                    .title("Error")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(self.theme.palette.accent_a)),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(popup, area);
    }

    fn render_help_popup(&self, frame: &mut Frame) {
        let area = centered_rect(80, 60, frame.area());

        frame.render_widget(Clear, area);

        let help_text = vec![
            Line::from("RAT - Rust Agent Terminal Help"),
            Line::from(""),
            Line::from("Global Commands:"),
            Line::from("  q       - Quit application"),
            Line::from("  ?       - Toggle this help"),
            Line::from("  Ctrl+C  - Force quit"),
            Line::from(""),
            Line::from("Session Management:"),
            Line::from("  n       - New session with default agent"),
            Line::from("  a       - Switch agent"),
            Line::from("  Tab     - Next tab"),
            Line::from("  Shift+Tab - Previous tab"),
            Line::from(""),
            Line::from("Chat:"),
            Line::from("  Enter   - Send message"),
            Line::from("  Esc     - Cancel input"),
            Line::from(""),
            Line::from("Edit Review:"),
            Line::from("  y       - Accept edit"),
            Line::from("  n       - Reject edit"),
            Line::from("  d       - Show diff"),
            Line::from(""),
            Line::from("Press any key to close help"),
        ];

        let popup = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(self.theme.palette.accent_b)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(popup, area);
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Intercept Enter to send a chat message bound to the active session
        if let KeyCode::Enter = key.code {
            if let Some(active_tab) = self.tabs.get_mut(self.active_tab) {
                if active_tab.chat_view.is_input_mode() {
                    let content = active_tab.chat_view.get_input_buffer().trim().to_string();
                    if !content.is_empty() {
                        if let Some(session_id) = active_tab.session_id.clone() {
                            // Create and add user message to chat history immediately
                            let user_message = Message::new(
                                session_id.clone(),
                                MessageContent::UserPrompt {
                                    content: vec![agent_client_protocol::ContentBlock::Text(
                                        agent_client_protocol::TextContent {
                                            text: content.clone(),
                                            annotations: Default::default(),
                                        },
                                    )],
                                },
                            );

                            // Add to current tab's chat view
                            if let Err(e) = active_tab.chat_view.add_message(user_message).await {
                                self.error_message = Some(format!("Failed to add message: {}", e));
                            }

                            let (tx, _rx) = oneshot::channel();
                            let _ = self.ui_tx.send(UiToApp::SendMessage {
                                agent_name: active_tab.agent_name.clone(),
                                session_id,
                                content,
                                respond_to: tx,
                            });
                        } else {
                            self.error_message = Some("No active session for this tab".to_string());
                        }
                    }
                }
            }
        }

        // Handle global keys first
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                return Ok(());
            }
            KeyCode::Esc => {
                self.error_message = None;
                self.show_help = false;
                return Ok(());
            }
            _ => {}
        }

        // If help or error is showing, consume any key to dismiss
        if self.show_help || self.error_message.is_some() {
            self.show_help = false;
            self.error_message = None;
            return Ok(());
        }

        // Check if chat input is active before processing global keys
        let chat_input_active = if let Some(active_tab) = self.tabs.get(self.active_tab) {
            active_tab.chat_view.is_input_mode()
        } else {
            false
        };

        // Tab navigation (always allowed)
        match key.code {
            KeyCode::Tab => {
                self.next_tab();
                return Ok(());
            }
            KeyCode::BackTab => {
                self.prev_tab();
                return Ok(());
            }
            _ => {}
        }

        // Only process these global keys if chat input is NOT active
        if !chat_input_active {
            match key.code {
                KeyCode::Char('n') => {
                    // Create new session with default agent
                    self.create_new_session().await?;
                    return Ok(());
                }
                KeyCode::Char('a') => {
                    // Show agent selector (toggle visibility)
                    self.agent_selector.toggle_visibility();
                    return Ok(());
                }
                KeyCode::Char('q') => {
                    // TODO: Implement quit functionality
                    return Ok(());
                }
                _ => {}
            }
        }

        // Pass to active tab
        if let Some(active_tab) = self.tabs.get_mut(self.active_tab) {
            active_tab.chat_view.handle_key_event(key).await?;
        }

        Ok(())
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Update chat views
        for tab in &mut self.tabs {
            tab.chat_view.tick().await?;
        }

        // Update status bar
        self.status_bar.tick().await?;

        // Ensure long-running ambience is registered (if enabled)
        if self.config.effects.enabled && !self.ambient_fx_initialized {
            // Subtle global hue drift
            self.fx.add_unique_effect("global_drift", subtle_hsl_drift());
            // Neon border pulse
            self.fx.add_unique_effect("neon_border", neon_pulse_border(&self.theme));
            self.ambient_fx_initialized = true;
        }

        Ok(())
    }

    pub async fn add_message(&mut self, agent_name: &str, message: Message) -> Result<()> {
        // Find the appropriate tab for this agent/session
        if let Some(tab) = self.tabs.iter_mut().find(|t| {
            t.agent_name == agent_name && t.session_id.as_ref() == Some(&message.session_id)
        }) {
            tab.chat_view.add_message(message).await?;
            // Attention effect over the chat area when a message lands
            let accent = self.theme.palette.accent_b;
            let area_ref = tab.chat_area_ref.clone();
            let attn = fx::dynamic_area(area_ref.clone(), sweep_in_attention(accent));
            let glitch = fx::dynamic_area(area_ref, glitch_burst());
            self.fx.add_unique_effect("chat-attn", fx::parallel(&[attn, glitch]));
        }
        Ok(())
    }

    pub fn set_agent_status(&mut self, agent_name: &str, status: String) {
        self.status_bar
            .set_agent_status(agent_name.to_string(), status);
    }

    pub fn show_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn add_session(&mut self, agent_name: &str, session_id: SessionId) -> Result<()> {
        let tab_name = format!("{} ({})", agent_name, &session_id.0[..8]);

        // If a pending tab exists for this agent, update it instead of creating a new one
        if let Some((idx, t)) = self
            .tabs
            .iter_mut()
            .enumerate()
            .find(|(_, t)| t.agent_name == agent_name && t.session_id.is_none())
        {
            t.session_id = Some(session_id);
            t.name = tab_name;

            // Deactivate other tabs and activate this one
            for (i, existing_tab) in self.tabs.iter_mut().enumerate() {
                existing_tab.active = i == idx;
            }
            self.active_tab = idx;
        } else {
            let tab = Tab {
                name: tab_name,
                agent_name: agent_name.to_string(),
                session_id: Some(session_id),
                chat_view: ChatView::new(self.config.layout.chat_history_limit),
                active: true,
                chat_area_ref: RefRect::default(),
            };

            // Deactivate other tabs
            for existing_tab in &mut self.tabs {
                existing_tab.active = false;
            }

            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
        }

        Ok(())
    }

    pub fn remove_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);

            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    pub fn get_active_session(&self) -> Option<(&str, &SessionId)> {
        self.tabs.get(self.active_tab).and_then(|tab| {
            tab.session_id
                .as_ref()
                .map(|sid| (tab.agent_name.as_str(), sid))
        })
    }

    pub async fn create_new_session(&mut self) -> Result<()> {
        // Request a real session from the App layer without blocking the UI.
        let agent_name = "claude-code".to_string();
        let (tx, rx) = oneshot::channel();
        // Best-effort send; errors surface through AppMessage::Error handling
        let _ = self.ui_tx.send(UiToApp::CreateSession {
            agent_name,
            respond_to: tx,
        });

        // Create or focus a pending tab so the user sees immediate feedback
        if let Some(existing_idx) = self
            .tabs
            .iter()
            .position(|t| t.agent_name == "claude-code" && t.session_id.is_none())
        {
            // Focus the existing pending tab
            for (i, t) in self.tabs.iter_mut().enumerate() {
                t.active = i == existing_idx;
            }
            self.active_tab = existing_idx;
        } else {
            let tab = Tab {
                name: "claude-code (creating)".to_string(),
                agent_name: "claude-code".to_string(),
                session_id: None,
                chat_view: ChatView::new(self.config.layout.chat_history_limit),
                active: true,
                chat_area_ref: RefRect::default(),
            };
            for t in &mut self.tabs {
                t.active = false;
            }
            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
        }

        // Provide immediate, non-blocking UI feedback
        self.status_bar
            .set_agent_status("claude-code".to_string(), "Creating session...".to_string());

        // Optionally, we could await rx and use the SessionId to name the tab,
        // but to keep the UI responsive we rely on AppMessage::SessionCreated
        // to add the tab when the session is ready.
        let _ = rx;
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl TuiManager {
    fn apply_fx(&mut self, frame: &mut Frame) {
        // Use frame buffer and area for post-processing effects
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_fx_tick);
        self.last_fx_tick = now;

        let elapsed_fx: FxDuration = elapsed.into();
        let area = frame.area();
        self.fx.process_effects(elapsed_fx, frame.buffer_mut(), area);
    }

    fn apply_startup_fx(&mut self, frame: &mut Frame) {
        // Initialize startup effect once with snapshot of current UI into a buffer
        if self.startup_effect.is_none() {
            let area = frame.area();
            let target = ref_count(Buffer::empty(area));
            // Snapshot current frame into target
            {
                let src = frame.buffer_mut().clone();
                let mut dst = target.borrow_mut();
                src.render_buffer(ratatui::layout::Offset::default(), &mut dst);
            }
            // Effect morphs rain into target UI
            self.startup_effect = Some(matrix_rain_morph_with_duration(target, self.startup_duration_ms));
            self.last_fx_tick = Instant::now();
        }

        if let Some(effect) = &mut self.startup_effect {
            let now = Instant::now();
            let elapsed = now.saturating_duration_since(self.last_fx_tick);
            self.last_fx_tick = now;
            let area = frame.area();
            let mut dur: FxDuration = elapsed.into();
            effect.process(dur, frame.buffer_mut(), area);
            if effect.done() {
                self.startup_effect = None;
                self.startup_running = false;
                // Initialize ambient fx after startup completes
                self.ambient_fx_initialized = false; // ensure init in next tick
            }
        }
    }
}
