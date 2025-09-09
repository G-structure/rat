use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::io::IsTerminal;
use std::io::Write as _;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::{timeout, Duration};
use which::which;

// Reuse the crate's installer logic to find the right binary/entrypoint
use rat::adapters::agent_installer::AgentInstaller;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("=== Simple ACP Test ===");

    // Always leave the terminal in a sane state on exit, even if TUIs misconfigure it
    let _term_reset_guard = TerminalResetGuard::new();

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
                if let Some(stdout) = child.stdout.take() {
                    let mut stdout_reader = BufReader::new(stdout);
                    let mut line = String::new();

                    // Set a timeout for reading
                    match timeout(
                        Duration::from_millis(1500),
                        stdout_reader.read_line(&mut line),
                    )
                    .await
                    {
                        Ok(Ok(0)) => println!("No response received"),
                        Ok(Ok(_)) => {
                            println!("Response: {}", line.trim());

                            // Attempt to parse initialize result and handle authMethods
                            if let Err(e) = maybe_handle_login(&installer, line.trim()).await {
                                eprintln!("⚠️  Login handling error: {}", e);
                            }

                            // After initialization, create a session like Zed does, then send a prompt and wait for response
                            if let Err(e) =
                                maybe_run_prompt_flow(&mut stdin, &mut stdout_reader).await
                            {
                                eprintln!("⚠️  ACP prompt flow error: {}", e);
                            }
                        }
                        Ok(Err(e)) => println!("Error reading response: {}", e),
                        Err(_) => println!("Timeout reading response"),
                    }
                }
            }

            // Kill the process
            child.kill().await?;
            // Normalize terminal line/cursor before further output in case a TUI left it mid-line
            normalize_terminal_line();
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

    let Some(result) = v.get("result") else {
        return Ok(());
    };
    let Some(methods) = result.get("authMethods").and_then(|m| m.as_array()) else {
        return Ok(());
    };
    let has_claude_login = methods
        .iter()
        .any(|m| m.get("id").and_then(|id| id.as_str()) == Some("claude-login"));
    if !has_claude_login {
        return Ok(());
    }

    println!("🔐 Auth method 'claude-login' detected; running login flow...");

    // Ensure terminal is restored after any TUI interaction from the login command
    let _term_reset_guard = TerminalResetGuard::new();

    let login_cmd = installer.get_claude_login_command().await?;
    println!(
        "➡️  Running login command: {:?} {:?}",
        login_cmd.path, login_cmd.args
    );

    // Prefer using a pseudo-terminal and watch for success markers, then close it.
    // When running in an interactive terminal, inherit stdin so the user can
    // interact with the login flow while we monitor output. This mirrors Zed's
    // approach: detect "Login successful" and immediately kill the spawned
    // login process so the calling code can continue.
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let mut login =
        spawn_login_with_pty_if_available(&login_cmd.path, &login_cmd.args, interactive)?;

    let stdout = login
        .stdout
        .take()
        .ok_or_else(|| anyhow!("no stdout from login process"))?;
    let maybe_stderr = login.stderr.take();

    let (tx, mut rx) = tokio_mpsc::unbounded_channel::<String>();
    // In interactive mode, only echo stdout to avoid duplicate, messy rendering
    // caused by many TUIs writing to both stdout and stderr.
    tokio::spawn(read_lines_to_channel(stdout, tx.clone()));
    if !interactive {
        if let Some(stderr) = maybe_stderr {
            tokio::spawn(read_lines_to_channel(stderr, tx.clone()));
        }
    }

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
                if interactive {
                    // Echo the raw PTY output to the user for an interactive feel.
                    // Avoid double newlines: `line` already contains a trailing newline.
                    print!("{}", line);
                } else {
                    println!("login: {}", trimmed);
                }
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

/// Best-effort terminal reset guard to recover from misbehaving TUIs.
struct TerminalResetGuard {
    enabled: bool,
}

impl TerminalResetGuard {
    fn new() -> Self {
        // Only attempt to reset if attached to a real TTY
        let enabled = std::io::stdout().is_terminal() || std::io::stderr().is_terminal();
        TerminalResetGuard { enabled }
    }
}

impl Drop for TerminalResetGuard {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }

        // ANSI sequence resets: attributes, show cursor, leave alt screen
        let mut out = std::io::stdout();
        // Return to column 0, clear the current line, then reset attributes and exit alt screen
        let _ = write!(out, "\r\x1b[2K\r\x1b[0m\x1b[?25h\x1b[?1049l");
        let _ = out.flush();

        // Fallbacks using common term utils, ignore any failures
        let _ = std::process::Command::new("stty").arg("sane").status();
        let _ = std::process::Command::new("tput").arg("cnorm").status(); // show cursor
        let _ = std::process::Command::new("tput").arg("rmcup").status(); // leave alt screen
    }
}

