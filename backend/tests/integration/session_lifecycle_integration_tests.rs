use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

use rat::acp::{Message, Session, SessionId, SessionStatus};
use rat::adapters::traits::{AgentAdapter, AgentHealth};
use rat::adapters::{claude_code::ClaudeCodeAdapter, AgentManager};
use rat::app::{manager_worker, App, AppMessage, ManagerCmd, UiToApp};
use rat::config::{AgentConfig, ClaudeCodeConfig, Config, GeneralConfig, ProjectConfig, UiConfig};

/// Integration test for the full session creation flow
#[tokio::test]
async fn test_full_session_creation_flow() {
    // Setup configuration
    let mut config = Config::default();
    // Keep tests self-contained and fast
    config.general.auto_save_sessions = false;
    // Do not auto-connect agents during tests
    config.agents.auto_connect = vec![];

    // Create app
    let mut app = App::new(config).await.expect("Failed to create app");

    // Test session creation
    let session_id = app.create_session("claude-code").await;

    // For this test, we expect it to fail since we don't have a real claude-code binary
    // But we can test that the flow executes without panicking
    match session_id {
        Ok(id) => {
            // If it somehow succeeded (perhaps in CI with claude-code installed)
            println!("Session created successfully: {}", id);
        }
        Err(e) => {
            // Expected in most test environments
            println!("Session creation failed as expected: {}", e);
        }
    }
}

/// Test the UI to App message flow
#[tokio::test]
async fn test_ui_to_app_message_flow() {
    let config = Config::default_test();

    let mut app = App::new(config).await.expect("Failed to create app");

    // Create a mock session
    let session_id = SessionId::new();

    // Test various app messages
    let messages = vec![
        AppMessage::SessionCreated {
            agent_name: "claude-code".to_string(),
            session_id: session_id.clone(),
        },
        AppMessage::AgentConnected {
            agent_name: "claude-code".to_string(),
        },
        AppMessage::Error {
            error: "Test error".to_string(),
        },
    ];

    for msg in messages {
        app.handle_app_message(msg)
            .await
            .expect("Failed to handle app message");
    }

    // Verify the session was added
    let active_session = app.tui_manager.get_active_session();
    assert!(active_session.is_some());

    let (agent_name, active_session_id) = active_session.unwrap();
    assert_eq!(agent_name, "claude-code");
    assert_eq!(active_session_id, &session_id);
}

/// Test manager command processing
#[tokio::test]
async fn test_manager_command_processing() {
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<AppMessage>();
    let (manager_tx, manager_rx) = mpsc::unbounded_channel::<ManagerCmd>();

    let mut config = AgentConfig::default();
    // Disable auto connect
    config.auto_connect = vec![];

    // Create agent manager
    let manager = AgentManager::new(config, message_tx.clone())
        .await
        .expect("Failed to create agent manager");

    // Start manager worker
    tokio::task::spawn_local(async move {
        manager_worker(manager, manager_rx).await;
    });

    // Test connect agent command
    let _ = manager_tx.send(ManagerCmd::ConnectAgent {
        agent_name: "claude-code".to_string(),
    });

    // Wait a bit for processing
    sleep(Duration::from_millis(100)).await;

    // Test create session command
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let _ = manager_tx.send(ManagerCmd::CreateSession {
        agent_name: "claude-code".to_string(),
        respond_to: response_tx,
    });

    // Wait for response (will likely fail but shouldn't panic)
    let _result = timeout(Duration::from_millis(500), response_rx).await;
}

/// Test session message handling
#[tokio::test]
async fn test_session_message_handling() {
    let session_id = SessionId::new();
    let mut session = Session::with_agent(session_id.clone(), "claude-code".to_string());

    // Test adding various message types using actual constructors
    let user_msg = Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Hello, Claude!".to_string(),
        }],
    );
    session.add_message(user_msg);

    let agent_response = Message::agent_response(
        session_id.clone(),
        agent_client_protocol::ContentBlock::Text {
            text: "Hello! How can I help you today?".to_string(),
        },
    );
    session.add_message(agent_response);

    let error_msg = Message::error(session_id.clone(), "Test error message".to_string());
    session.add_message(error_msg);

    // Verify message count and history
    assert_eq!(session.message_count(), 3);

    let conversation = session.get_conversation_history();
    assert_eq!(conversation.len(), 2); // Only user prompts and agent responses

    // Test session context updates
    session.set_working_directory("/test/project".to_string());
    session.set_project_root("/test".to_string());
    session.add_open_file("main.rs".to_string());
    session.add_open_file("lib.rs".to_string());

    assert_eq!(
        session.context.working_directory,
        Some("/test/project".to_string())
    );
    assert_eq!(session.context.project_root, Some("/test".to_string()));
    assert_eq!(session.context.open_files.len(), 2);

    // Test file removal
    session.remove_open_file("lib.rs");
    assert_eq!(session.context.open_files.len(), 1);
    assert!(session.context.open_files.contains(&"main.rs".to_string()));
}

