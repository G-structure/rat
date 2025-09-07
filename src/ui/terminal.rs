use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::collections::VecDeque;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

pub struct TerminalView {
    output_lines: VecDeque<TerminalLine>,
    max_lines: usize,
    scroll_offset: usize,
    processes: Vec<TerminalProcess>,
    visible: bool,
}

#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: TerminalLineLevel,
}

#[derive(Debug, Clone)]
pub enum TerminalLineLevel {
    Output,
    Error,
    Command,
    System,
}

#[derive(Debug)]
pub struct TerminalProcess {
    pub id: String,
    pub command: String,
    pub child: Child,
    pub status: ProcessStatus,
    pub output_rx: Option<mpsc::UnboundedReceiver<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Completed(i32),
    Failed(String),
}

impl TerminalView {
    pub fn new(max_lines: usize) -> Self {
        Self {
            output_lines: VecDeque::new(),
            max_lines,
            scroll_offset: 0,
            processes: Vec::new(),
            visible: false,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Output
            ])
            .split(area);

        // Header with process info
        self.render_header(frame, chunks[0]);

        // Terminal output
        self.render_output(frame, chunks[1]);

        Ok(())
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let active_count = self
            .processes
            .iter()
            .filter(|p| p.status == ProcessStatus::Running)
            .count();

        let header_text = format!("Terminal | Active processes: {}", active_count);

        let header = Paragraph::new(header_text).block(
            Block::default()
                .title("Terminal")
                .borders(Borders::ALL)
                .border_style(Style::default().green()),
        );

        frame.render_widget(header, area);
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let visible_lines: Vec<ListItem> = self
            .output_lines
            .iter()
            .skip(self.scroll_offset)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| self.format_terminal_line(line))
            .collect();

        let output_list = List::new(visible_lines).block(Block::default().borders(Borders::ALL));

        frame.render_widget(output_list, area);
    }

    fn format_terminal_line(&self, line: &TerminalLine) -> ListItem {
        let timestamp = line.timestamp.format("%H:%M:%S");
        let formatted = format!("[{}] {}", timestamp, line.content);

        let style = match line.level {
            TerminalLineLevel::Output => Style::default().white(),
            TerminalLineLevel::Error => Style::default().red(),
            TerminalLineLevel::Command => Style::default().green().bold(),
            TerminalLineLevel::System => Style::default().blue(),
        };

        ListItem::new(formatted).style(style)
    }

    pub async fn execute_command(&mut self, command: &str, args: Vec<&str>) -> Result<String> {
        let process_id = uuid::Uuid::new_v4().to_string();

        // Add command to output
        self.add_line(
            format!("> {} {}", command, args.join(" ")),
            TerminalLineLevel::Command,
        );

        // Start process
        let mut child = Command::new(command)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stderr"))?;

        let (output_tx, output_rx) = mpsc::unbounded_channel();

        // Spawn task to read stdout
        let output_tx_clone = output_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                let _ = output_tx_clone.send(line.trim_end().to_string());
                line.clear();
            }
        });

        // Spawn task to read stderr
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                let _ = output_tx.send(format!("ERROR: {}", line.trim_end()));
                line.clear();
            }
        });

        let process = TerminalProcess {
            id: process_id.clone(),
            command: format!("{} {}", command, args.join(" ")),
            child,
            status: ProcessStatus::Running,
            output_rx: Some(output_rx),
        };

        self.processes.push(process);
        Ok(process_id)
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Collect output from active processes
        for process in &mut self.processes {
            if let Some(ref mut rx) = process.output_rx {
                while let Ok(line) = rx.try_recv() {
                    let level = if line.starts_with("ERROR:") {
                        TerminalLineLevel::Error
                    } else {
                        TerminalLineLevel::Output
                    };
                    self.add_line(line, level);
                }
            }

            // Check process status
            if process.status == ProcessStatus::Running {
                match process.child.try_wait() {
                    Ok(Some(exit_status)) => {
                        let code = exit_status.code().unwrap_or(-1);
                        process.status = if exit_status.success() {
                            ProcessStatus::Completed(code)
                        } else {
                            ProcessStatus::Failed(format!("Exit code: {}", code))
                        };

                        let status_msg = match &process.status {
                            ProcessStatus::Completed(_) => "Process completed successfully",
                            ProcessStatus::Failed(reason) => &format!("Process failed: {}", reason),
                            _ => "Unknown status",
                        };

                        self.add_line(
                            format!("Process '{}' finished: {}", process.command, status_msg),
                            TerminalLineLevel::System,
                        );
                    }
                    Ok(None) => {
                        // Process still running
                    }
                    Err(e) => {
                        process.status = ProcessStatus::Failed(e.to_string());
                        self.add_line(
                            format!("Process '{}' error: {}", process.command, e),
                            TerminalLineLevel::System,
                        );
                    }
                }
            }
        }

        // Clean up completed processes
        self.processes
            .retain(|p| p.status == ProcessStatus::Running);

        Ok(())
    }

    pub fn add_line(&mut self, content: String, level: TerminalLineLevel) {
        let line = TerminalLine {
            content,
            timestamp: chrono::Utc::now(),
            level,
        };

        self.output_lines.push_back(line);

        // Keep only max_lines
        while self.output_lines.len() > self.max_lines {
            self.output_lines.pop_front();
        }

        // Auto-scroll to bottom
        self.scroll_offset = self.output_lines.len().saturating_sub(1);
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.output_lines.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn clear(&mut self) {
        self.output_lines.clear();
        self.scroll_offset = 0;
    }

    pub fn get_active_process_count(&self) -> usize {
        self.processes
            .iter()
            .filter(|p| p.status == ProcessStatus::Running)
            .count()
    }

    pub async fn kill_all_processes(&mut self) -> Result<()> {
        for process in &mut self.processes {
            if process.status == ProcessStatus::Running {
                if let Err(e) = process.child.kill().await {
                    self.add_line(
                        format!("Failed to kill process '{}': {}", process.command, e),
                        TerminalLineLevel::Error,
                    );
                } else {
                    self.add_line(
                        format!("Killed process '{}'", process.command),
                        TerminalLineLevel::System,
                    );
                }
            }
        }
        self.processes.clear();
        Ok(())
    }
}