/// Clear the current line and move cursor to column 0 to avoid offset printing
fn normalize_terminal_line() {
    if !(std::io::stdout().is_terminal() || std::io::stderr().is_terminal()) {
        return;
    }
    let mut out = std::io::stdout();
    let _ = write!(out, "\r\x1b[2K\r");
    let _ = out.flush();
}

fn spawn_login_with_pty_if_available(
    path: &std::path::Path,
    args: &[String],
    inherit_stdin: bool,
) -> Result<Child> {
    // If `script` is available, wrap command to allocate a PTY for interactive login
    let use_script = which("script").is_ok();
    if use_script {
        let mut cmd = Command::new("script");
        // -q: quiet, /dev/null: discard transcript, then the actual command and args
        cmd.arg("-q").arg("/dev/null").arg(path);
        for a in args {
            cmd.arg(a);
        }
        let stdin = if inherit_stdin {
            std::process::Stdio::inherit()
        } else {
            std::process::Stdio::null()
        };
        return Ok(cmd
            .stdin(stdin)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?);
    }

    // Fallback: spawn directly (may not work if CLI requires a TTY)
    Ok(Command::new(path)
        .args(args)
        .stdin(if inherit_stdin {
            std::process::Stdio::inherit()
        } else {
            std::process::Stdio::null()
        })
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

/// Create a new ACP session, send a prompt, and wait for the response.
async fn maybe_run_prompt_flow(
    stdin: &mut (impl AsyncWriteExt + Unpin),
    stdout_reader: &mut BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> Result<()> {
    // Build and send session/new
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();
    let new_session = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {"mcpServers": [], "cwd": cwd}
    });
    let req = serde_json::to_string(&new_session)?;
    println!("➡️  Sending ACP: {}", req);
    stdin.write_all(req.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read responses until we get the session/new result
    let new_session_id = 2u64;
    let session_id = loop {
        let mut line = String::new();
        match timeout(
            Duration::from_millis(2000),
            stdout_reader.read_line(&mut line),
        )
        .await
        {
            Ok(Ok(0)) => {
                println!("No response to session/new");
                return Ok(());
            }
            Ok(Ok(_)) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
                    if let Some(id) = v.get("id").and_then(|x| x.as_u64()) {
                        if id == new_session_id {
                            if let Some(result) = v.get("result") {
                                if let Some(sid) = result.get("sessionId").and_then(|s| s.as_str())
                                {
                                    println!("session/new -> {}", trimmed);
                                    break sid.to_string();
                                }
                            }
                        }
                    } else if v.get("method").and_then(|m| m.as_str()) == Some("session/update") {
                        println!("🪄 update: {}", trimmed);
                    } else {
                        println!("↪️  other: {}", trimmed);
                    }
                } else {
                    println!("raw: {}", trimmed);
                }
            }
            Ok(Err(e)) => {
                println!("Error reading session/new response: {}", e);
                return Ok(());
            }
            Err(_) => {
                println!("Timeout waiting for session/new response");
                return Ok(());
            }
        }
    };
    let session_id = session_id;
    println!("🆔 New session id: {}", session_id);

    // Send a prompt request
    let prompt_id = 3u32;
    let prompt = json!({
        "jsonrpc": "2.0",
        "id": prompt_id,
        "method": "session/prompt",
        "params": {
            "sessionId": session_id,
            "prompt": [
                {"type": "text", "text": "Reply with 'pong' and stop."}
            ]
        }
    });
    let prompt_req = serde_json::to_string(&prompt)?;
    println!("➡️  Sending ACP prompt: {}", prompt_req);
    stdin.write_all(prompt_req.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read loop: print session/update notifications and wait for the prompt response
    let start = std::time::Instant::now();
    let max = Duration::from_secs(30);
    loop {
        if start.elapsed() > max {
            println!("⏱️  Timed out waiting for prompt response");
            break;
        }

        let mut msg = String::new();
        match timeout(
            Duration::from_millis(2000),
            stdout_reader.read_line(&mut msg),
        )
        .await
        {
            Ok(Ok(0)) => {
                println!("ℹ️  Agent closed stdout");
                break;
            }
            Ok(Ok(_n)) => {
                let trimmed = msg.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Try to parse JSON-RPC
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(v) => {
                        if let Some(method) = v.get("method").and_then(|m| m.as_str()) {
                            if method == "session/update" {
                                println!("🪄 update: {}", trimmed);
                            } else {
                                println!("ℹ️  notif: {}", trimmed);
                            }
                        } else if let Some(id) = v.get("id").and_then(|x| x.as_u64()) {
                            if id == prompt_id as u64 {
                                // the prompt response
                                println!("✅ prompt response: {}", trimmed);
                                break;
                            } else {
                                println!("↩️  response: {}", trimmed);
                            }
                        } else {
                            println!("↪️  other: {}", trimmed);
                        }
                    }
                    Err(_) => {
                        println!("raw: {}", trimmed);
                    }
                }
            }
            Ok(Err(e)) => {
                println!("Error reading agent output: {}", e);
                break;
            }
            Err(_) => {
                println!("…waiting for agent output…");
            }
        }
    }

    Ok(())
}
