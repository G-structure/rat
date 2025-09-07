use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};
use std::collections::HashMap;

use crate::acp::{Message, SessionId};
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
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub name: String,
    pub agent_name: String,
    pub session_id: Option<SessionId>,
    pub chat_view: ChatView,
    pub active: bool,
}

impl TuiManager {
    pub fn new(config: UiConfig) -> Result<Self> {
        Ok(Self {
            config,
            active_tab: 0,
            tabs: Vec::new(),
            agent_selector: AgentSelector::new(),
            status_bar: StatusBar::new(),
            error_message: None,
            show_help: false,
        })
    }

    pub fn render(&mut self, frame: &mut Frame) -> Result<()> {
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

        Ok(())
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let tab_names: Vec<&str> = self.tabs.iter().map(|tab| tab.name.as_str()).collect();

        let tabs = Tabs::new(tab_names)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(Style::default().white())
            .highlight_style(Style::default().yellow().bold())
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
                    .border_style(Style::default().red()),
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
                    .border_style(Style::default().blue()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(popup, area);
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
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

        // Tab navigation
        match key.code {
            KeyCode::Tab => {
                self.next_tab();
                return Ok(());
            }
            KeyCode::BackTab => {
                self.prev_tab();
                return Ok(());
            }
            KeyCode::Char('n') => {
                // Create new session - would need to communicate with app
                return Ok(());
            }
            KeyCode::Char('a') => {
                // Show agent selector
                return Ok(());
            }
            _ => {}
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

        Ok(())
    }

    pub async fn add_message(&mut self, agent_name: &str, message: Message) -> Result<()> {
        // Find the appropriate tab for this agent/session
        if let Some(tab) = self.tabs.iter_mut().find(|t| {
            t.agent_name == agent_name && t.session_id.as_ref() == Some(&message.session_id)
        }) {
            tab.chat_view.add_message(message).await?;
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

        let tab = Tab {
            name: tab_name,
            agent_name: agent_name.to_string(),
            session_id: Some(session_id),
            chat_view: ChatView::new(self.config.layout.chat_history_limit),
            active: true,
        };

        // Deactivate other tabs
        for existing_tab in &mut self.tabs {
            existing_tab.active = false;
        }

        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;

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
