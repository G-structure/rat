use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tokio::process::Child;
use tokio::sync::mpsc as tokio_mpsc;
use which::which;
use std::io::IsTerminal;

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

            // Try to send a valid ACP initialize request
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
                        Ok(Ok(_)) => {
                            println!("Response: {}", line.trim());

                            // Attempt to parse initialize result and handle authMethods
                            if let Err(e) = maybe_handle_login(&installer, line.trim()).await {
                                eprintln!("⚠️  Login handling error: {}", e);
                            }
                        }
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

async fn maybe_handle_login(installer: &AgentInstaller, response_line: &str) -> Result<()> {
    // Parse JSON-RPC response and look for result.authMethods with id == "claude-login"
    let v: Value = serde_json::from_str(response_line)
        .with_context(|| format!("failed to parse JSON: {}", response_line))?;

    let Some(result) = v.get("result") else { return Ok(()); };
    let Some(methods) = result.get("authMethods").and_then(|m| m.as_array()) else { return Ok(()); };
    let has_claude_login = methods.iter().any(|m| m.get("id").and_then(|id| id.as_str()) == Some("claude-login"));
    if !has_claude_login { return Ok(()); }

    println!("🔐 Auth method 'claude-login' detected; running login flow...");

    let login_cmd = installer.get_claude_login_command().await?;
    println!("➡️  Running login command: {:?} {:?}", login_cmd.path, login_cmd.args);

    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if interactive {
        // Attach to the user's terminal so they can answer prompts.
        let status = Command::new(&login_cmd.path)
            .args(&login_cmd.args)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await
            .context("failed to run interactive login")?;
        if status.success() {
            println!("✅ Login flow completed (exit code 0)");
            return Ok(());
        } else {
            return Err(anyhow!("login failed with status: {}", status));
        }
    }

    // Non-interactive fallback: try to allocate a PTY and watch for success markers.
    let mut login = spawn_login_with_pty_if_available(&login_cmd.path, &login_cmd.args)?;

    let stdout = login
        .stdout
        .take()
        .ok_or_else(|| anyhow!("no stdout from login process"))?;
    let stderr = login
        .stderr
        .take()
        .ok_or_else(|| anyhow!("no stderr from login process"))?;

    let (tx, mut rx) = tokio_mpsc::unbounded_channel::<String>();
    tokio::spawn(read_lines_to_channel(stdout, tx.clone()));
    tokio::spawn(read_lines_to_channel(stderr, tx.clone()));

    let start = std::time::Instant::now();
    let max = Duration::from_secs(180);
    let success_markers = [
        "Login successful",
        "Already logged in",
        "You are logged in",
        "Logged in as",
        "Successfully logged in",
    ];

    loop {
        if let Some(status) = login.try_wait()? {
            if status.success() {
                println!("ℹ️  Login process exited with code 0; assuming success");
                break;
            } else {
                return Err(anyhow!("login process exited with status: {}", status));
            }
        }

        match timeout(Duration::from_millis(1000), rx.recv()).await {
            Ok(Some(mut line)) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.contains("\u{1b}[2J") || trimmed.contains("\u{1b}c") {
                    println!("[SCREEN CLEARED]");
                    line.clear();
                    continue;
                }
                println!("login: {}", trimmed);
                if success_markers.iter().any(|m| trimmed.contains(m)) {
                    println!("✅ Login successful detected");
                    break;
                }
            }
            Ok(None) => {}
            Err(_) => {
                if start.elapsed() > max {
                    let _ = login.kill().await;
                    return Err(anyhow!("timed out waiting for login to complete"));
                }
            }
        }
    }

    let _ = login.kill().await;
    Ok(())
}

fn spawn_login_with_pty_if_available(path: &std::path::Path, args: &[String]) -> Result<Child> {
    // If `script` is available, wrap command to allocate a PTY for interactive login
    let use_script = which("script").is_ok();
    if use_script {
        let mut cmd = Command::new("script");
        // -q: quiet, /dev/null: discard transcript, then the actual command and args
        cmd.arg("-q").arg("/dev/null").arg(path);
        for a in args {
            cmd.arg(a);
        }
        return Ok(cmd
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?);
    }

    // Fallback: spawn directly (may not work if CLI requires a TTY)
    Ok(Command::new(path)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?)
}

async fn read_lines_to_channel<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    reader: R,
    tx: tokio_mpsc::UnboundedSender<String>,
) {
    let mut rdr = BufReader::new(reader);
    let mut buf = String::new();
    loop {
        buf.clear();
        match rdr.read_line(&mut buf).await {
            Ok(0) => break,
            Ok(_) => {
                let _ = tx.send(buf.clone());
            }
            Err(_) => break,
        }
    }
}
