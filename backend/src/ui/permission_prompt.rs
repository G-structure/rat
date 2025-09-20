use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, BorderType},
};
use std::collections::HashMap;

use agent_client_protocol::{RequestPermissionRequest, RequestPermissionOutcome, PermissionOptionId, ToolCallContent, ContentBlock, PermissionOptionKind};
use ratatui::widgets::Wrap;

#[derive(Debug, Clone)]
pub struct PermissionPrompt {
    pub request: Option<RequestPermissionRequest>,
    pub selected_option: usize,
    pub visible: bool,
}

impl PermissionPrompt {
    pub fn new() -> Self {
        Self {
            request: None,
            selected_option: 0,
            visible: false,
        }
    }

    pub fn show(&mut self, request: RequestPermissionRequest) {
        self.request = Some(request);
        self.selected_option = 0;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.request = None;
        self.visible = false;
        self.selected_option = 0;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn get_selected_outcome(&self) -> Option<RequestPermissionOutcome> {
        if let Some(ref request) = self.request {
            if let Some(option) = request.options.get(self.selected_option) {
                Some(RequestPermissionOutcome::Selected {
                    option_id: option.id.clone(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.visible || self.request.is_none() {
            return Ok(());
        }

        // Check for minimum area size
        if area.width < 20 || area.height < 10 {
            return Ok(()); // Skip rendering if too small
        }

        let request = self.request.as_ref().unwrap();

        // Create a centered popup area
        let popup_area = centered_rect(80, 60, area);

        frame.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Content
                Constraint::Length(5), // Options
                Constraint::Length(3), // Instructions
            ])
            .split(popup_area);

        // Title
        let title_block = Block::default()
            .title("🔒 Permission Required")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::from_u32(0xff6b6b)));

        let title_text = vec![
            Line::from("The agent is requesting permission to perform an action."),
        ];

        let title = Paragraph::new(title_text)
            .block(title_block)
            .alignment(Alignment::Center);

        frame.render_widget(title, chunks[0]);

        // Content area - show tool call details
        let content_block = Block::default()
            .title("Tool Call Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::from_u32(0x4ecdc4)));

        let mut content_lines = Vec::new();

        // Tool title
        if let Some(title) = &request.tool_call.fields.title {
            content_lines.push(Line::from(vec![
                Span::styled("Tool: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(title.clone(), Style::default().fg(Color::White)),
            ]));
        }

        // Tool kind
        if let Some(kind) = &request.tool_call.fields.kind {
            content_lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(format!("{:?}", kind), Style::default().fg(Color::White)),
            ]));
        }

        // Show some content preview if available
        if let Some(content) = &request.tool_call.fields.content {
            if !content.is_empty() {
                content_lines.push(Line::from(""));
                content_lines.push(Line::from(Span::styled(
                    "Content Preview:",
                    Style::default().fg(Color::Cyan).bold(),
                )));

                // Show first content item as preview
                if let Some(first_content) = content.first() {
                    match first_content {
                        ToolCallContent::Content { content } => {
                            match content {
                                ContentBlock::Text(text) => {
                                    let preview = if text.text.len() > 100 {
                                        format!("{}...", &text.text[..100])
                                    } else {
                                        text.text.clone()
                                    };
                                    for line in preview.lines().take(3) {
                                        content_lines.push(Line::from(vec![
                                            Span::styled("  ", Style::default()),
                                            Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
                                        ]));
                                    }
                                }
                                _ => {
                                    content_lines.push(Line::from(vec![
                                        Span::styled("  ", Style::default()),
                                        Span::styled("[Non-text content]", Style::default().fg(Color::Gray)),
                                    ]));
                                }
                            }
                        }
                        ToolCallContent::Diff { diff } => {
                            content_lines.push(Line::from(vec![
                                Span::styled("  File: ", Style::default().fg(Color::Green)),
                                Span::styled(diff.path.display().to_string(), Style::default().fg(Color::White)),
                            ]));
                        }
                    }
                }
            }
        }

        let content = Paragraph::new(content_lines)
            .block(content_block)
            .wrap(Wrap { trim: true });

        frame.render_widget(content, chunks[1]);

        // Options area
        let options_block = Block::default()
            .title("Choose an Option")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::from_u32(0x45b7d1)));

        let mut options_lines = Vec::new();

        for (i, option) in request.options.iter().enumerate() {
            let mut style = Style::default().fg(Color::White);
            let mut prefix = "  ";

            if i == self.selected_option {
                style = style.bg(Color::from_u32(0x2d5aa0)).bold();
                prefix = "▶ ";
            }

            let option_text = format!("{} {}", prefix, option.name);

            // Add keyboard shortcut hint
            let shortcut = match option.id.0.as_ref() {
                "approve" => " (y)",
                "deny" => " (n)",
                "maybe" => " (m)",
                _ => "",
            };

            options_lines.push(Line::from(vec![
                Span::styled(option_text, style),
                Span::styled(shortcut, Style::default().fg(Color::DarkGray)),
            ]));

            // Add description of what this option does
            let description = match option.kind {
                PermissionOptionKind::AllowOnce => "Allow this action once",
                PermissionOptionKind::AllowAlways => "Allow this action and remember",
                PermissionOptionKind::RejectOnce => "Reject this action once",
                PermissionOptionKind::RejectAlways => "Reject this action and remember",
            };

            options_lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(description, Style::default().fg(Color::DarkGray).italic()),
            ]));

            if i < request.options.len() - 1 {
                options_lines.push(Line::from(""));
            }
        }

        let options = Paragraph::new(options_lines)
            .block(options_block);

        frame.render_widget(options, chunks[2]);

        // Instructions
        let instructions_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::from_u32(0x96ceb4)));

        let instructions_text = vec![
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" Navigate • ", Style::default().fg(Color::White)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" Select • ", Style::default().fg(Color::White)),
                Span::styled("Esc", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" Cancel", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("y/n/m", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" Quick select option", Style::default().fg(Color::White)),
            ]),
        ];

        let instructions = Paragraph::new(instructions_text)
            .block(instructions_block)
            .alignment(Alignment::Center);

        frame.render_widget(instructions, chunks[3]);

        Ok(())
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<RequestPermissionOutcome> {
        if !self.visible || self.request.is_none() {
            return None;
        }

        let request = self.request.as_ref().unwrap();

        match key.code {
            KeyCode::Enter => {
                // Return the selected option
                self.get_selected_outcome()
            }
            KeyCode::Esc => {
                // Cancel the request
                Some(RequestPermissionOutcome::Cancelled)
            }
            KeyCode::Up => {
                if self.selected_option > 0 {
                    self.selected_option -= 1;
                } else {
                    self.selected_option = request.options.len() - 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected_option < request.options.len() - 1 {
                    self.selected_option += 1;
                } else {
                    self.selected_option = 0;
                }
                None
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Quick approve
                if let Some(option) = request.options.iter().find(|o| o.id.0.as_ref() == "approve") {
                    Some(RequestPermissionOutcome::Selected {
                        option_id: option.id.clone(),
                    })
                } else {
                    None
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                // Quick deny
                if let Some(option) = request.options.iter().find(|o| o.id.0.as_ref() == "deny") {
                    Some(RequestPermissionOutcome::Selected {
                        option_id: option.id.clone(),
                    })
                } else {
                    None
                }
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                // Quick maybe
                if let Some(option) = request.options.iter().find(|o| o.id.0.as_ref() == "maybe") {
                    Some(RequestPermissionOutcome::Selected {
                        option_id: option.id.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
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