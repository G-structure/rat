use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct AgentSelector {
    agents: Vec<AgentInfo>,
    state: ListState,
    visible: bool,
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub display_name: String,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

impl AgentSelector {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            agents: Vec::new(),
            state,
            visible: false,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Check for minimum area size
        if area.width < 20 || area.height < 10 {
            return Ok(()); // Skip rendering if too small
        }

        // Center the selector popup
        let popup_area = centered_rect(60, 70, area);
        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self
            .agents
            .iter()
            .map(|agent| {
                let status_text = match agent.status {
                    AgentStatus::Connected => "🟢 Connected".to_string(),
                    AgentStatus::Connecting => "🟡 Connecting...".to_string(),
                    AgentStatus::Disconnected => "🔴 Disconnected".to_string(),
                    AgentStatus::Error(ref e) => format!("❌ Error: {}", e),
                };

                let text = format!("{} - {}", agent.name, status_text);
                ListItem::new(text).style(Style::default().white())
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Select Agent")
                    .borders(Borders::ALL)
                    .border_style(Style::default().blue()),
            )
            .highlight_style(Style::default().reversed())
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, popup_area, &mut self.state);

        // Show help text at bottom
        let help_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height - 3,
            width: popup_area.width,
            height: 3,
        };

        let help_text = vec![Line::from("↑/↓: Navigate, Enter: Select, Esc: Cancel")];

        let help = ratatui::widgets::Paragraph::new(help_text)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().gray());

        frame.render_widget(help, help_area);

        Ok(())
    }

    fn format_agent_item(&self, agent: &AgentInfo) -> ListItem {
        let status_char = match agent.status {
            AgentStatus::Connected => "●",
            AgentStatus::Disconnected => "○",
            AgentStatus::Connecting => "◐",
            AgentStatus::Error(_) => "✗",
        };

        let status_color = match agent.status {
            AgentStatus::Connected => Color::Green,
            AgentStatus::Disconnected => Color::Gray,
            AgentStatus::Connecting => Color::Yellow,
            AgentStatus::Error(_) => Color::Red,
        };

        let capabilities_text = if agent.capabilities.is_empty() {
            String::new()
        } else {
            format!(" ({})", agent.capabilities.join(", "))
        };

        let item_text = format!(
            "{} {}{}",
            status_char, agent.display_name, capabilities_text
        );

        let style = if agent.enabled {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        ListItem::new(item_text)
            .style(style)
            .add_modifier(Modifier::BOLD)
    }

    pub fn show(&mut self) {
        self.visible = true;
        if !self.agents.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn next(&mut self) {
        if self.agents.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.agents.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.agents.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.agents.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected_agent(&self) -> Option<&AgentInfo> {
        self.state.selected().and_then(|i| self.agents.get(i))
    }

    pub fn update_agents(&mut self, agents: Vec<AgentInfo>) {
        let current_selection = self.get_selected_agent().map(|a| a.name.clone());

        self.agents = agents;

        // Try to maintain selection
        if let Some(selected_name) = current_selection {
            if let Some(pos) = self.agents.iter().position(|a| a.name == selected_name) {
                self.state.select(Some(pos));
            } else if !self.agents.is_empty() {
                self.state.select(Some(0));
            }
        } else if !self.agents.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn update_agent_status(&mut self, agent_name: &str, status: AgentStatus) {
        if let Some(agent) = self.agents.iter_mut().find(|a| a.name == agent_name) {
            agent.status = status;
        }
    }

    pub fn get_agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn get_connected_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|a| a.status == AgentStatus::Connected)
            .count()
    }

    pub fn get_agents(&self) -> &[AgentInfo] {
        &self.agents
    }
}

impl AgentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AgentStatus::Connected => "Connected",
            AgentStatus::Disconnected => "Disconnected",
            AgentStatus::Connecting => "Connecting",
            AgentStatus::Error(_) => "Error",
        }
    }
}

impl AgentInfo {
    pub fn new(name: String, display_name: String) -> Self {
        Self {
            name,
            display_name,
            status: AgentStatus::Disconnected,
            capabilities: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_status(mut self, status: AgentStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
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