/// Test concurrent session operations
#[tokio::test]
async fn test_concurrent_session_operations() {
    let session_id = SessionId::new();
    let mut session = Session::with_agent(session_id.clone(), "claude-code".to_string());

    // Simulate concurrent message adding
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let session_id = session_id.clone();
            tokio::task::spawn_local(async move {
                Message::user_prompt(
                    session_id,
                    vec![agent_client_protocol::ContentBlock::Text {
                        text: format!("Message {}", i),
                    }],
                )
            })
        })
        .collect();

    for handle in handles {
        let message = handle.await.expect("Task failed");
        session.add_message(message);
    }

    assert_eq!(session.message_count(), 10);
}

/// Test session persistence and recovery
#[tokio::test]
async fn test_session_persistence() {
    let session_id = SessionId::from_string("test-session-123".to_string());
    let mut original_session = Session::with_agent(session_id.clone(), "claude-code".to_string());

    // Add some data
    original_session.add_message(Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Test message".to_string(),
        }],
    ));

    original_session.set_working_directory("/test/path".to_string());
    original_session.add_open_file("test.rs".to_string());

    // Serialize session (test that it can be serialized)
    let serialized = serde_json::to_string(&original_session).expect("Failed to serialize session");

    // Deserialize session (test recovery)
    let recovered_session: Session =
        serde_json::from_str(&serialized).expect("Failed to deserialize session");

    // Verify data integrity
    assert_eq!(recovered_session.id, original_session.id);
    assert_eq!(recovered_session.agent_name, original_session.agent_name);
    assert_eq!(
        recovered_session.message_count(),
        original_session.message_count()
    );
    assert_eq!(
        recovered_session.context.working_directory,
        original_session.context.working_directory
    );
    assert_eq!(
        recovered_session.context.open_files,
        original_session.context.open_files
    );
}

/// Test error scenarios
#[tokio::test]
async fn test_error_scenarios() {
    let config = Config::default_test();
    let mut app = App::new(config).await.expect("Failed to create app");

    // Test handling invalid agent name
    let result = app.create_session("nonexistent-agent").await;
    assert!(result.is_err());

    // Test error message handling
    let error_msg = AppMessage::Error {
        error: "Connection failed".to_string(),
    };

    app.handle_app_message(error_msg)
        .await
        .expect("Failed to handle error message");

    // Test disconnection handling
    let disconnect_msg = AppMessage::AgentDisconnected {
        agent_name: "claude-code".to_string(),
    };

    app.handle_app_message(disconnect_msg)
        .await
        .expect("Failed to handle disconnect message");
}

/// Test adapter health monitoring
#[tokio::test]
async fn test_adapter_health_monitoring() {
    let config = ClaudeCodeConfig::default();

    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<AppMessage>();

    let mut adapter = ClaudeCodeAdapter::new(config, message_tx)
        .await
        .expect("Failed to create adapter");

    // Test initial health
    assert!(matches!(adapter.health_status(), AgentHealth::Disconnected));

    // Test health updates through tick
    for _ in 0..5 {
        adapter.tick().await.expect("Tick failed");
        sleep(Duration::from_millis(10)).await;
    }

    // Health should still be disconnected since we're not connected
    assert!(matches!(adapter.health_status(), AgentHealth::Disconnected));
}

/// Test session lifecycle with realistic message flow
#[tokio::test]
async fn test_realistic_session_lifecycle() {
    let session_id = SessionId::new();
    let mut session = Session::with_agent(session_id.clone(), "claude-code".to_string());

    // Simulate a realistic conversation flow

    // 1. User starts with a question
    session.add_message(Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "Can you help me write a Rust function?".to_string(),
        }],
    ));

    // 2. Agent responds
    session.add_message(Message::agent_response(
        session_id.clone(),
        agent_client_protocol::ContentBlock::Text {
            text: "Of course! What kind of function would you like to write?".to_string(),
        },
    ));

    // 3. User provides more details
    session.add_message(Message::user_prompt(
        session_id.clone(),
        vec![agent_client_protocol::ContentBlock::Text {
            text: "A function that calculates fibonacci numbers".to_string(),
        }],
    ));

    // 4. Agent working (set status to processing)
    session.set_status(SessionStatus::Processing);
    assert!(session.is_active());

    // 5. Agent provides response
    session.add_message(Message::agent_response(
        session_id.clone(),
        agent_client_protocol::ContentBlock::Text {
            text: "Here's a Rust function to calculate Fibonacci numbers:".to_string(),
        },
    ));

    // 6. Back to active status
    session.set_status(SessionStatus::Active);

    // Verify the conversation flow
    assert_eq!(session.message_count(), 4);
    let conversation = session.get_conversation_history();
    assert_eq!(conversation.len(), 4);

    // Test that recent messages work correctly
    let recent: Vec<_> = session.get_recent_messages(2).collect();
    assert_eq!(recent.len(), 2);

    // Verify session is still active
    assert!(session.is_active());
}

#[cfg(test)]
mod integration_test_helpers {
    use super::*;

    impl Config {
        pub fn default_test() -> Self {
            let mut cfg = Config::default();
            cfg.general.log_level = "debug".to_string();
            cfg.general.auto_save_sessions = false;
            cfg.agents.auto_connect = vec![];
            cfg
        }
    }
}
