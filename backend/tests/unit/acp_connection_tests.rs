use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

use rat::acp::{AcpClient, Message, MessageContent, Session, SessionId};
use rat::adapters::claude_code::ClaudeCodeAdapter;
use rat::adapters::traits::{AgentAdapter, AgentHealth};
use rat::app::AppMessage;
use rat::config::agent::ClaudeCodeConfig;

#[tokio::test]
async fn test_acp_client_creation() {
    let (message_tx, _message_rx) = mpsc::unbounded_channel::<AppMessage>();

    // Create ACP client with mock command
    let client = AcpClient::new(
        "claude-code",
        "/usr/bin/echo", // Mock command that exists
        vec!["test".to_string()],
        Some(HashMap::new()),
        message_tx,
        None,
    );

    assert_eq!(client.name(), "claude-code");
    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_claude_code_adapter_creation() {
    let config = ClaudeCodeConfig::default();

    let (message_tx, _message_rx) = mpsc::unbounded_channel::<AppMessage>();

    let adapter = ClaudeCodeAdapter::new(config, message_tx)
        .await
        .expect("Failed to create Claude Code adapter");

    assert_eq!(adapter.name(), "claude-code");
    assert!(!adapter.is_connected());
    assert!(matches!(adapter.health_status(), AgentHealth::Disconnected));
}

#[tokio::test]
async fn test_adapter_lifecycle() {
    let config = ClaudeCodeConfig::default();

    let (message_tx, _message_rx) = mpsc::unbounded_channel::<AppMessage>();

    let mut adapter = ClaudeCodeAdapter::new(config, message_tx)
        .await
        .expect("Failed to create Claude Code adapter");

    // Initially disconnected
    assert!(!adapter.is_connected());
    assert_eq!(adapter.get_session_ids().len(), 0);

    // Note: We can't actually start the adapter in tests without a real claude-code binary
    // But we can test the interface and state management

    // Test tick without connection (should not fail)
    adapter.tick().await.expect("Tick should not fail");

    // Test capabilities
    let capabilities = adapter.capabilities();
    assert!(capabilities.supports_streaming);
    assert!(capabilities.supports_tools);
}

#[tokio::test]
async fn test_session_management() {
    let session_id = SessionId::new();
    let mut session = Session::with_agent(session_id.clone(), "claude-code".to_string());

    // Test initial state
    assert_eq!(session.id, session_id);
    assert_eq!(session.agent_name, Some("claude-code".to_string()));
    assert!(session.is_active());
    assert_eq!(session.message_count(), 0);

    // Test adding messages using the actual Message constructors
    let message = Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Hello".to_string(),
        }],
    );

    session.add_message(message);
    assert_eq!(session.message_count(), 1);

    // Test context updates
    session.set_working_directory("/test/path".to_string());
    assert_eq!(
        session.context.working_directory,
        Some("/test/path".to_string())
    );

    session.add_open_file("test.rs".to_string());
    assert!(session.context.open_files.contains(&"test.rs".to_string()));

    session.remove_open_file("test.rs");
    assert!(!session.context.open_files.contains(&"test.rs".to_string()));
}

#[tokio::test]
async fn test_session_status_transitions() {
    let session_id = SessionId::new();
    let mut session = Session::new(session_id);

    // Test status transitions
    use rat::acp::SessionStatus;

    assert!(matches!(session.status, SessionStatus::Active));
    assert!(session.is_active());

    session.set_status(SessionStatus::Processing);
    assert!(matches!(session.status, SessionStatus::Processing));
    assert!(session.is_active()); // Processing is considered active

    session.set_status(SessionStatus::Idle);
    assert!(matches!(session.status, SessionStatus::Idle));
    assert!(!session.is_active());

    session.set_status(SessionStatus::Error("Test error".to_string()));
    assert!(!session.is_active());

    session.set_status(SessionStatus::Disconnected);
    assert!(!session.is_active());
}

