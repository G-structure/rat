use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

use rat::adapters::traits::{AgentAdapter, AgentCapabilities, AgentHealth};
use rat::adapters::manager::AgentManager;
use rat::acp::{Session, SessionId};
use rat::app::AppMessage;
use rat::config::agent::AgentConfig;

// Bring in ManagerCmd and manager_worker from app.rs
use rat::app::{manager_worker, ManagerCmd};

struct MockAdapter {
    name: String,
    connected: bool,
    sessions: HashMap<SessionId, Session>,
}

#[async_trait::async_trait(?Send)]
impl AgentAdapter for MockAdapter {
    fn name(&self) -> &str { &self.name }
    fn is_connected(&self) -> bool { self.connected }
    async fn start(&mut self) -> Result<()> { self.connected = true; Ok(()) }
    async fn stop(&mut self) -> Result<()> { self.connected = false; Ok(()) }
    async fn create_session(&mut self) -> Result<SessionId> {
        if !self.connected { return Err(anyhow::anyhow!("not connected")); }
        let sid = SessionId(format!("session-{}", self.sessions.len() + 1));
        self.sessions.insert(sid.clone(), Session::new(sid.clone()));
        Ok(sid)
    }
    async fn send_message(&mut self, _session_id: &SessionId, _content: String) -> Result<()> {
        if !self.connected { return Err(anyhow::anyhow!("not connected")); }
        Ok(())
    }
    fn get_session_ids(&self) -> Vec<SessionId> { self.sessions.keys().cloned().collect() }
    fn get_session(&self, session_id: &SessionId) -> Option<&Session> { self.sessions.get(session_id) }
    fn get_session_mut(&mut self, session_id: &SessionId) -> Option<&mut Session> { self.sessions.get_mut(session_id) }
    async fn tick(&mut self) -> Result<()> { Ok(()) }
    fn health_status(&self) -> AgentHealth { AgentHealth::Healthy }
    fn capabilities(&self) -> AgentCapabilities { AgentCapabilities::default() }
}

#[tokio::test(flavor = "current_thread")]
async fn manager_worker_connect_and_create_session() -> Result<()> {
    // Capture app messages
    let (app_tx, mut app_rx) = mpsc::unbounded_channel::<AppMessage>();

    // Start with disabled agents to avoid external processes
    let mut config = AgentConfig::default();
    config.claude_code.enabled = false;
    config.gemini.enabled = false;

    let mut manager = AgentManager::new(config, app_tx).await?;
    manager.register_agent(
        "mock".to_string(),
        Box::new(MockAdapter { name: "mock".to_string(), connected: false, sessions: HashMap::new() }),
    );

    // Start worker inside LocalSet
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<ManagerCmd>();
    let local = tokio::task::LocalSet::new();
    local.spawn_local(manager_worker(manager, cmd_rx));

    // Connect agent
    cmd_tx.send(ManagerCmd::ConnectAgent { agent_name: "mock".to_string() }).unwrap();

    // Create session
    let (resp_tx, resp_rx) = oneshot::channel();
    cmd_tx.send(ManagerCmd::CreateSession { agent_name: "mock".to_string(), respond_to: resp_tx }).unwrap();

    // Drive the local tasks a bit
    local.run_until(async {
        // Await session id
        let sid = resp_rx.await.expect("manager response").expect("session created");
        assert!(sid.0.starts_with("session-"));

        // Expect a SessionCreated message to UI
        // Drain until we find it or timeout
        let mut found = false;
        for _ in 0..10 {
            if let Ok(msg) = app_rx.try_recv() {
                if let AppMessage::SessionCreated { agent_name, session_id } = msg {
                    assert_eq!(agent_name, "mock");
                    assert_eq!(session_id.0, sid.0);
                    found = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(found, "expected SessionCreated message");
    }).await;

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn tui_create_new_session_emits_command() -> Result<()> {
    use rat::config::UiConfig;
    use rat::ui::TuiManager;

    let ui_config = UiConfig::default();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tui = TuiManager::new(ui_config, tx)?;

    // Call and ensure a CreateSession is sent
    tui.create_new_session().await?;
    match rx.try_recv() {
        Ok(rat::app::UiToApp::CreateSession { agent_name, .. }) => {
            assert_eq!(agent_name, "claude-code");
        }
        _ => panic!("unexpected message"),
    }
    Ok(())
}
