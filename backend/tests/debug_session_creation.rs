use anyhow::Result;
use log::info;
use std::path::PathBuf;

use rat::adapters::agent_installer::AgentInstaller;
use rat::adapters::claude_code::ClaudeCodeAdapter;
use rat::adapters::traits::AgentAdapter;
use rat::app::AppMessage;
use rat::config::agent::ClaudeCodeConfig;
use tokio::sync::mpsc;

#[ignore]
#[tokio::test]
async fn debug_claude_code_session_creation() -> Result<()> {
    env_logger::init();

    info!("=== Starting debug test for claude-code session creation ===");

    // Step 1: Test agent installer
    info!("Step 1: Testing AgentInstaller");
    let installer = AgentInstaller::new()?;

    match installer.get_or_install_claude_code().await {
        Ok(command) => {
            info!("✅ AgentInstaller found claude-code-acp: {:?}", command);

            // Step 2: Test command verification
            info!("Step 2: Verifying command");
            match installer.verify_agent_command(&command).await {
                Ok(version) => {
                    info!("✅ Command verification successful: {}", version);
                }
                Err(e) => {
                    info!("❌ Command verification failed: {}", e);
                    info!("Continuing test without verification...");
                }
            }

            // Step 3: Test adapter creation
            info!("Step 3: Creating ClaudeCodeAdapter");
            // Use default config for current struct layout
            let config = ClaudeCodeConfig::default();

            let (message_tx, mut message_rx) = mpsc::unbounded_channel::<AppMessage>();
            let mut adapter = ClaudeCodeAdapter::new(config, message_tx).await?;
            info!("✅ ClaudeCodeAdapter created successfully");

            // Step 4: Test adapter start
            info!("Step 4: Starting adapter");
            match adapter.start().await {
                Ok(()) => {
                    info!("✅ Adapter started successfully");

                    // Step 5: Test session creation
                    info!("Step 5: Creating session");
                    match adapter.create_session().await {
                        Ok(session_id) => {
                            info!("✅ Session created successfully: {}", session_id.0);

                            // Step 6: Test sending a message
                            info!("Step 6: Sending test message");
                            match adapter
                                .send_message(
                                    &session_id,
                                    "Hello, can you help me test this connection?".to_string(),
                                )
                                .await
                            {
                                Ok(()) => {
                                    info!("✅ Message sent successfully");

                                    // Wait for potential response
                                    info!("Waiting for potential response...");
                                    let mut attempts = 0;
                                    while attempts < 10 {
                                        if let Ok(app_message) = message_rx.try_recv() {
                                            info!("📨 Received message: {:?}", app_message);
                                            break;
                                        }
                                        tokio::time::sleep(tokio::time::Duration::from_millis(500))
                                            .await;
                                        attempts += 1;
                                    }

                                    if attempts >= 10 {
                                        info!("⚠️  No response received within timeout");
                                    }
                                }
                                Err(e) => {
                                    info!("❌ Failed to send message: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            info!("❌ Failed to create session: {}", e);
                        }
                    }

                    // Step 7: Clean up
                    info!("Step 7: Stopping adapter");
                    if let Err(e) = adapter.stop().await {
                        info!("⚠️  Error stopping adapter: {}", e);
                    } else {
                        info!("✅ Adapter stopped successfully");
                    }
                }
                Err(e) => {
                    info!("❌ Failed to start adapter: {}", e);
                    info!("Error details: {:?}", e);

                    // Let's try to understand the specific error
                    // Collect error chain with explicit trait object type
                    let error_chain: Vec<&(dyn std::error::Error + 'static)> =
                        std::iter::successors(
                            Some(e.as_ref() as &(dyn std::error::Error + 'static)),
                            |e| e.source(),
                        )
                        .collect();

                    for (i, err) in error_chain.iter().enumerate() {
                        info!("Error level {}: {}", i, err);
                    }
                }
            }
        }
        Err(e) => {
            info!("❌ AgentInstaller failed to find claude-code-acp: {}", e);

            // Let's check what paths were searched
            info!("Let's manually check common locations...");

            let potential_paths = vec![
                PathBuf::from("/Users/luc/Library/Application Support/rat/agents/claude-code/0.3.0/node_modules/.bin/claude-code-acp"),
                PathBuf::from("/Users/luc/Library/Application Support/Zed/external_agents/claude-code-acp/0.3.0/node_modules/.bin/claude-code-acp"),
                PathBuf::from("/Users/luc/.npm/bin/claude-code-acp"),
                PathBuf::from("/usr/local/bin/claude-code-acp"),
            ];

            for path in potential_paths {
                if path.exists() {
                    info!("Found potential path: {}", path.display());

                    // Test if it's executable
                    let test_result = std::process::Command::new("node")
                        .arg(&path)
                        .arg("--version")
                        .output();

                    match test_result {
                        Ok(output) => {
                            if output.status.success() {
                                info!("✅ Path is executable: {}", path.display());
                            } else {
                                info!("❌ Path exists but execution failed: {}", path.display());
                                info!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                            }
                        }
                        Err(e) => {
                            info!("❌ Failed to test path {}: {}", path.display(), e);
                        }
                    }
                } else {
                    info!("Path does not exist: {}", path.display());
                }
            }
        }
    }

    info!("=== Debug test completed ===");
    Ok(())
}

#[ignore]
#[tokio::test]
async fn test_manual_acp_connection() -> Result<()> {
    env_logger::init();

    info!("=== Testing manual ACP connection ===");

    // Use the known working path
    let claude_path = PathBuf::from("/Users/luc/Library/Application Support/rat/agents/claude-code/0.3.0/node_modules/.bin/claude-code-acp");

    if !claude_path.exists() {
        info!(
            "❌ Claude-code-acp not found at expected path: {}",
            claude_path.display()
        );
        return Ok(());
    }

    info!("✅ Found claude-code-acp at: {}", claude_path.display());

    // Try to start the process manually
    let mut child = tokio::process::Command::new("node")
        .arg(&claude_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    info!("✅ Successfully spawned claude-code-acp process");

    // Wait a moment for startup
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Check if process is still running
    match child.try_wait()? {
        Some(status) => {
            info!("❌ Process exited early with status: {}", status);

            // Read stderr to see what happened
            if let Some(mut stderr) = child.stderr.take() {
                use tokio::io::AsyncReadExt;
                let mut buffer = Vec::new();
                let _ = stderr.read_to_end(&mut buffer).await;
                let stderr_output = String::from_utf8_lossy(&buffer);
                info!("Process stderr: {}", stderr_output);
            }
        }
        None => {
            info!("✅ Process is running");

            // Kill the process
            child.kill().await?;
            info!("✅ Process terminated");
        }
    }

    info!("=== Manual ACP connection test completed ===");
    Ok(())
}