#[tokio::test]
async fn test_message_history_management() {
    let session_id = SessionId::new();
    let mut session = Session::new(session_id.clone());

    // Add multiple messages using actual constructors
    for i in 0..5 {
        let message = Message::user_prompt(
            session_id.clone(),
            vec![agent_client_protocol::ContentBlock::Text {
                text: format!("Message {}", i),
            }],
        );
        session.add_message(message);
    }

    assert_eq!(session.message_count(), 5);

    // Test getting recent messages
    let recent_messages: Vec<_> = session.get_recent_messages(3).collect();
    assert_eq!(recent_messages.len(), 3);

    // Test conversation history filtering
    let conversation = session.get_conversation_history();
    assert_eq!(conversation.len(), 5); // All are user prompts

    // Test message limit (add many messages to test the 1000 limit)
    for i in 5..1005 {
        let message = Message::user_prompt(
            session_id.clone(),
            vec![agent_client_protocol::ContentBlock::Text {
                text: format!("Message {}", i),
            }],
        );
        session.add_message(message);
    }

    // Should be capped at 1000
    assert_eq!(session.message_count(), 1000);
}

#[tokio::test]
async fn test_acp_message_types() {
    let session_id = SessionId::new();

    // Test user prompt
    let user_message = Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Hello".to_string(),
        }],
    );
    assert!(matches!(
        user_message.content,
        MessageContent::UserPrompt { .. }
    ));

    // Test agent response
    let agent_message = Message::agent_response(
        session_id.clone(),
        agent_client_protocol::ContentBlock::Text {
            text: "Hello back!".to_string(),
        },
    );
    assert!(matches!(
        agent_message.content,
        MessageContent::AgentResponse { .. }
    ));

    // Test error message
    let error_message = Message::error(session_id.clone(), "Test error".to_string());
    assert!(matches!(
        error_message.content,
        MessageContent::Error { .. }
    ));
}

#[tokio::test]
async fn test_session_context_defaults() {
    use rat::acp::SessionContext;

    let context = SessionContext::default();

    // Should have current directory set
    assert!(context.working_directory.is_some());
    assert!(context.open_files.is_empty());
    assert!(context.environment_variables.is_empty());
    assert!(context.project_root.is_none());
}

#[tokio::test]
async fn test_adapter_health_monitoring() {
    let config = ClaudeCodeConfig::default();

    let (message_tx, _message_rx) = mpsc::unbounded_channel::<AppMessage>();

    let mut adapter = ClaudeCodeAdapter::new(config, message_tx)
        .await
        .expect("Failed to create Claude Code adapter");

    // Initial health should be disconnected
    assert!(matches!(adapter.health_status(), AgentHealth::Disconnected));

    // Test tick updates health
    adapter.tick().await.expect("Tick should not fail");

    // Health should still be disconnected since we're not connected
    assert!(matches!(adapter.health_status(), AgentHealth::Disconnected));
}

#[tokio::test]
async fn test_session_duration_tracking() {
    let session_id = SessionId::new();
    let mut session = Session::new(session_id.clone());

    // Test that duration tracking works
    let duration = session.duration_since_last_activity();
    assert!(duration.num_milliseconds() >= 0);

    // Add a message to update last activity
    let message = Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Test".to_string(),
        }],
    );
    session.add_message(message);

    // Duration should be small (just created)
    let new_duration = session.duration_since_last_activity();
    assert!(new_duration.num_milliseconds() < 1000); // Less than 1 second
}

#[tokio::test]
async fn test_session_message_search() {
    let session_id = SessionId::new();
    let mut session = Session::new(session_id.clone());

    // Add a message
    let message = Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Test message".to_string(),
        }],
    );
    let message_id = message.id.clone();
    session.add_message(message);

    // Test finding message by ID
    let found_message = session.find_message_by_id(&message_id);
    assert!(found_message.is_some());
    assert_eq!(found_message.unwrap().id, message_id);

    // Test searching for non-existent message
    let not_found = session.find_message_by_id("nonexistent");
    assert!(not_found.is_none());
}
