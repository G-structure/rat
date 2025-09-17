use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use std::env;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::server::{Request, Response},
        Message,
    },
};

use crate::adapters::agent_installer::{AgentCommand, AgentInstaller};

/// Start a local WebSocket server for direct connections (no encryption, no pairing)
/// This is for local development only - WARNING: No security/encryption!
pub async fn start_local_ws_server(port: u16) -> Result<()> {
    env_logger::init();

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🔧 LOCAL DEV: WebSocket server listening on ws://{}", addr);
    info!("🔧 LOCAL DEV: WARNING - No encryption, no authentication! Local development only!");

    // Try to automatically resolve an ACP agent (Claude Code preferred), like the TUI does
    let resolved_agent: Option<AgentCommand> = match env::var("RAT2E_AGENT_CMD") {
        Ok(cmd_path) => {
            let args = env::var("RAT2E_AGENT_ARGS").unwrap_or_default();
            let args_vec: Vec<String> = if args.is_empty() {
                vec![]
            } else {
                args.split_whitespace().map(|s| s.to_string()).collect()
            };
            info!(
                "🔧 LOCAL DEV: Using ACP agent from env: {} {}",
                cmd_path,
                args_vec.join(" ")
            );
            Some(AgentCommand::new(cmd_path.into()).with_args(args_vec))
        }
        Err(_) => match AgentInstaller::new() {
            Ok(installer) => match installer.get_or_install_claude_code().await {
                Ok(cmd) => {
                    info!(
                        "🔧 LOCAL DEV: Auto-resolved Claude Code ACP at {}",
                        cmd.path.display()
                    );
                    Some(cmd)
                }
                Err(e1) => {
                    warn!("🔧 LOCAL DEV: Claude Code not available: {}", e1);
                    match installer.get_or_install_gemini().await {
                        Ok(cmd) => {
                            info!(
                                "🔧 LOCAL DEV: Auto-resolved Gemini CLI at {}",
                                cmd.path.display()
                            );
                            Some(cmd)
                        }
                        Err(e2) => {
                            warn!(
                                "🔧 LOCAL DEV: No ACP agent resolved automatically: {}; {}",
                                e1, e2
                            );
                            None
                        }
                    }
                }
            },
            Err(e) => {
                warn!("🔧 LOCAL DEV: AgentInstaller init failed: {}", e);
                None
            }
        },
    };

    while let Ok((stream, peer_addr)) = listener.accept().await {
        info!("🔧 LOCAL DEV: New connection from {}", peer_addr);
        let agent_clone = resolved_agent.clone();
        tokio::spawn(handle_local_connection(stream, peer_addr, agent_clone));
    }

    Ok(())
}

