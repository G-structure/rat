use agent_client_protocol as acp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SessionId;
use crate::utils::diff::DiffGenerator;

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
    ToolCallUpdate {
        update: acp::ToolCallUpdate,
    },
    SessionStatus {
        status: String,
    },
    Error {
        error: String,
    },
    Plan(acp::Plan),
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
            acp::SessionUpdate::UserMessageChunk { content } => MessageContent::UserPrompt {
                content: vec![content],
            },
            acp::SessionUpdate::AgentMessageChunk { content } => {
                MessageContent::AgentMessageChunk { content }
            }
            acp::SessionUpdate::AgentThoughtChunk { content } => {
                MessageContent::AgentResponse { content }
            }
            acp::SessionUpdate::ToolCall(tool_call) => {
                // Check if this tool call contains a diff (edit operation)
                if let Some(edit_proposal) = EditProposal::from_acp_tool_call(&tool_call) {
                    MessageContent::EditProposed { edit: edit_proposal }
                } else {
                    MessageContent::ToolCall {
                        tool_call: ToolCallRequest::from_acp_tool_call(tool_call),
                    }
                }
            },
            acp::SessionUpdate::ToolCallUpdate(tool_call_update) => MessageContent::ToolCallUpdate {
                update: tool_call_update.clone(),
            },
            acp::SessionUpdate::Plan(plan) => MessageContent::Plan(plan),
            #[cfg(feature = "unstable")]
            acp::SessionUpdate::AvailableCommandsUpdate { available_commands } => {
                MessageContent::SessionStatus {
                    status: format!(
                        "Available commands updated: {} commands",
                        available_commands.len()
                    ),
                }
            }
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
    pub fn from_acp_tool_call(tool_call: &acp::ToolCall) -> Option<Self> {
        // Look for diff content in the tool call
        for content in &tool_call.content {
            if let acp::ToolCallContent::Diff { diff } = content {
                let original_content = diff.old_text.clone().unwrap_or_default();
                let proposed_content = diff.new_text.clone();

                // Generate the actual diff using our diff utility
                let diff_text = DiffGenerator::generate_diff(&original_content, &proposed_content);

                return Some(Self {
                    id: tool_call.id.0.to_string(),
                    file_path: diff.path.to_string_lossy().to_string(),
                    original_content,
                    proposed_content,
                    diff: diff_text,
                    description: Some(tool_call.title.clone()),
                });
            }
        }
        None
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