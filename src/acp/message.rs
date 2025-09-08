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
        content: Vec<acp::ContentBlock>,
    },
    AgentResponse {
        content: acp::ContentBlock,
    },
    AgentMessageChunk {
        content: acp::ContentBlock,
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
            acp::SessionUpdate::ToolCall(tool_call) => {
                if tool_call.kind == acp::ToolKind::Edit {
                    if let Some(edit) = EditProposal::from_acp_tool_call(tool_call) {
                        MessageContent::EditProposed { edit }
                    } else {
                        MessageContent::SessionStatus {
                            status: "Invalid edit proposal received ".to_string(),
                        }
                    }
                } else {
                    MessageContent::ToolCall {
                        tool_call: ToolCallRequest::from_acp_tool_call(tool_call),
                    }
                }
            }
            acp::SessionUpdate::ToolCallUpdate(tool_call_update) => {
                // For now, we'll just log the update.
                // In the future, we might want to handle this more gracefully.
                MessageContent::SessionStatus {
                    status: format!("Tool call update: {:?}", tool_call_update),
                }
            }
            _ => MessageContent::SessionStatus {
                status: "Unknown update received ".to_string(),
            },
        };

        Self::new(session_id, content)
    }

    pub fn user_prompt(session_id: SessionId, content: Vec<acp::ContentBlock>) -> Self {
        Self::new(session_id, MessageContent::UserPrompt { content })
    }

    pub fn agent_response(session_id: SessionId, content: acp::ContentBlock) -> Self {
        Self::new(session_id, MessageContent::AgentResponse { content })
    }

    pub fn error(session_id: SessionId, error: String) -> Self {
        Self::new(session_id, MessageContent::Error { error })
    }
}

impl EditProposal {
    fn from_acp_tool_call(tool_call: acp::ToolCall) -> Option<Self> {
        if let Some(acp::ToolCallContent::Diff { diff }) = tool_call.content.get(0) {
            Some(Self {
                id: tool_call.id.0.to_string(),
                file_path: diff.path.to_string_lossy().to_string(),
                original_content: diff.old_text.clone().unwrap_or_default(),
                proposed_content: diff.new_text.clone(),
                diff: String::new(), // Would be computed
                description: Some(tool_call.title),
            })
        } else {
            None
        }
    }
}

impl ToolCallRequest {
    fn from_acp_tool_call(tool_call: acp::ToolCall) -> Self {
        Self {
            id: tool_call.id.0.to_string(),
            tool_name: format!("{:?}", tool_call.kind),
            parameters: tool_call.raw_input.unwrap_or_default(),
            requires_permission: true, // Default to requiring permission
        }
    }
}
