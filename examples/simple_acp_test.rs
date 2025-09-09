use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

// Reuse the crate's installer logic to find the right binary/entrypoint
use rat::adapters::agent_installer::AgentInstaller;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("=== Simple ACP Test ===");

    // Resolve the agent command the same way the app does
    let installer = AgentInstaller::new()?;
    let command = installer.get_or_install_claude_code().await?;
    println!("✅ Using command: {:?} {:?}", command.path, command.args);

    // Test 2: Try to run it with --version, but don’t hang — use a timeout
    println!("Testing --version flag (3s timeout)...");
    let mut ver_cmd = Command::new(&command.path);
    if !command.args.is_empty() {
        ver_cmd.args(&command.args);
    }
    ver_cmd.arg("--version");

    match timeout(Duration::from_secs(3), ver_cmd.output()).await {
        Ok(Ok(output)) => {
            println!("Exit code: {}", output.status);
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(Err(e)) => {
            println!("❌ Failed to run --version: {}", e);
        }
        Err(_) => {
            println!("⏱️  Timeout waiting for --version; continuing...");
        }
    }

    // Test 3: Try to start it and see what happens
    println!("\nTesting process startup...");
    let mut child = Command::new(&command.path)
        .args(&command.args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    println!("✅ Process spawned successfully");

    // Wait a moment for startup
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Check if process is still running
    match child.try_wait()? {
        Some(status) => {
            println!("❌ Process exited with status: {}", status);

            // Read stderr to see what happened
            if let Some(mut stderr) = child.stderr.take() {
                let mut stderr_reader = BufReader::new(stderr);
                let mut line = String::new();
                while stderr_reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
                    print!("stderr: {}", line);
                    line.clear();
                }
            }
        }
        None => {
            println!("✅ Process is running");

            // Try to send some simple input
            if let Some(mut stdin) = child.stdin.take() {
                println!("Trying to write to stdin...");
                // Send a valid ACP initialize request so the agent responds deterministically
                let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":false}}}"#;
                stdin.write_all(init.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
                stdin.flush().await?;

                // Give it time to respond
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

                // Try to read response
                if let Some(mut stdout) = child.stdout.take() {
                    let mut stdout_reader = BufReader::new(stdout);
                    let mut line = String::new();

                    // Set a timeout for reading
                    match timeout(Duration::from_millis(1500), stdout_reader.read_line(&mut line))
                        .await
                    {
                        Ok(Ok(0)) => println!("No response received"),
                        Ok(Ok(_)) => println!("Response: {}", line.trim()),
                        Ok(Err(e)) => println!("Error reading response: {}", e),
                        Err(_) => println!("Timeout reading response"),
                    }
                }
            }

            // Kill the process
            child.kill().await?;
            println!("✅ Process terminated");
        }
    }

    println!("=== Test completed ===");
    Ok(())
}