async fn handle_local_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    resolved_agent: Option<AgentCommand>,
) -> Result<()> {
    // Accept WS and echo subprotocol if client asks for acp.jsonrpc.v1 (browser correctness)
    let ws_stream = accept_hdr_async(stream, |req: &Request, mut resp: Response| {
        // Look for Sec-WebSocket-Protocol and echo acp.jsonrpc.v1 if requested
        if let Some(values) = req.headers().get("Sec-WebSocket-Protocol") {
            if let Ok(hv) = values.to_str() {
                // Header may contain a comma-separated list per RFC 6455
                let asked = hv
                    .split(',')
                    .map(|s| s.trim())
                    .any(|p| p.eq_ignore_ascii_case("acp.jsonrpc.v1"));
                if asked {
                    // Avoid direct `http` crate import by parsing into the expected HeaderValue
                    let _ = resp
                        .headers_mut()
                        .append("Sec-WebSocket-Protocol", "acp.jsonrpc.v1".parse().unwrap());
                }
            }
        }
        Ok(resp)
    })
    .await
    .map_err(|e| anyhow::anyhow!("WebSocket handshake failed: {}", e))?;

    info!("🔧 LOCAL DEV: WebSocket connection established with {}", peer_addr);

    let (mut ws_write, mut ws_read) = ws_stream.split();

    // If an ACP agent was resolved (env or auto), run the bridge using direct (unencrypted) transport
    if resolved_agent.is_some() || std::env::var("RAT2E_AGENT_CMD").is_ok() {
        run_acp_bridge_local(ws_write, ws_read, resolved_agent).await?;
        info!("🔧 LOCAL DEV: ACP bridge session ended for {}", peer_addr);
        return Ok(());
    }

    // If no ACP agent, run a simple echo server for testing
    info!("🔧 LOCAL DEV: No ACP agent configured, running echo mode");

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "message": "Connected to RAT local development server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    ws_write.send(Message::Text(welcome.to_string())).await?;

    // Echo loop
    while let Some(msg) = ws_read.next().await {
        match msg? {
            Message::Text(text) => {
                info!("🔧 LOCAL DEV: Received text: {}", text);
                let echo = serde_json::json!({
                    "type": "echo",
                    "original": text,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                ws_write.send(Message::Text(echo.to_string())).await?;
            }
            Message::Binary(data) => {
                info!("🔧 LOCAL DEV: Received binary data: {} bytes", data.len());
                // Try to parse as JSON
                if let Ok(text) = String::from_utf8(data.clone()) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let echo = serde_json::json!({
                            "type": "echo",
                            "original": parsed,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });
                        ws_write.send(Message::Text(echo.to_string())).await?;
                    } else {
                        ws_write.send(Message::Binary(data)).await?;
                    }
                } else {
                    ws_write.send(Message::Binary(data)).await?;
                }
            }
            Message::Close(_) => {
                info!("🔧 LOCAL DEV: Connection closed by {}", peer_addr);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn run_acp_bridge_local<WS, WR>(
    mut ws_write: WS,
    mut ws_read: WR,
    resolved_agent: Option<AgentCommand>,
) -> Result<()>
where
    WS: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
    WR: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin + Send + 'static,
{
    // Determine agent command: prefer resolved_agent; fallback to env variables
    let (path, args_vec, env_map): (
        std::path::PathBuf,
        Vec<String>,
        Option<std::collections::HashMap<String, String>>,
    ) = if let Some(cmd) = resolved_agent {
        (cmd.path, cmd.args, cmd.env)
    } else {
        let cmd = env::var("RAT2E_AGENT_CMD").map_err(|_| {
            anyhow::anyhow!(
                "No ACP agent resolved. Set RAT2E_AGENT_CMD or install @zed-industries/claude-code-acp"
            )
        })?;
        let args = env::var("RAT2E_AGENT_ARGS").unwrap_or_default();
        let args_vec: Vec<String> = if args.is_empty() {
            vec![]
        } else {
            args.split_whitespace().map(|s| s.to_string()).collect()
        };
        (cmd.into(), args_vec, None)
    };

    info!(
        "🔧 LOCAL DEV: Starting ACP agent: {} {}",
        path.display(),
        args_vec.join(" ")
    );
    let mut command = Command::new(path);
    command
        .args(args_vec)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());
    if let Some(envs) = env_map {
        command.envs(envs);
    }
    let mut child = command.spawn()?;
    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get agent stdin"))?;
    let mut child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get agent stdout"))?;

    // Task: WS -> agent stdin (direct pass-through, no encryption)
    let ws_to_agent = tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = child_stdin.write_all(text.as_bytes()).await {
                        warn!("🔧 LOCAL DEV: stdin write error: {}", e);
                        break;
                    }
                    if let Err(e) = child_stdin.write_all(b"\n").await {
                        warn!("🔧 LOCAL DEV: stdin newline write error: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Err(e) = child_stdin.write_all(&data).await {
                        warn!("🔧 LOCAL DEV: stdin write error: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    // Task: agent stdout -> WS (direct pass-through, no encryption)
    let agent_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 8192];
        loop {
            match child_stdout.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let data = &buf[..n];
                    // Try to send as text if it's valid UTF-8, otherwise binary
                    if let Ok(text) = std::str::from_utf8(data) {
                        if let Err(e) = ws_write.send(Message::Text(text.to_string())).await {
                            warn!("🔧 LOCAL DEV: ws send error: {}", e);
                            break;
                        }
                    } else {
                        if let Err(e) = ws_write.send(Message::Binary(data.to_vec())).await {
                            warn!("🔧 LOCAL DEV: ws send error: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("🔧 LOCAL DEV: stdout read error: {}", e);
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(ws_to_agent, agent_to_ws);
    Ok(())
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::header::{HeaderName, HeaderValue};
    use tokio_tungstenite::{connect_async, connect_async_with_config, tungstenite::Message};

    async fn find_free_port(start: u16) -> u16 {
        for p in start..(start + 50) {
            if tokio::net::TcpListener::bind(("127.0.0.1", p)).await.is_ok() {
                return p;
            }
        }
        start
    }

    #[tokio::test]
    async fn ws_handshake_echoes_acp_subprotocol() {
        let port = find_free_port(8950).await;
        tokio::spawn(async move {
            let _ = start_local_ws_server(port).await;
        });
        sleep(Duration::from_millis(100)).await;

        let url = format!("ws://127.0.0.1:{}/", port);
        let mut req = url.into_client_request().expect("valid request");
        let hname = HeaderName::from_static("sec-websocket-protocol");
        req.headers_mut()
            .insert(hname, HeaderValue::from_static("acp.jsonrpc.v1"));

        let (_ws, resp) = connect_async_with_config(req, None, false)
            .await
            .expect("connect ok");

        let echoed = resp
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        assert_eq!(echoed, "acp.jsonrpc.v1");
    }

    #[tokio::test]
    async fn ws_accepts_acp_text_frames_in_echo_mode() {
        // Ensure no agent is auto-resolved by clearing env
        std::env::remove_var("RAT2E_AGENT_CMD");

        let port = find_free_port(8960).await;
        tokio::spawn(async move {
            let _ = start_local_ws_server(port).await;
        });
        sleep(Duration::from_millis(100)).await;

        let url = format!("ws://127.0.0.1:{}/", port);
        let (mut ws, _resp) = connect_async(url).await.expect("connect ok");

        // Welcome
        let first = ws.next().await.expect("welcome").expect("ok");
        let welcome = match first { Message::Text(t) => t, _ => String::new() };
        assert!(welcome.contains("Connected to RAT local development server"));

        // Send initialize as Text
        let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":false}}}"#;
        ws.send(Message::Text(init.to_string())).await.expect("send");

        // Expect echo wrapper containing our JSON
        let msg = ws.next().await.expect("echo").expect("ok");
        let echoed = match msg { Message::Text(t) => t, _ => String::new() };
        assert!(echoed.contains("\"jsonrpc\":\"2.0\""));
        assert!(echoed.contains("\"method\":\"initialize\""));

        ws.close(None).await.ok();
    }
}
