
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use super::{Message, MessageContent};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub agent_name: Option<String>,
    pub messages: VecDeque<Message>,
    pub status: SessionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub context: SessionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Processing,
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub working_directory: Option<String>,
    pub open_files: Vec<String>,
    pub environment_variables: std::collections::HashMap<String, String>,
    pub project_root: Option<String>,
}

impl Default for SessionContext {
    fn default() -> Self {
        Self {
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(String::from)),
            open_files: Vec::new(),
            environment_variables: std::collections::HashMap::new(),
            project_root: None,
        }
    }
}

impl Session {
    pub fn new(id: SessionId) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            agent_name: None,
            messages: VecDeque::new(),
            status: SessionStatus::Active,
            created_at: now,
            last_activity: now,
            context: SessionContext::default(),
        }
    }

    pub fn with_agent(id: SessionId, agent_name: String) -> Self {
        let mut session = Self::new(id);
        session.agent_name = Some(agent_name);
        session
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);
        self.last_activity = chrono::Utc::now();

        // Keep only the last 1000 messages to prevent memory bloat
        if self.messages.len() > 1000 {
            self.messages.pop_front();
        }
    }

    pub fn get_messages(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter()
    }

    pub fn get_recent_messages(&self, count: usize) -> impl Iterator<Item = &Message> {
        let start = self.messages.len().saturating_sub(count);
        self.messages.iter().skip(start)
    }

    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status;
        self.last_activity = chrono::Utc::now();
    }

    pub fn update_context(&mut self, context: SessionContext) {
        self.context = context;
        self.last_activity = chrono::Utc::now();
    }

    pub fn add_open_file(&mut self, file_path: String) {
        if !self.context.open_files.contains(&file_path) {
            self.context.open_files.push(file_path);
        }
        self.last_activity = chrono::Utc::now();
    }

    pub fn remove_open_file(&mut self, file_path: &str) {
        self.context.open_files.retain(|f| f != file_path);
        self.last_activity = chrono::Utc::now();
    }

    pub fn set_working_directory(&mut self, dir: String) {
        self.context.working_directory = Some(dir);
        self.last_activity = chrono::Utc::now();
    }

    pub fn set_project_root(&mut self, root: String) {
        self.context.project_root = Some(root);
        self.last_activity = chrono::Utc::now();
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            SessionStatus::Active | SessionStatus::Processing
        )
    }

    pub fn duration_since_last_activity(&self) -> chrono::Duration {
        chrono::Utc::now() - self.last_activity
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn find_message_by_id(&self, id: &str) -> Option<&Message> {
        self.messages.iter().find(|msg| msg.id == id)
    }

    pub fn get_conversation_history(&self) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|msg| {
                matches!(
                    msg.content,
                    MessageContent::UserPrompt { .. }
                        | MessageContent::AgentResponse { .. }
                        | MessageContent::AgentMessageChunk { .. }
                )
            })
            .collect()
    }
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "Active"),
            SessionStatus::Idle => write!(f, "Idle"),
            SessionStatus::Processing => write!(f, "Processing"),
            SessionStatus::Error(e) => write!(f, "Error: {}", e),
            SessionStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}
