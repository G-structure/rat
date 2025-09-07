use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::collections::VecDeque;

use crate::acp::{Message, MessageContent};

pub struct ChatView {
    messages: VecDeque<Message>,
    max_messages: usize,
    input_buffer: String,
    input_mode: bool,
    scroll_offset: usize,
}

impl ChatView {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
            input_buffer: String::new(),
            input_mode: false,
            scroll_offset: 0,
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

    fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .skip(self.scroll_offset)
            .map(|msg| self.format_message(msg))
            .collect();

        let messages_list =
            List::new(messages).block(Block::default().title("Conversation").borders(Borders::ALL));

        frame.render_widget(messages_list, area);
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

    fn format_message(&self, message: &Message) -> ListItem {
        let timestamp = message.timestamp.format("%H:%M:%S");

        match &message.content {
            MessageContent::UserPrompt { .. } => ListItem::new(format!(
                "[{}] You: {}",
                timestamp,
                self.extract_text_content(message)
            ))
            .style(Style::default().cyan()),
            MessageContent::AgentResponse { content } => ListItem::new(format!(
                "[{}] Agent: {}",
                timestamp,
                self.content_to_string(content)
            ))
            .style(Style::default().green()),
            MessageContent::AgentMessageChunk { content } => ListItem::new(format!(
                "[{}] Agent: {}",
                timestamp,
                self.content_to_string(content)
            ))
            .style(Style::default().green()),
            MessageContent::EditProposed { edit } => {
                ListItem::new(format!("[{}] Edit proposed: {}", timestamp, edit.file_path))
                    .style(Style::default().yellow())
            }
            MessageContent::EditAccepted { edit_id } => {
                ListItem::new(format!("[{}] Edit accepted: {}", timestamp, edit_id))
                    .style(Style::default().green())
            }
            MessageContent::EditRejected { edit_id } => {
                ListItem::new(format!("[{}] Edit rejected: {}", timestamp, edit_id))
                    .style(Style::default().red())
            }
            MessageContent::ToolCall { tool_call } => ListItem::new(format!(
                "[{}] Tool call: {}",
                timestamp, tool_call.tool_name
            ))
            .style(Style::default().blue()),
            MessageContent::ToolResult {
                tool_call_id,
                result,
            } => ListItem::new(format!(
                "[{}] Tool result: {} chars",
                timestamp,
                result.len()
            ))
            .style(Style::default().blue()),
            MessageContent::SessionStatus { status } => {
                ListItem::new(format!("[{}] Status: {}", timestamp, status))
                    .style(Style::default().gray())
            }
            MessageContent::Error { error } => {
                ListItem::new(format!("[{}] Error: {}", timestamp, error))
                    .style(Style::default().red())
            }
        }
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

    fn content_to_string(&self, content: &agent_client_protocol::Content) -> String {
        match content {
            agent_client_protocol::Content::Text(text) => text.clone(),
            agent_client_protocol::Content::Image(_) => "[Image]".to_string(),
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
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll_offset < self.messages.len().saturating_sub(1) {
                    self.scroll_offset += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn add_message(&mut self, message: Message) -> Result<()> {
        self.messages.push_back(message);

        // Keep only the max number of messages
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }

        // Auto-scroll to bottom
        self.scroll_offset = self.messages.len().saturating_sub(1);

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
