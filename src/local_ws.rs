use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use std::env;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio::sync::{Mutex, oneshot};
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
    fn id_key(v: &serde_json::Value) -> Option<String> {
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Null => None,
            other => Some(other.to_string()),
        }
    }
    // Track permission prompts awaiting a browser decision
    let pending_perms: Arc<Mutex<HashMap<String, oneshot::Sender<bool>>>> = Arc::new(Mutex::new(HashMap::new()));
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

    // Build additional args to ensure Claude Code has edit/tools enabled when used
    // via the WS bridge. Only applies when we detect Claude Code entrypoints.
    let mut extra_args: Vec<String> = Vec::new();
    let path_str = path.to_string_lossy().to_string();
    let looks_like_claude = path_str.contains("claude-code-acp")
        || args_vec
            .iter()
            .any(|a| a.contains("@zed-industries/claude-code-acp") || a.contains("@anthropic-ai/claude-code/cli.js"));
    if looks_like_claude {
        let permission_tool = std::env::var("RAT_PERMISSION_PROMPT_TOOL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "mcp__acp__permission".to_string());
        extra_args.push("--permission-prompt-tool".into());
        extra_args.push(permission_tool);

        // In local web mode, prefer ACP FS path so the bridge can gate and
        // perform operations with permission prompts.
        let allowed_default = "mcp__acp__read,mcp__acp__write";
        let allowed = std::env::var("RAT_ALLOWED_TOOLS")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| allowed_default.to_string());
        extra_args.push("--allowedTools".into());
        extra_args.push(allowed);

        // By default, disallow Claude built-in edit/write tools in web mode
        // so the agent requests ACP FS and we can prompt in the browser.
        match std::env::var("RAT_DISALLOWED_TOOLS") {
            Ok(disallowed) => {
                if !disallowed.trim().is_empty() {
                    extra_args.push("--disallowedTools".into());
                    extra_args.push(disallowed);
                }
            }
            Err(_) => {
                let disallowed_default = "Read,Write,Edit,MultiEdit";
                extra_args.push("--disallowedTools".into());
                extra_args.push(disallowed_default.into());
            }
        }
    }

    info!(
        "🔧 LOCAL DEV: Starting ACP agent: {} {} {}",
        path.display(),
        args_vec.join(" "),
        if extra_args.is_empty() { String::new() } else { format!("{}", extra_args.join(" ")) }
    );
    let mut command = Command::new(path);
    command
        .args(args_vec)
        .args(extra_args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());
    if let Some(envs) = env_map {
        command.envs(envs);
    }
    let mut child = command.spawn()?;
    let child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get agent stdin"))?;
    let mut child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get agent stdout"))?;

    // Share stdin between tasks for local handling of fs/* RPCs
    let child_stdin = std::sync::Arc::new(tokio::sync::Mutex::new(child_stdin));
    // Share WS writer across tasks
    let ws_writer = std::sync::Arc::new(tokio::sync::Mutex::new(ws_write));

    // Task: WS -> agent stdin (direct pass-through, no encryption)
    let stdin_for_ws = child_stdin.clone();
    let perms_for_ws = pending_perms.clone();
    let ws_to_agent = tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Intercept permission responses addressed to local bridge
                    let mut intercepted = false;
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        let is_response = v.get("method").is_none() && v.get("id").is_some();
                        if is_response {
                            let id_str = id_key(&v["id"]).unwrap_or_default();
                            if let Some(tx) = perms_for_ws.lock().await.remove(&id_str) {
                                let mut allowed = false;
                                if let Some(res) = v.get("result") {
                                    if res.get("outcome").and_then(|o| o.get("cancelled")).and_then(|b| b.as_bool()) == Some(true) {
                                        allowed = false;
                                    } else if let Some(sel) = res.get("outcome").and_then(|o| o.get("selected")) {
                                        let opt = sel.get("optionId").and_then(|s| s.as_str()).unwrap_or("").to_lowercase();
                                        // Accept common allow variants (ACP may use allow/allow_once/allow_always)
                                        allowed = opt == "allow"
                                            || opt.starts_with("allow")
                                            || opt == "yes"
                                            || opt == "ok"
                                            || opt == "approve";
                                        if !allowed {
                                            warn!("🔧 LOCAL DEV: permission response optionId='{}' not recognized as allow; treating as deny", opt);
                                        }
                                    }
                                }
                                let _ = tx.send(allowed);
                                intercepted = true;
                            }
                        }
                    }
                    if intercepted { continue; }
                    if let Err(e) = stdin_for_ws.lock().await.write_all(text.as_bytes()).await {
                        warn!("🔧 LOCAL DEV: stdin write error: {}", e);
                        break;
                    }
                    if let Err(e) = stdin_for_ws.lock().await.write_all(b"\n").await {
                        warn!("🔧 LOCAL DEV: stdin newline write error: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Err(e) = stdin_for_ws.lock().await.write_all(&data).await {
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
    let stdin_for_agent = child_stdin.clone();
    let perms_for_agent = pending_perms.clone();
    let agent_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 8192];
        loop {
            match child_stdout.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let data = &buf[..n];
                    // Try to treat output as NDJSON and intercept fs/* requests locally
                    if let Ok(text) = std::str::from_utf8(data) {
                        for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
                            let maybe_json: Result<serde_json::Value, _> = serde_json::from_str(line);
                            if let Ok(v) = maybe_json {
                                if let Some(m) = v.get("method").and_then(|x| x.as_str()) {
                                    if m == "fs/write_text_file" {
                                        let id = v.get("id").cloned().unwrap_or(serde_json::json!(null));
                                        let id_str = id_key(&id).unwrap_or_else(|| "".into());
                                        let path = v["params"]["path"].as_str().unwrap_or("").to_string();
                                        let content = v["params"]["content"].as_str().unwrap_or("").to_string();

                                        // Prompt the browser for permission before writing
                                        let (tx, rx) = oneshot::channel::<bool>();
                                        perms_for_agent.lock().await.insert(id_str.clone(), tx);
                                        let perm_req = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id_str,
                                            "method": "session/request_permission",
                                            "params": {
                                                "tool": "write_text_file",
                                                "reason": format!("Agent requested to write {}", path),
                                                "options": [
                                                    {"id": "allow", "label": "Allow"},
                                                    {"id": "deny", "label": "Deny"}
                                                ]
                                            }
                                        });
                                        if let Err(e) = ws_writer.lock().await.send(Message::Text(perm_req.to_string())).await { warn!("🔧 LOCAL DEV: ws send perm req error: {}", e); }

                                        // Spawn a task to wait for decision and then perform the write + reply to agent
                                        let stdin_for_agent2 = stdin_for_agent.clone();
                                        tokio::spawn(async move {
                                            let allowed = rx.await.unwrap_or(false);
                                            let resp = if allowed {
                                                // Try to write the file locally
                                                if let Some(parent) = std::path::Path::new(&path).parent() { let _ = tokio::fs::create_dir_all(parent).await; }
                                                match tokio::fs::write(&path, content).await {
                                                    Ok(_) => serde_json::json!({"jsonrpc":"2.0","id": id, "result": {}}),
                                                    Err(e) => serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to write {}: {}", path, e)}}),
                                                }
                                            } else {
                                                serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": "permission denied"}})
                                            };
                                            let s = resp.to_string() + "\n";
                                            if let Err(e) = stdin_for_agent2.lock().await.write_all(s.as_bytes()).await { warn!("🔧 LOCAL DEV: reply write error: {}", e); }
                                        });
                                        // Do not forward this request to the browser
                                        continue;
                                    } else if m == "fs/read_text_file" {
                                        let id = v.get("id").cloned().unwrap_or(serde_json::json!(null));
                                        let path = v["params"]["path"].as_str().unwrap_or("").to_string();
                                        let result = tokio::fs::read_to_string(&path).await;
                                        let resp = match result {
                                            Ok(content) => serde_json::json!({"jsonrpc":"2.0","id": id, "result": {"content": content}}),
                                            Err(_) => serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to read {}", path)}}),
                                        };
                                        let s = resp.to_string() + "\n";
                                        if let Err(e) = stdin_for_agent.lock().await.write_all(s.as_bytes()).await {
                                            warn!("🔧 LOCAL DEV: reply write error: {}", e);
                                        }
                                        continue;
                                    } else if m == "fs/mkdir" || m == "fs/create_dir" {
                                        let id = v.get("id").cloned().unwrap_or(serde_json::json!(null));
                                        let id_str = id_key(&id).unwrap_or_else(|| "".into());
                                        let path = v["params"]["path"].as_str().unwrap_or("").to_string();
                                        let (tx, rx) = oneshot::channel::<bool>();
                                        perms_for_agent.lock().await.insert(id_str.clone(), tx);
                                        let perm_req = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id_str,
                                            "method": "session/request_permission",
                                            "params": {"tool":"mkdir","reason": format!("Agent requested to create directory {}", path), "options":[{"id":"allow"},{"id":"deny"}]}
                                        });
                                        let _ = ws_writer.lock().await.send(Message::Text(perm_req.to_string())).await;
                                        let stdin_for_agent2 = stdin_for_agent.clone();
                                        tokio::spawn(async move {
                                            let allowed = rx.await.unwrap_or(false);
                                            let resp = if allowed {
                                                match tokio::fs::create_dir_all(&path).await {
                                                    Ok(_) => serde_json::json!({"jsonrpc":"2.0","id": id, "result": {}}),
                                                    Err(e) => serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to mkdir {}: {}", path, e)}})
                                                }
                                            } else { serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": "permission denied"}}) };
                                            let s = resp.to_string() + "\n";
                                            let _ = stdin_for_agent2.lock().await.write_all(s.as_bytes()).await;
                                        });
                                        continue;
                                    } else if m == "fs/delete_file" || m == "fs/remove_file" {
                                        let id = v.get("id").cloned().unwrap_or(serde_json::json!(null));
                                        let id_str = id_key(&id).unwrap_or_else(|| "".into());
                                        let path = v["params"]["path"].as_str().unwrap_or("").to_string();
                                        let (tx, rx) = oneshot::channel::<bool>();
                                        perms_for_agent.lock().await.insert(id_str.clone(), tx);
                                        let perm_req = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id_str,
                                            "method": "session/request_permission",
                                            "params": {"tool":"delete_file","reason": format!("Agent requested to delete {}", path), "options":[{"id":"allow"},{"id":"deny"}]}
                                        });
                                        let _ = ws_writer.lock().await.send(Message::Text(perm_req.to_string())).await;
                                        let stdin_for_agent2 = stdin_for_agent.clone();
                                        tokio::spawn(async move {
                                            let allowed = rx.await.unwrap_or(false);
                                            let resp = if allowed {
                                                match tokio::fs::remove_file(&path).await {
                                                    Ok(_) => serde_json::json!({"jsonrpc":"2.0","id": id, "result": {}}),
                                                    Err(e) => serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to delete {}: {}", path, e)}})
                                                }
                                            } else { serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": "permission denied"}}) };
                                            let s = resp.to_string() + "\n";
                                            let _ = stdin_for_agent2.lock().await.write_all(s.as_bytes()).await;
                                        });
                                        continue;
                                    } else if m == "fs/rename" || m == "fs/move" {
                                        let id = v.get("id").cloned().unwrap_or(serde_json::json!(null));
                                        let id_str = id_key(&id).unwrap_or_else(|| "".into());
                                        let from = v["params"]["from"].as_str().unwrap_or("").to_string();
                                        let to = v["params"]["to"].as_str().unwrap_or("").to_string();
                                        let (tx, rx) = oneshot::channel::<bool>();
                                        perms_for_agent.lock().await.insert(id_str.clone(), tx);
                                        let perm_req = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id_str,
                                            "method": "session/request_permission",
                                            "params": {"tool":"rename","reason": format!("Agent requested to rename {} -> {}", from, to), "options":[{"id":"allow"},{"id":"deny"}]}
                                        });
                                        let _ = ws_writer.lock().await.send(Message::Text(perm_req.to_string())).await;
                                        let stdin_for_agent2 = stdin_for_agent.clone();
                                        tokio::spawn(async move {
                                            let allowed = rx.await.unwrap_or(false);
                                            let resp = if allowed {
                                                match tokio::fs::rename(&from, &to).await {
                                                    Ok(_) => serde_json::json!({"jsonrpc":"2.0","id": id, "result": {}}),
                                                    Err(e) => serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to rename {} -> {}: {}", from, to, e)}})
                                                }
                                            } else { serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": "permission denied"}}) };
                                            let s = resp.to_string() + "\n";
                                        let _ = stdin_for_agent2.lock().await.write_all(s.as_bytes()).await;
                                        });
                                        continue;
                                    } else if m == "terminal/execute" {
                                        // Prompt and execute command locally, stream output to browser, send result to agent
                                        let id = v.get("id").cloned().unwrap_or(serde_json::json!(null));
                                        let id_str = id_key(&id).unwrap_or_else(|| "".into());
                                        let cmd = v["params"]["cmd"].as_str().unwrap_or("").to_string();
                                        let args: Vec<String> = v["params"]["args"].as_array()
                                            .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                                            .unwrap_or_else(|| vec![]);
                                        let cwd = v["params"]["cwd"].as_str().map(|s| s.to_string());
                                        if cmd.is_empty() {
                                            let resp = serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32602, "message": "terminal/execute missing cmd"}});
                                            let _ = stdin_for_agent.lock().await.write_all((resp.to_string()+"\n").as_bytes()).await;
                                            continue;
                                        }
                                        let (tx, rx) = oneshot::channel::<bool>();
                                        perms_for_agent.lock().await.insert(id_str.clone(), tx);
                                        let perm_req = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id_str,
                                            "method": "session/request_permission",
                                            "params": {"tool":"terminal_execute","reason": format!("Agent requested to run: {} {}", cmd, args.join(" ")), "options":[{"id":"allow"},{"id":"deny"}]}
                                        });
                                        let _ = ws_writer.lock().await.send(Message::Text(perm_req.to_string())).await;

                                        let stdin_for_agent2 = stdin_for_agent.clone();
                                        let ws_write2 = ws_writer.clone();
                                        tokio::spawn(async move {
                                            let allowed = rx.await.unwrap_or(false);
                                            if !allowed {
                                                let resp = serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": "permission denied"}});
                                                let _ = stdin_for_agent2.lock().await.write_all((resp.to_string()+"\n").as_bytes()).await;
                                                return;
                                            }
                                            let mut c = Command::new(&cmd);
                                            c.args(&args)
                                                .stdin(std::process::Stdio::null())
                                                .stdout(std::process::Stdio::piped())
                                                .stderr(std::process::Stdio::piped());
                                            if let Some(ref d) = cwd { c.current_dir(d); }
                                            match c.spawn() {
                                                Ok(mut child) => {
                                                    // Stream stdout
                                                    if let Some(mut out) = child.stdout.take() {
                                                        let mut rdr = tokio::io::BufReader::new(out);
                                                        loop {
                                                            let mut line = String::new();
                                                            match rdr.read_line(&mut line).await {
                                                                Ok(0) => break,
                                                                Ok(_) => {
                                                                    let term = serde_json::json!({"jsonrpc":"2.0","method":"terminal/output","params": {"stream":"stdout","line": line.trim_end()}});
                                                                    let _ = ws_write2.lock().await.send(Message::Text(term.to_string())).await;
                                                                }
                                                                Err(_) => break,
                                                            }
                                                        }
                                                    }
                                                    // Stream stderr
                                                    if let Some(mut err) = child.stderr.take() {
                                                        let mut rdr = tokio::io::BufReader::new(err);
                                                        loop {
                                                            let mut line = String::new();
                                                            match rdr.read_line(&mut line).await {
                                                                Ok(0) => break,
                                                                Ok(_) => {
                                                                    let term = serde_json::json!({"jsonrpc":"2.0","method":"terminal/output","params": {"stream":"stderr","line": line.trim_end()}});
                                                                    let _ = ws_write2.lock().await.send(Message::Text(term.to_string())).await;
                                                                }
                                                                Err(_) => break,
                                                            }
                                                        }
                                                    }
                                                    let status = child.wait().await;
                                                    let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
                                                    let resp = serde_json::json!({"jsonrpc":"2.0","id": id, "result": {"exitCode": code }});
                                                    let _ = stdin_for_agent2.lock().await.write_all((resp.to_string()+"\n").as_bytes()).await;
                                                }
                                                Err(e) => {
                                                    let resp = serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to spawn {}: {}", cmd, e)}});
                                                    let _ = stdin_for_agent2.lock().await.write_all((resp.to_string()+"\n").as_bytes()).await;
                                                }
                                            }
                                        });
                                        continue;
                                    }
                                }
                            }
                            // Forward non-intercepted lines to the browser
                            if let Err(e) = ws_writer.lock().await.send(Message::Text(line.to_string())).await {
                                warn!("🔧 LOCAL DEV: ws send error: {}", e);
                                break;
                            }
                        }
                    } else {
                        if let Err(e) = ws_writer.lock().await.send(Message::Binary(data.to_vec())).await {
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
