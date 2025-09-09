use anyhow::Result;
use rat::acp::AcpClient;
use rat::app::AppMessage;
use std::env;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Check if we have an agent command
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <agent-command> [agent-args...]", args[0]);
        eprintln!("Example: {} target/debug/examples/agent", args[0]);
        return Ok(());
    }

    let agent_command = &args[1];
    let extra_args: Vec<String> = if args.len() > 2 { args[2..].to_vec() } else { vec![] };

    println!("Testing ACP client with agent: {}", agent_command);

    // Create message channel
    let (message_tx, mut message_rx) = mpsc::unbounded_channel::<AppMessage>();

    // Create ACP client
    let mut client = AcpClient::new("test-agent", agent_command, extra_args, None, message_tx);

    println!("Starting ACP client...");

    // Start the client
    if let Err(e) = client.start().await {
        eprintln!("Failed to start ACP client: {}", e);
        return Err(e);
    }

    println!("ACP client started successfully!");

    // Create a session
    let session_id = client.create_session().await?;
    println!("Created session: {}", session_id.0);

    // Send a test message
    println!("Sending test message...");
    client
        .send_message(
            &session_id,
            "Hello! Can you help me with a simple task?".to_string(),
        )
        .await?;

    // Listen for messages for a limited time
    println!("Listening for messages...");
    let listen_timeout = Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < listen_timeout {
        // Check for messages with a short timeout
        if let Ok(Some(app_message)) = timeout(Duration::from_millis(100), message_rx.recv()).await
        {
            match app_message {
                AppMessage::AgentMessage {
                    agent_name,
                    message,
                } => {
                    println!(
                        "Received message from {}: {:?}",
                        agent_name, message.content
                    );
                }
                AppMessage::AgentConnected { agent_name } => {
                    println!("Agent connected: {}", agent_name);
                }
                AppMessage::AgentDisconnected { agent_name } => {
                    println!("Agent disconnected: {}", agent_name);
                }
                AppMessage::SessionCreated {
                    agent_name,
                    session_id,
                } => {
                    println!("Session created for {}: {}", agent_name, session_id.0);
                }
                AppMessage::Error { error } => {
                    println!("Error: {}", error);
                }
                AppMessage::Quit => {
                    println!("Quit message received");
                    break;
                }
            }
        }

        // Check if client is still connected
        if !client.is_connected() {
            println!("Client disconnected");
            break;
        }
    }

    println!("Stopping ACP client...");
    client.stop().await?;
    println!("Test completed successfully!");

    Ok(())
}
