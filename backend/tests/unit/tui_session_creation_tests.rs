use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

use rat::acp::{Session, SessionId, SessionStatus};
use rat::app::{App, AppMessage, ManagerCmd, UiToApp};
use rat::config::{Config, UiConfig};
use rat::ui::TuiManager;

#[tokio::test]
async fn test_tui_new_session_key_press_flow() {
    // Setup test configuration
    let config = Config::default_test();
    let ui_config = UiConfig::default();

    // Create channels for testing
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel::<UiToApp>();
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<AppMessage>();

    // Create TUI manager
    let mut tui_manager = TuiManager::new(ui_config, ui_tx).expect("Failed to create TUI manager");

    // Simulate 'n' key press (create new session)
    let key_event = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('n'),
        crossterm::event::KeyModifiers::NONE,
    );

    // Handle the key event
    tui_manager
        .handle_key_event(key_event)
        .await
        .expect("Failed to handle key event");

    // Verify that a CreateSession command was sent
    let ui_command = timeout(Duration::from_millis(100), ui_rx.recv())
        .await
        .expect("Timeout waiting for UI command")
        .expect("No UI command received");

    match ui_command {
        UiToApp::CreateSession {
            agent_name,
            respond_to: _,
        } => {
            assert_eq!(agent_name, "claude-code");
        }
        _ => panic!("Expected CreateSession command, got {:?}", ui_command),
    }

    // Verify status bar update
    assert_eq!(tui_manager.get_active_session(), None);
}

#[tokio::test]
async fn test_app_create_session_message_flow() {
    // Setup test configuration
    let config = Config::default_test();

    // Create test app
    let mut app = App::new(config).await.expect("Failed to create app");

    // Create a session ID for testing
    let session_id = SessionId::new();

    // Simulate SessionCreated message
    let session_created_msg = AppMessage::SessionCreated {
        agent_name: "claude-code".to_string(),
        session_id: session_id.clone(),
    };

    // Handle the message
    app.handle_app_message(session_created_msg)
        .await
        .expect("Failed to handle session created message");

    // Verify that a new tab was created with the session
    let active_session = app.tui_manager.get_active_session();
    assert!(active_session.is_some());

    let (agent_name, active_session_id) = active_session.unwrap();
    assert_eq!(agent_name, "claude-code");
    assert_eq!(active_session_id, &session_id);
}

#[tokio::test]
async fn test_manager_cmd_create_session() {
    // Setup test channels
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<AppMessage>();
    let (manager_tx, mut manager_rx) = mpsc::unbounded_channel::<ManagerCmd>();

    // Create a oneshot channel for the response
    let (response_tx, response_rx) = oneshot::channel();

    // Send create session command
    let cmd = ManagerCmd::CreateSession {
        agent_name: "claude-code".to_string(),
        respond_to: response_tx,
    };

    manager_tx
        .send(cmd)
        .expect("Failed to send manager command");

    // Verify the command was received
    let received_cmd = timeout(Duration::from_millis(100), manager_rx.recv())
        .await
        .expect("Timeout waiting for manager command")
        .expect("No manager command received");

    match received_cmd {
        ManagerCmd::CreateSession {
            agent_name,
            respond_to: _,
        } => {
            assert_eq!(agent_name, "claude-code");
        }
        _ => panic!("Expected CreateSession command, got {:?}", received_cmd),
    }
}

#[tokio::test]
async fn test_session_creation_with_default_agent() {
    // Test that creating a session with default agent sets up correctly
    let session_id = SessionId::new();
    let mut session = Session::with_agent(session_id.clone(), "claude-code".to_string());

    assert_eq!(session.id, session_id);
    assert_eq!(session.agent_name, Some("claude-code".to_string()));
    assert!(matches!(session.status, SessionStatus::Active));
    assert!(session.is_active());
    assert_eq!(session.message_count(), 0);
}

#[tokio::test]
async fn test_tui_session_status_updates() {
    let config = Config::default_test();
    let ui_config = UiConfig::default();
    let (ui_tx, _ui_rx) = mpsc::unbounded_channel::<UiToApp>();

    let mut tui_manager = TuiManager::new(ui_config, ui_tx).expect("Failed to create TUI manager");

    // Test setting agent status
    tui_manager.set_agent_status("claude-code", "Creating session...".to_string());

    // Test showing error
    tui_manager.show_error("Test error message".to_string());

    // Test adding session
    let session_id = SessionId::new();
    tui_manager
        .add_session("claude-code", session_id.clone())
        .expect("Failed to add session");

    // Verify session was added
    let active_session = tui_manager.get_active_session();
    assert!(active_session.is_some());

    let (agent_name, active_session_id) = active_session.unwrap();
    assert_eq!(agent_name, "claude-code");
    assert_eq!(active_session_id, &session_id);
}

#[tokio::test]
async fn test_tui_multiple_sessions() {
    let config = Config::default_test();
    let ui_config = UiConfig::default();
    let (ui_tx, _ui_rx) = mpsc::unbounded_channel::<UiToApp>();

    let mut tui_manager = TuiManager::new(ui_config, ui_tx).expect("Failed to create TUI manager");

    // Create multiple sessions
    let session1 = SessionId::new();
    let session2 = SessionId::new();

    tui_manager
        .add_session("claude-code", session1.clone())
        .expect("Failed to add first session");
    tui_manager
        .add_session("claude-code", session2.clone())
        .expect("Failed to add second session");

    // Verify active session is the latest one
    let active_session = tui_manager.get_active_session();
    assert!(active_session.is_some());

    let (_agent_name, active_session_id) = active_session.unwrap();
    assert_eq!(active_session_id, &session2);

    // Test tab navigation
    tui_manager.prev_tab();
    let active_session = tui_manager.get_active_session();
    let (_agent_name, active_session_id) = active_session.unwrap();
    assert_eq!(active_session_id, &session1);

    tui_manager.next_tab();
    let active_session = tui_manager.get_active_session();
    let (_agent_name, active_session_id) = active_session.unwrap();
    assert_eq!(active_session_id, &session2);
}

#[tokio::test]
async fn test_session_error_handling() {
    let config = Config::default_test();
    let ui_config = UiConfig::default();
    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel::<UiToApp>();

    let mut tui_manager = TuiManager::new(ui_config, ui_tx).expect("Failed to create TUI manager");

    // Simulate 'n' key press
    let key_event = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('n'),
        crossterm::event::KeyModifiers::NONE,
    );

    tui_manager
        .handle_key_event(key_event)
        .await
        .expect("Failed to handle key event");

    // Consume the CreateSession command
    let _ui_command = ui_rx.recv().await;

    // Test error handling
    tui_manager.show_error("Failed to create session: Agent not available".to_string());

    // The error should be displayed but session creation shouldn't have succeeded
    assert_eq!(tui_manager.get_active_session(), None);
}

#[cfg(test)]
mod config_helpers {
    use rat::config::{AgentConfig, ClaudeCodeConfig, Config, GeneralConfig};

    impl Config {
        pub fn default_test() -> Self {
            let mut cfg = Config::default();
            cfg.general.log_level = "debug".to_string();
            cfg.general.auto_save_sessions = false;
            cfg.agents.auto_connect = vec!["claude-code".to_string()];
            cfg
        }
    }
}
