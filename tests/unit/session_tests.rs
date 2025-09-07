use rat::acp::{Session, SessionId, SessionStatus};

#[test]
fn test_session_creation() {
    let session_id = SessionId::new();
    let session = Session::new(session_id.clone());

    assert_eq!(session.id, session_id);
    assert!(matches!(session.status, SessionStatus::Active));
    assert_eq!(session.message_count(), 0);
    assert!(session.is_active());
}

#[test]
fn test_session_with_agent() {
    let session_id = SessionId::new();
    let agent_name = "test-agent".to_string();
    let session = Session::with_agent(session_id.clone(), agent_name.clone());

    assert_eq!(session.id, session_id);
    assert_eq!(session.agent_name, Some(agent_name));
}

#[test]
fn test_session_id_display() {
    let session_id = SessionId::from_string("test-session-123".to_string());
    assert_eq!(format!("{}", session_id), "test-session-123");
}

#[test]
fn test_session_status_display() {
    assert_eq!(format!("{}", SessionStatus::Active), "Active");
    assert_eq!(format!("{}", SessionStatus::Idle), "Idle");
    assert_eq!(format!("{}", SessionStatus::Processing), "Processing");
    assert_eq!(format!("{}", SessionStatus::Disconnected), "Disconnected");

    let error_status = SessionStatus::Error("Test error".to_string());
    assert_eq!(format!("{}", error_status), "Error: Test error");
}
