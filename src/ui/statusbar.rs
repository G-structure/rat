use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use std::collections::HashMap;

pub struct StatusBar {
    agent_statuses: HashMap<String, String>,
    current_message: String,
    memory_usage: Option<u64>,
    connection_count: usize,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            agent_statuses: HashMap::new(),
            current_message: "Ready".to_string(),
            memory_usage: None,
            connection_count: 0,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Check for minimum area size
        if area.width < 5 || area.height < 1 {
            return Ok(()); // Skip rendering if too small
        }

        let status_text = self.build_status_text();

        let paragraph = Paragraph::new(status_text).style(
            Style::default()
                .bg(Color::from_u32(0x121821))
                .fg(Color::from_u32(0x18e5ff)),
        );

        frame.render_widget(paragraph, area);
        Ok(())
    }

    fn build_status_text(&self) -> String {
        let mut parts = Vec::new();

        // Current message
        parts.push(self.current_message.clone());

        // Agent statuses
        if !self.agent_statuses.is_empty() {
            let agent_info: Vec<String> = self
                .agent_statuses
                .iter()
                .map(|(name, status)| format!("{}:{}", name, status))
                .collect();
            parts.push(format!("Agents[{}]", agent_info.join(", ")));
        }

        // Connection count
        if self.connection_count > 0 {
            parts.push(format!("Connections: {}", self.connection_count));
        }

        // Memory usage
        if let Some(memory) = self.memory_usage {
            parts.push(format!("Mem: {}MB", memory / 1024 / 1024));
        }

        // Current time
        let now = chrono::Local::now();
        parts.push(now.format("%H:%M:%S").to_string());

        format!(" {} ", parts.join(" | "))
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Update memory usage periodically
        self.update_memory_usage();
        Ok(())
    }

    pub fn set_agent_status(&mut self, agent_name: String, status: String) {
        self.agent_statuses.insert(agent_name, status);
    }

    pub fn remove_agent(&mut self, agent_name: &str) {
        self.agent_statuses.remove(agent_name);
    }

    pub fn set_message(&mut self, message: String) {
        self.current_message = message;
    }

    pub fn set_connection_count(&mut self, count: usize) {
        self.connection_count = count;
    }

    fn update_memory_usage(&mut self) {
        // Simple memory usage tracking
        // In a real implementation, you might use a proper system info crate
        self.memory_usage = Some(get_process_memory().unwrap_or(0));
    }
}

#[cfg(unix)]
fn get_process_memory() -> Option<u64> {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let kb: u64 = parts[1].parse().ok()?;
                return Some(kb * 1024); // Convert to bytes
            }
        }
    }
    None
}

#[cfg(not(unix))]
fn get_process_memory() -> Option<u64> {
    // Fallback for non-Unix systems
    None
}
