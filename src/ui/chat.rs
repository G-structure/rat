use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::collections::VecDeque;

use crate::acp::{Message, MessageContent};

#[derive(Debug, Clone)]
pub struct ChatView {
    messages: VecDeque<Message>,
    max_messages: usize,
    input_buffer: String,
    input_mode: bool,
    // Scroll offset measured in visual lines from the bottom (0 = stick to bottom)
    scroll_offset: usize,
    // Cached layout info from last render to make scrolling feel correct
    last_total_lines: usize,
    last_visible_lines: usize,
    last_inner_width: usize,
}

impl ChatView {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
            input_buffer: String::new(),
            input_mode: false,
            scroll_offset: 0,
            last_total_lines: 0,
            last_visible_lines: 0,
            last_inner_width: 0,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Messages
                Constraint::Length(3), // Input
            ])
            .split(area);

        // Render messages
        self.render_messages(frame, chunks[0]);

        // Render input
        self.render_input(frame, chunks[1]);

        Ok(())
    }

    fn render_messages(&mut self, frame: &mut Frame, area: Rect) {
        // Area available for content inside the border
        let inner_width = area.width.saturating_sub(2) as usize;
        let visible_lines = area.height.saturating_sub(2) as usize;

        // Build wrapped, styled lines for all messages
        let mut lines: Vec<Line> = Vec::new();
        for msg in &self.messages {
            let msg_lines = self.format_message_lines(msg, inner_width);
            lines.extend(msg_lines);
        }

        let total_lines = lines.len();

        // Persist last layout for scroll logic elsewhere
        self.last_total_lines = total_lines;
        self.last_visible_lines = visible_lines;
        self.last_inner_width = inner_width;

        // Determine scroll origin from bottom
        let base_from_top = total_lines.saturating_sub(visible_lines);
        let start_from_top = if self.scroll_offset == 0 {
            base_from_top
        } else {
            base_from_top.saturating_sub(self.scroll_offset)
        };

        let title = if total_lines <= visible_lines {
            format!("Conversation ({} messages)", self.messages.len())
        } else if self.scroll_offset == 0 {
            format!("Conversation ({} messages) - Latest", self.messages.len())
        } else {
            format!("Conversation ({} messages) - ↑↓ to scroll", self.messages.len())
        };

        let para = Paragraph::new(lines)
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((start_from_top as u16, 0));

        frame.render_widget(para, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input_style = if self.input_mode {
            Style::default().green()
        } else {
            Style::default()
        };

        let input_title = if self.input_mode {
            "Message (Press Enter to send, Esc to cancel)"
        } else {
            "Press Enter to start typing"
        };

        let input = Paragraph::new(self.input_buffer.as_str()).block(
            Block::default()
                .title(input_title)
                .borders(Borders::ALL)
                .border_style(input_style),
        );

        frame.render_widget(input, area);

        // Show cursor if in input mode
        if self.input_mode {
            frame.set_cursor_position(Position {
                x: area.x + 1 + self.input_buffer.len() as u16,
                y: area.y + 1,
            });
        }
    }

    fn format_message_lines(&self, message: &Message, max_width: usize) -> Vec<Line<'static>> {
        let timestamp = message.timestamp.format("%H:%M:%S");

        let (prefix, body, style) = match &message.content {
            MessageContent::UserPrompt { .. } => (
                format!("[{}] You: ", timestamp),
                self.extract_text_content(message),
                Style::default().cyan(),
            ),
            MessageContent::AgentResponse { content } => (
                format!("[{}] Agent: ", timestamp),
                self.content_to_string(content),
                Style::default().green(),
            ),
            MessageContent::AgentMessageChunk { content } => (
                format!("[{}] Agent: ", timestamp),
                self.content_to_string(content),
                Style::default().green(),
            ),
            MessageContent::EditProposed { edit } => (
                format!("[{}] Edit proposed: ", timestamp),
                edit.file_path.clone(),
                Style::default().yellow(),
            ),
            MessageContent::EditAccepted { edit_id } => (
                format!("[{}] Edit accepted: ", timestamp),
                edit_id.clone(),
                Style::default().green(),
            ),
            MessageContent::EditRejected { edit_id } => (
                format!("[{}] Edit rejected: ", timestamp),
                edit_id.clone(),
                Style::default().red(),
            ),
            MessageContent::ToolCall { tool_call } => (
                format!("[{}] Tool call: ", timestamp),
                tool_call.tool_name.clone(),
                Style::default().blue(),
            ),
            MessageContent::ToolResult { result, .. } => (
                format!("[{}] Tool result: ", timestamp),
                format!("{} chars", result.len()),
                Style::default().blue(),
            ),
            MessageContent::SessionStatus { status } => (
                format!("[{}] Status: ", timestamp),
                status.clone(),
                Style::default().gray(),
            ),
            MessageContent::Error { error } => (
                format!("[{}] Error: ", timestamp),
                error.clone(),
                Style::default().red(),
            ),
        };

        self.wrap_styled(format!("{}{}", prefix, body), style, max_width)
    }

    fn extract_text_content(&self, message: &Message) -> String {
        match &message.content {
            MessageContent::UserPrompt { content } => content
                .iter()
                .map(|c| self.content_to_string(c))
                .collect::<Vec<_>>()
                .join(" "),
            MessageContent::AgentResponse { content } => self.content_to_string(content),
            MessageContent::AgentMessageChunk { content } => self.content_to_string(content),
            _ => "Non-text content".to_string(),
        }
    }

    fn content_to_string(&self, content: &agent_client_protocol::ContentBlock) -> String {
        match content {
            agent_client_protocol::ContentBlock::Text(text) => text.text.clone(),
            agent_client_protocol::ContentBlock::Image(_) => "[Image]".to_string(),
            _ => "[Unsupported Content]".to_string(),
        }
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if self.input_mode {
                    if !self.input_buffer.trim().is_empty() {
                        // Send message - would need to communicate with app layer
                        self.input_buffer.clear();
                    }
                    self.input_mode = false;
                } else {
                    self.input_mode = true;
                }
            }
            KeyCode::Esc => {
                if self.input_mode {
                    self.input_buffer.clear();
                    self.input_mode = false;
                }
            }
            KeyCode::Char(c) => {
                if self.input_mode {
                    self.input_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                if self.input_mode {
                    self.input_buffer.pop();
                }
            }
            KeyCode::Up => {
                if !self.input_mode {
                    // Scroll up by one visual line (older content)
                    let max_from_bottom = self
                        .last_total_lines
                        .saturating_sub(self.last_visible_lines);
                    if self.scroll_offset < max_from_bottom {
                        self.scroll_offset += 1;
                    }
                }
            }
            KeyCode::Down => {
                if !self.input_mode {
                    // Scroll down by one visual line (toward latest)
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn add_message(&mut self, message: Message) -> Result<()> {
        // If the user has scrolled up, keep their viewport anchored by
        // increasing the offset by the number of visual lines added.
        let mut added_lines = 1usize;
        if self.last_inner_width > 0 {
            let tmp_lines = self.format_message_lines(&message, self.last_inner_width);
            added_lines = tmp_lines.len().max(1);
        }

        self.messages.push_back(message);

        // Keep only the max number of messages
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }

        // Stick to bottom only if already at bottom; otherwise preserve position
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_add(added_lines);
        } else {
            self.scroll_offset = 0;
        }

        Ok(())
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Handle any periodic updates
        Ok(())
    }

    pub fn get_input_buffer(&self) -> &str {
        &self.input_buffer
    }

    pub fn clear_input_buffer(&mut self) {
        self.input_buffer.clear();
    }

    pub fn is_input_mode(&self) -> bool {
        self.input_mode
    }

    pub fn set_input_mode(&mut self, mode: bool) {
        self.input_mode = mode;
        if !mode {
            self.input_buffer.clear();
        }
    }
}

