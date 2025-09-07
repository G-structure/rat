use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::collections::VecDeque;

use crate::acp::message::EditProposal;

pub struct DiffView {
    proposals: VecDeque<EditProposal>,
    state: ListState,
    visible: bool,
    selected_proposal: Option<EditProposal>,
    show_diff_detail: bool,
}

impl DiffView {
    pub fn new() -> Self {
        Self {
            proposals: VecDeque::new(),
            state: ListState::default(),
            visible: false,
            selected_proposal: None,
            show_diff_detail: false,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        if self.show_diff_detail {
            self.render_diff_detail(frame, area)
        } else {
            self.render_proposal_list(frame, area)
        }
    }

    fn render_proposal_list(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let popup_area = centered_rect(80, 60, area);
        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = self
            .proposals
            .iter()
            .map(|proposal| self.format_proposal_item(proposal))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Edit Proposals")
                    .borders(Borders::ALL)
                    .border_style(Style::default().yellow()),
            )
            .highlight_style(Style::default().reversed())
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, popup_area, &mut self.state);

        // Show help text
        let help_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height - 3,
            width: popup_area.width,
            height: 3,
        };

        let help_text = vec![Line::from(
            "↑/↓: Navigate, Enter: View diff, y: Accept, n: Reject, Esc: Close",
        )];

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().gray());

        frame.render_widget(help, help_area);

        Ok(())
    }

    fn render_diff_detail(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Some(ref proposal) = self.selected_proposal {
            let popup_area = centered_rect(90, 80, area);
            frame.render_widget(Clear, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(1),    // Diff content
                    Constraint::Length(3), // Actions
                ])
                .split(popup_area);

            // Header
            let header = Paragraph::new(format!("File: {}", proposal.file_path)).block(
                Block::default()
                    .title("Diff View")
                    .borders(Borders::ALL)
                    .border_style(Style::default().blue()),
            );
            frame.render_widget(header, chunks[0]);

            // Diff content
            let diff_lines = self.format_diff_content(proposal);
            let diff_content = List::new(diff_lines).block(Block::default().borders(Borders::ALL));
            frame.render_widget(diff_content, chunks[1]);

            // Actions
            let actions = Paragraph::new("y: Accept edit | n: Reject edit | Esc: Back to list")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().cyan());
            frame.render_widget(actions, chunks[2]);
        }

        Ok(())
    }

    fn format_proposal_item(&self, proposal: &EditProposal) -> ListItem {
        let description = proposal.description.as_deref().unwrap_or("No description");
        let item_text = format!("{}: {}", proposal.file_path, description);

        ListItem::new(item_text).style(Style::default().yellow())
    }

    fn format_diff_content(&self, proposal: &EditProposal) -> Vec<ListItem> {
        if proposal.diff.is_empty() {
            // Generate a simple diff if not provided
            self.generate_simple_diff(proposal)
        } else {
            // Parse existing diff
            proposal
                .diff
                .lines()
                .map(|line| self.format_diff_line(line))
                .collect()
        }
    }

    fn generate_simple_diff(&self, proposal: &EditProposal) -> Vec<ListItem> {
        let mut items = Vec::new();

        items.push(ListItem::new("--- Original").style(Style::default().red()));
        items.push(ListItem::new("+++ Proposed").style(Style::default().green()));
        items.push(ListItem::new(""));

        // Show first few lines of original content
        for (i, line) in proposal.original_content.lines().take(5).enumerate() {
            items
                .push(ListItem::new(format!("-{}: {}", i + 1, line)).style(Style::default().red()));
        }

        items.push(ListItem::new(""));

        // Show first few lines of proposed content
        for (i, line) in proposal.proposed_content.lines().take(5).enumerate() {
            items.push(
                ListItem::new(format!("+{}: {}", i + 1, line)).style(Style::default().green()),
            );
        }

        items
    }

    fn format_diff_line(&self, line: &str) -> ListItem {
        if line.starts_with('+') {
            ListItem::new(line).style(Style::default().green())
        } else if line.starts_with('-') {
            ListItem::new(line).style(Style::default().red())
        } else if line.starts_with("@@") {
            ListItem::new(line).style(Style::default().cyan().bold())
        } else {
            ListItem::new(line)
        }
    }

    pub fn add_proposal(&mut self, proposal: EditProposal) {
        self.proposals.push_back(proposal);

        // Auto-select first item if nothing is selected
        if self.state.selected().is_none() && !self.proposals.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn remove_proposal(&mut self, proposal_id: &str) {
        self.proposals.retain(|p| p.id != proposal_id);

        // Adjust selection if needed
        if let Some(selected) = self.state.selected() {
            if selected >= self.proposals.len() && !self.proposals.is_empty() {
                self.state.select(Some(self.proposals.len() - 1));
            } else if self.proposals.is_empty() {
                self.state.select(None);
            }
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.show_diff_detail = false;

        if !self.proposals.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.show_diff_detail = false;
        self.selected_proposal = None;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn next(&mut self) {
        if self.proposals.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.proposals.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.proposals.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.proposals.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn show_diff_detail(&mut self) {
        if let Some(selected) = self.state.selected() {
            if let Some(proposal) = self.proposals.get(selected) {
                self.selected_proposal = Some(proposal.clone());
                self.show_diff_detail = true;
            }
        }
    }

    pub fn back_to_list(&mut self) {
        self.show_diff_detail = false;
        self.selected_proposal = None;
    }

    pub fn get_selected_proposal(&self) -> Option<&EditProposal> {
        self.state.selected().and_then(|i| self.proposals.get(i))
    }

    pub fn has_proposals(&self) -> bool {
        !self.proposals.is_empty()
    }

    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
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
