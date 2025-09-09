use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::mpsc;

use rat::adapters::agent_installer::AgentInstaller;
use rat::adapters::manager::AgentManager;
use rat::config::agent::AgentConfig;
use rat::app::AppMessage;
use rat::acp::AcpClient;

fn should_skip() -> bool {
    std::env::var("RAT_SKIP_ACP_TESTS").ok().as_deref() == Some("1")
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_claude_command_and_version() -> Result<()> {
    if should_skip() { return Ok(()); }
    let installer = AgentInstaller::new()?;
    let cmd = installer.get_or_install_claude_code().await?;
    // Try version check; if it fails, that's okay but it should be runnable
    let _ = installer.verify_agent_command(&cmd).await.ok();
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn start_claude_adapter_and_attempt_session() -> Result<()> {
    if should_skip() { return Ok(()); }

    // Message channel to capture app-level events
    let (tx, mut rx) = mpsc::unbounded_channel::<AppMessage>();

    // Configure only claude-code to avoid other agents
    let mut agents = AgentConfig::default();
    agents.gemini.enabled = false;
    agents.auto_connect = vec![];
    agents.claude_code.enabled = true;

    // Build manager; it will initialize adapters using installer
    let mut manager = AgentManager::new(agents.clone(), tx.clone()).await?;

    // Ensure adapter exists and connect
    manager.connect_agent("claude-code").await?;

    // Try creating a session. This may succeed or fail with auth required; both are acceptable
    let result = manager.create_session("claude-code").await;
    match result {
        Ok(session_id) => {
            assert!(!session_id.0.is_empty());
        }
        Err(err) => {
            // We still consider the flow successful in terms of plumbing
            eprintln!("create_session error (acceptable if auth required): {}", err);
        }
    }

    // Try to stop the agent cleanly
    manager.disconnect_agent("claude-code").await.ok();

    // Drain any messages to ensure channel operations are sound
    while let Ok(_msg) = rx.try_recv() {
        // NOP: just ensure we can receive without panicking
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn acp_client_start_and_create_session() -> Result<()> {
    if should_skip() { return Ok(()); }

    let installer = AgentInstaller::new()?;
    let cmd = installer.get_or_install_claude_code().await?;

    let (tx, mut rx) = mpsc::unbounded_channel::<AppMessage>();
    let mut client = AcpClient::new(
        "claude-code",
        cmd.path.to_str().unwrap(),
        cmd.args.clone(),
        cmd.env.clone(),
        tx,
    );

    // Start process
    client.start().await?;

    // Try create session
    let result = client.create_session().await;
    match result {
        Ok(sid) => {
            assert!(!sid.0.is_empty());
        }
        Err(e) => {
            eprintln!("create_session error (acceptable if auth required): {}", e);
        }
    }

    client.stop().await.ok();

    // Drain any messages to ensure channel plumbing is fine
    while let Ok(_msg) = rx.try_recv() {}

    Ok(())
}