impl ChatView {
    fn wrap_styled(&self, text: String, style: Style, max_width: usize) -> Vec<Line<'static>> {
        if max_width == 0 {
            return vec![Line::from(Span::styled(text, style))];
        }

        // Simple word-wrapping to avoid splitting in the middle of words
        let mut lines: Vec<Line> = Vec::new();
        let mut current = String::new();

        for word in text.split_whitespace() {
            // +1 for the space if current is not empty
            let prospective_len = current.len() + if current.is_empty() { 0 } else { 1 } + word.len();
            if prospective_len <= max_width {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            } else {
                if current.is_empty() {
                    // Very long single word; hard wrap
                    let mut start = 0usize;
                    let bytes = word.as_bytes();
                    while start < bytes.len() {
                        let end = (start + max_width).min(bytes.len());
                        let chunk = &word[start..end];
                        lines.push(Line::from(Span::styled(chunk.to_string(), style)));
                        start = end;
                    }
                } else {
                    lines.push(Line::from(Span::styled(current.clone(), style)));
                    current.clear();
                    // Put the word on a new line or split if still too long
                    if word.len() <= max_width {
                        current.push_str(word);
                    } else {
                        let mut start = 0usize;
                        let bytes = word.as_bytes();
                        while start < bytes.len() {
                            let end = (start + max_width).min(bytes.len());
                            let chunk = &word[start..end];
                            if chunk.len() == max_width {
                                lines.push(Line::from(Span::styled(chunk.to_string(), style)));
                            } else {
                                current.push_str(chunk);
                            }
                            start = end;
                        }
                    }
                }
            }
        }

        if !current.is_empty() {
            lines.push(Line::from(Span::styled(current, style)));
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(String::new(), style)));
        }

        lines
    }
}
