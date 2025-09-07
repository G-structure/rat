use agent_client_protocol as acp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: SessionId,
    pub content: MessageContent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    UserPrompt {
        content: Vec<acp::Content>,
    },
    AgentResponse {
        content: acp::Content,
    },
    AgentMessageChunk {
        content: acp::Content,
    },
    EditProposed {
        edit: EditProposal,
    },
    EditAccepted {
        edit_id: String,
    },
    EditRejected {
        edit_id: String,
    },
    ToolCall {
        tool_call: ToolCallRequest,
    },
    ToolResult {
        tool_call_id: String,
        result: String,
    },
    SessionStatus {
        status: String,
    },
    Error {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditProposal {
    pub id: String,
    pub file_path: String,
    pub original_content: String,
    pub proposed_content: String,
    pub diff: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub requires_permission: bool,
}

impl Message {
    pub fn new(session_id: SessionId, content: MessageContent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id,
            content,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn from_session_update(session_id: SessionId, update: acp::SessionUpdate) -> Self {
        let content = match update {
            acp::SessionUpdate::AgentMessageChunk { content } => {
                MessageContent::AgentMessageChunk { content }
            }
            acp::SessionUpdate::EditProposed { edit } => MessageContent::EditProposed {
                edit: EditProposal::from_acp_edit(edit),
            },
            acp::SessionUpdate::ToolCallRequested { tool_call } => MessageContent::ToolCall {
                tool_call: ToolCallRequest::from_acp_tool_call(tool_call),
            },
            _ => MessageContent::SessionStatus {
                status: "Unknown update received".to_string(),
            },
        };

        Self::new(session_id, content)
    }

    pub fn user_prompt(session_id: SessionId, content: Vec<acp::Content>) -> Self {
        Self::new(session_id, MessageContent::UserPrompt { content })
    }

    pub fn agent_response(session_id: SessionId, content: acp::Content) -> Self {
        Self::new(session_id, MessageContent::AgentResponse { content })
    }

    pub fn error(session_id: SessionId, error: String) -> Self {
        Self::new(session_id, MessageContent::Error { error })
    }
}

impl EditProposal {
    fn from_acp_edit(edit: acp::Edit) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file_path: edit.path.unwrap_or_else(|| "unknown".to_string()),
            original_content: String::new(), // Would be populated from file system
            proposed_content: edit.new_text.unwrap_or_default(),
            diff: String::new(), // Would be computed
            description: edit.description,
        }
    }
}

impl ToolCallRequest {
    fn from_acp_tool_call(tool_call: acp::ToolCall) -> Self {
        Self {
            id: tool_call.call_id.clone(),
            tool_name: tool_call.name,
            parameters: tool_call.arguments,
            requires_permission: true, // Default to requiring permission
        }
    }
}
