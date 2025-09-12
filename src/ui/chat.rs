use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap, BorderType},
};
use std::collections::VecDeque;

use crate::acp::{Message, MessageContent, message::{ToolCallRequest, EditProposal}};
use crate::utils::diff::{DiffGenerator, DiffLineType};
use agent_client_protocol::{ToolCallUpdate, ToolCallStatus, ToolCallContent, ContentBlock};

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
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    let content_area = chunks[0];
    let input_area = chunks[1];

    let msg_area = content_area;

    self.render_messages(frame, msg_area);
    self.render_input(frame, input_area);

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
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::from_u32(0x18e5ff)))
                    .border_type(BorderType::Double),
            )
            .wrap(Wrap { trim: false })
            .scroll((start_from_top as u16, 0));

        frame.render_widget(para, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input_style = if self.input_mode {
            Style::default().fg(Color::from_u32(0xff2e88))
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_title = if self.input_mode {
            "Message (Enter: send, Esc: cancel)"
        } else {
            "Press Enter to start typing"
        };

        let input = Paragraph::new(self.input_buffer.as_str()).block(
            Block::default()
                .title(input_title)
                .borders(Borders::ALL)
                .border_style(input_style)
                .border_type(BorderType::Double),
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

        match &message.content {
            MessageContent::EditProposed { edit } => {
                // Special handling for edit proposals - return neovim-style diff
                let mut lines = Vec::new();

                // Add timestamp header with clean styling
                lines.push(Line::from(Span::styled(
                    format!("[{}] Code Edit", timestamp),
                    Style::default().fg(Color::Yellow),
                )));

                // Add separator
                lines.push(Line::from(Span::styled(
                    "─".repeat(max_width.min(50)),
                    Style::default().fg(Color::DarkGray),
                )));

                // Add the neovim-style diff content
                lines.extend(self.format_edit_content_styled(edit, max_width));

                lines
            },
            _ => {
                // Standard message formatting for all other types
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
                        format!("[{}] Tool Call", timestamp),
                        self.format_tool_call_content(tool_call),
                        Style::default().blue(),
                    ),
                    MessageContent::ToolResult { result, .. } => (
                        format!("[{}] Tool Result", timestamp),
                        self.format_tool_result_content(result),
                        Style::default().blue(),
                    ),
                    MessageContent::ToolCallUpdate { update } => (
                        format!("[{}] Tool Update", timestamp),
                        self.format_tool_call_update_content(update),
                        Style::default().cyan(),
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
                    MessageContent::Plan(plan) => (
                        format!("[{}] Agent Plan: ", timestamp),
                        self.format_plan_content(plan),
                        Style::default().fg(Color::Cyan),
                    ),
                    MessageContent::EditProposed { .. } => unreachable!("Handled above"),
                };

                self.wrap_styled(format!("{}{}", prefix, body), style, max_width)
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

    fn content_to_string(&self, content: &agent_client_protocol::ContentBlock) -> String {
        match content {
            agent_client_protocol::ContentBlock::Text(text) => text.text.clone(),
            agent_client_protocol::ContentBlock::Image(_) => "[Image]".to_string(),
            _ => "[Unsupported Content]".to_string(),
        }
    }

    fn format_plan_content(&self, plan: &agent_client_protocol::Plan) -> String {
        if plan.entries.is_empty() {
            return "┌─ Agent Plan ─┐\n│   No tasks   │\n└──────────────┘".to_string();
        }

        let mut lines = Vec::new();
        lines.push("┌─ Agent Plan ──────────────────────────┐".to_string());

        for entry in &plan.entries {
            let status_icon = match entry.status {
                agent_client_protocol::PlanEntryStatus::Pending => "⏳",
                agent_client_protocol::PlanEntryStatus::InProgress => "⚡",
                agent_client_protocol::PlanEntryStatus::Completed => "✅",
            };

            let priority_icon = match entry.priority {
                agent_client_protocol::PlanEntryPriority::High => "🔴",
                agent_client_protocol::PlanEntryPriority::Medium => "🟡",
                agent_client_protocol::PlanEntryPriority::Low => "🟢",
            };

            let priority_text = match entry.priority {
                agent_client_protocol::PlanEntryPriority::High => "High",
                agent_client_protocol::PlanEntryPriority::Medium => "Medium",
                agent_client_protocol::PlanEntryPriority::Low => "Low",
            };

            // Truncate content if too long for display, but be more conservative
            let content = if entry.content.len() > 25 {
                format!("{}...", &entry.content[..22])
            } else {
                entry.content.clone()
            };

            let line = format!("│ {} {} {}: {} │", status_icon, priority_icon, priority_text, content);
            lines.push(line);
        }

        lines.push("└───────────────────────────────────────┘".to_string());
        lines.join("\n")
    }

    fn format_tool_call_content(&self, tool_call: &ToolCallRequest) -> String {
        let mut lines = Vec::new();
        lines.push("┌─ Tool Call ──────────────────────────────┐".to_string());

        // Tool name
        lines.push(format!("│ 🔧 {} │", tool_call.tool_name));

        // Parameters (simplified JSON preview)
        let params_str = if tool_call.parameters.is_null() {
            "No parameters".to_string()
        } else {
            let json_str = serde_json::to_string_pretty(&tool_call.parameters)
                .unwrap_or_else(|_| "Invalid JSON".to_string());
            if json_str.len() > 35 {
                format!("{}...", &json_str[..32])
            } else {
                json_str
            }
        };
        lines.push(format!("│ 📋 {} │", params_str));

        // Permission status
        let perm_status = if tool_call.requires_permission {
            "🔒 Requires permission"
        } else {
            "✅ Auto-approved"
        };
        lines.push(format!("│ {} │", perm_status));

        lines.push("└─────────────────────────────────────────┘".to_string());
        lines.join("\n")
    }

    fn format_tool_result_content(&self, result: &str) -> String {
        let mut lines = Vec::new();
        lines.push("┌─ Tool Result ────────────────────────────┐".to_string());

        // Result preview
        let preview = if result.len() > 35 {
            format!("{}...", &result[..32])
        } else {
            result.to_string()
        };
        lines.push(format!("│ 📄 {} │", preview));

        // Stats
        let lines_count = result.lines().count();
        let chars_count = result.len();
        lines.push(format!("│ 📊 {} lines, {} chars │", lines_count, chars_count));

        lines.push("└─────────────────────────────────────────┘".to_string());
        lines.join("\n")
    }

    fn format_tool_call_update_content(&self, update: &ToolCallUpdate) -> String {
        // Compact single-line format to save vertical space
        let status_icon = match update.fields.status {
            Some(ToolCallStatus::Completed) => "✅",
            Some(ToolCallStatus::Failed) => "❌",
            Some(ToolCallStatus::InProgress) => "🔄",
            Some(ToolCallStatus::Pending) => "⏳",
            None => "❓",
        };

        let status_text = match update.fields.status {
            Some(ToolCallStatus::Completed) => "Completed",
            Some(ToolCallStatus::Failed) => "Failed",
            Some(ToolCallStatus::InProgress) => "In Progress",
            Some(ToolCallStatus::Pending) => "Pending",
            None => "Unknown",
        };

        // Single line format: [icon] tool_id - Status
        format!("{} {} - {}", status_icon, update.id.0, status_text)
    }

    fn format_edit_content_styled(&self, edit: &EditProposal, max_width: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // File header (like git diff)
        lines.push(Line::from(Span::styled(
            format!("diff --git a/{} b/{}", edit.file_path, edit.file_path),
            Style::default().fg(Color::Magenta).bold(),
        )));

        // Index line (simulated)
        lines.push(Line::from(Span::styled(
            "index 0000000..1111111 100644",
            Style::default().fg(Color::Magenta),
        )));

        // File path header
        lines.push(Line::from(Span::styled(
            format!("--- a/{}", edit.file_path),
            Style::default().fg(Color::Red),
        )));
        lines.push(Line::from(Span::styled(
            format!("+++ b/{}", edit.file_path),
            Style::default().fg(Color::Green),
        )));

        // Add description if available
        if let Some(desc) = &edit.description {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("// {}", desc),
                Style::default().fg(Color::Blue).italic(),
            )));
        }

        lines.push(Line::from(""));

        // Parse and display the diff with neovim-style formatting
        if let Ok(hunks) = DiffGenerator::parse_diff(&edit.diff) {
            let mut total_additions = 0;
            let mut total_deletions = 0;

            for hunk_idx in 0..hunks.len() {
                if hunk_idx > 0 {
                    lines.push(Line::from(""));
                }

                let hunk = &hunks[hunk_idx];

                // Hunk header with line numbers
                lines.push(Line::from(Span::styled(
                    hunk.header.clone(),
                    Style::default().fg(Color::Cyan).bold(),
                )));

                // Display diff lines with proper colors and prefixes
                for line_idx in 0..hunk.lines.len().min(10) {
                    let line = &hunk.lines[line_idx];
                    let (prefix, style) = match line.line_type {
                        DiffLineType::Added => {
                            total_additions += 1;
                            ("+", Style::default().fg(Color::Green))
                        },
                        DiffLineType::Removed => {
                            total_deletions += 1;
                            ("-", Style::default().fg(Color::Red))
                        },
                        DiffLineType::Context => (" ", Style::default().fg(Color::White)),
                    };

                    // Handle long lines by truncating
                    let line_content = if line.content.len() > (max_width.saturating_sub(3)) {
                        format!("{}...", &line.content[..max_width.saturating_sub(6)])
                    } else {
                        line.content.clone()
                    };

                    lines.push(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(" ", Style::default()),
                        Span::styled(line_content, style),
                    ]));
                }

                // Add ellipsis if there are more lines
                if hunk.lines.len() > 10 {
                    lines.push(Line::from(Span::styled(
                        format!("... ({} more lines)", hunk.lines.len() - 10),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }

            // Summary statistics
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("{} insertions(+), {} deletions(-)", total_additions, total_deletions),
                Style::default().fg(Color::Yellow),
            )));
        } else {
            // Fallback: display raw diff if parsing fails
            lines.push(Line::from(Span::styled(
                "Failed to parse diff, showing raw:",
                Style::default().fg(Color::Red),
            )));
            lines.push(Line::from(""));

            for line in edit.diff.lines().take(15) {
                let (style, prefix) = if line.starts_with('+') {
                    (Style::default().fg(Color::Green), "+")
                } else if line.starts_with('-') {
                    (Style::default().fg(Color::Red), "-")
                } else if line.starts_with("@@") {
                    (Style::default().fg(Color::Cyan).bold(), "")
                } else {
                    (Style::default().fg(Color::White), "")
                };

                let content = if line.len() > max_width {
                    format!("{}...", &line[..max_width.saturating_sub(3)])
                } else {
                    line.to_string()
                };

                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(" ", Style::default()),
                    Span::styled(content, style),
                ]));
            }
        }

        lines
    }

    fn format_edit_content(&self, edit: &EditProposal) -> String {
        let mut lines = Vec::new();
        lines.push("┌─ Code Edit ──────────────────────────────┐".to_string());

        // File path
        let path_display = if edit.file_path.len() > 35 {
            format!("{}...", &edit.file_path[edit.file_path.len().saturating_sub(32)..])
        } else {
            edit.file_path.clone()
        };
        lines.push(format!("│ 📁 {} │", path_display));

        // Description if available
        if let Some(desc) = &edit.description {
            let desc_display = if desc.len() > 35 {
                format!("{}...", &desc[..32])
            } else {
                desc.clone()
            };
            lines.push(format!("│ 💬 {} │", desc_display));
        }

        lines.push("├─────────────────────────────────────────┤".to_string());

        // Parse and display the diff with visual formatting
        if let Ok(hunks) = DiffGenerator::parse_diff(&edit.diff) {
            let mut total_additions = 0;
            let mut total_deletions = 0;

            for (hunk_idx, hunk) in hunks.iter().enumerate() {
                if hunk_idx > 0 {
                    lines.push("│ ···                                      │".to_string());
                }

                // Hunk header
                lines.push(format!("│ {} │", hunk.header));

                // Show up to 8 lines from this hunk
                let display_lines = hunk.lines.iter().take(8).collect::<Vec<_>>();
                for line in display_lines {
                    let prefix = match line.line_type {
                        DiffLineType::Added => "+",
                        DiffLineType::Removed => "-",
                        DiffLineType::Context => " ",
                    };

                    let line_content = if line.content.len() > 35 {
                        format!("{}...", &line.content[..32])
                    } else {
                        line.content.clone()
                    };

                    lines.push(format!("│ {} {} │", prefix, line_content));

                    match line.line_type {
                        DiffLineType::Added => total_additions += 1,
                        DiffLineType::Removed => total_deletions += 1,
                        _ => {}
                    }
                }

                if hunk.lines.len() > 8 {
                    lines.push(format!("│ ... ({} more lines)                 │", hunk.lines.len() - 8));
                }
            }

            lines.push("├─────────────────────────────────────────┤".to_string());
            lines.push(format!("│ 📊 Changes: +{} -{} lines               │", total_additions, total_deletions));
        } else {
            // Fallback if diff parsing fails
            let diff_lines: Vec<&str> = edit.diff.lines().collect();
            let preview_lines = diff_lines.iter().take(5).cloned().collect::<Vec<_>>().join("\n");
            let diff_preview = if preview_lines.len() > 30 {
                format!("{}...", &preview_lines[..27])
            } else {
                preview_lines
            };
            lines.push(format!("│ 🔄 {} │", diff_preview));

            let additions = diff_lines.iter().filter(|l| l.starts_with('+')).count();
            let deletions = diff_lines.iter().filter(|l| l.starts_with('-')).count();
            lines.push(format!("│ 📊 +{} -{} │", additions, deletions));
        }

        lines.push("└─────────────────────────────────────────┘".to_string());
        lines.join("\n")
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