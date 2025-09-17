use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use snow::{params::NoiseParams, Builder};
use std::env;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::{HeaderName, HeaderValue};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::{connect_async_with_config, tungstenite::Message};
use url::Url;

fn b64u(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}
fn b64u_decode(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s.as_bytes())
        .ok()
}

pub async fn start_pairing() -> Result<()> {
    env_logger::init();
    let relay_url = env::var("RAT_RELAY_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Generate Noise static keypair using snow
    let params = "Noise_IK_25519_ChaChaPoly_BLAKE2s".parse::<NoiseParams>()?;
    let builder = Builder::new(params);
    let keypair = builder.generate_keypair()?;
    let static_kp = keypair.public.to_vec();
    let pubkey_b64 = general_purpose::STANDARD.encode(static_kp);

    info!("Generated RAT pubkey: {}", pubkey_b64);

    // POST /v1/pair/start
    let client = Client::new();
    let url = Url::parse(&format!("{}/v1/pair/start", relay_url))?;
    let res = client
        .post(url)
        .json(&json!({
            "rat_pubkey": pubkey_b64,
            "caps": ["acp", "fs", "tools"],
            "rat_version": env!("CARGO_PKG_VERSION")
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Pair start failed: {}", res.status()));
    }

    let pair_info: serde_json::Value = res.json().await?;
    let user_code = pair_info["user_code"].as_str().ok_or(anyhow::anyhow!("No user_code"))?;
    let device_code = pair_info["device_code"].as_str().ok_or(anyhow::anyhow!("No device_code"))?;
    let relay_ws_url = pair_info["relay_ws_url"].as_str().ok_or(anyhow::anyhow!("No relay_ws_url"))?;

    info!("Pairing started. User code: {}. Enter on hosted UI.", user_code);
    info!("Device code (internal): {}", device_code);

    // Connect WS to relay with device_code and explicit subprotocol per spec
    let ws_url = format!("{}?device_code={}", relay_ws_url, device_code);
    let mut request = ws_url.into_client_request()?;
    let hname = HeaderName::from_static("sec-websocket-protocol");
    // RAT side does not need attach token; advertise baseline subprotocol
    request
        .headers_mut()
        .insert(hname, HeaderValue::from_static("acp.jsonrpc.v1"));
    let ws_cfg = WebSocketConfig::default();
    let (ws_stream, _) = connect_async_with_config(request, Some(ws_cfg), false).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    // Wait for browser to send binding context
    let (session_id, stk_sha256_b64u, protocol) = wait_for_noise_bind(&mut ws_read).await?;
    let prologue = derive_prologue(&session_id, &stk_sha256_b64u, &protocol);

    // Build Noise XX initiator with prologue and static key
    let params = "Noise_XX_25519_ChaChaPoly_BLAKE2s".parse::<NoiseParams>()?;
    let mut hs = Builder::new(params)
        .prologue(&prologue)
        .local_private_key(&keypair.private)
        .build_initiator()?;

    // Act 1 ->
    let mut buf = vec![0u8; 1024];
    let n = hs.write_message(&[], &mut buf)?;
    buf.truncate(n);
    ws_write.send(Message::Binary(buf)).await?;
    // Act 2 <-
    let m2 = read_binary(&mut ws_read).await?;
    let mut tmp = vec![0u8; 1024];
    let _ = hs.read_message(&m2, &mut tmp)?;
    // Act 3 ->
    let mut out = vec![0u8; 1024];
    let n3 = hs.write_message(&[], &mut out)?;
    out.truncate(n3);
    ws_write.send(Message::Binary(out)).await?;

    // Enter transport mode
    let mut ts = hs.into_transport_mode()?;
    info!("Noise XX key established");

    // If ACP is configured, run the bridge using Noise transport
    if std::env::var("RAT2E_AGENT_CMD").is_ok() {
        run_acp_bridge_noise(ws_write, ws_read, ts).await?;
        info!("ACP bridge session ended");
        return Ok(());
    }

    Ok(())
}

fn derive_prologue(session_id: &str, stk_sha256_b64u: &str, protocol: &str) -> Vec<u8> {
    let canon = format!("RAT2E/v1|sid:{}|stk:{}|proto:{}", session_id, stk_sha256_b64u, protocol);
    let mut h = Sha256::new();
    h.update(canon.as_bytes());
    h.finalize().to_vec()
}

async fn wait_for_noise_bind<WR>(
    ws_read: &mut WR,
) -> Result<(String, String, String)>
where
    WR: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while let Some(msg) = ws_read.next().await {
        match msg? {
            Message::Binary(data) => {
                // Allow JSON in binary frame
                if let Ok(text) = String::from_utf8(data) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                        if v.get("type").and_then(|x| x.as_str()) == Some("noise_bind") {
                            let sid = v["session_id"].as_str().unwrap_or("").to_string();
                            let stk = v["attach_token_sha256"].as_str().unwrap_or("").to_string();
                            let proto = v["protocol"].as_str().unwrap_or("acp.jsonrpc.v1").to_string();
                            return Ok((sid, stk, proto));
                        }
                    }
                }
            }
            Message::Text(text) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("noise_bind") {
                        let sid = v["session_id"].as_str().unwrap_or("").to_string();
                        let stk = v["attach_token_sha256"].as_str().unwrap_or("").to_string();
                        let proto = v["protocol"].as_str().unwrap_or("acp.jsonrpc.v1").to_string();
                        return Ok((sid, stk, proto));
                    }
                }
            }
            _ => {}
        }
    }
    Err(anyhow::anyhow!("noise_bind not received"))
}

async fn read_binary<WR>(ws_read: &mut WR) -> Result<Vec<u8>>
where
    WR: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while let Some(msg) = ws_read.next().await {
        if let Message::Binary(data) = msg? {
            return Ok(data);
        }
    }
    Err(anyhow::anyhow!("eof waiting for binary"))
}

async fn run_acp_bridge_noise<WS, WR>(
    mut ws_write: WS,
    mut ws_read: WR,
    ts: snow::TransportState,
) -> Result<()>
where
    WS: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
    WR: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin + Send + 'static,
{
    // Spawn agent process from env
    let cmd = env::var("RAT2E_AGENT_CMD")
        .map_err(|_| anyhow::anyhow!("Set RAT2E_AGENT_CMD to the ACP agent command"))?;
    let args = env::var("RAT2E_AGENT_ARGS").unwrap_or_default();
    let args_vec: Vec<String> = if args.is_empty() {
        vec![]
    } else {
        args.split_whitespace().map(|s| s.to_string()).collect()
    };
    info!("Starting local ACP agent: {} {}", cmd, args_vec.join(" "));
    let mut child = Command::new(cmd)
        .args(args_vec)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get agent stdin"))?;
    let mut child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get agent stdout"))?;

    let ts_ptr = Arc::new(tokio::sync::Mutex::new(ts));

    // Task: WS -> agent stdin (decrypt then write)
    let c_stdin = child_stdin;
    let ts_in = ts_ptr.clone();
    let ws_to_agent = tokio::spawn(async move {
        let mut child_stdin = c_stdin;
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    let mut out = vec![0u8; data.len()];
                    let mut guard = ts_in.lock().await;
                    match guard.read_message(&data, &mut out) {
                        Ok(n) => {
                            out.truncate(n);
                            if let Err(e) = child_stdin.write_all(&out).await {
                                warn!("stdin write error: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("decrypt error: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
    });

    // Task: agent stdout -> WS (read then encrypt and send)
    let ts_out = ts_ptr.clone();
    let agent_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 8192];
        loop {
            match child_stdout.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let mut ct = vec![0u8; n + 16];
                    {
                        let mut guard = ts_out.lock().await;
                        match guard.write_message(&buf[..n], &mut ct) {
                            Ok(m) => ct.truncate(m),
                            Err(e) => {
                                warn!("encrypt error: {}", e);
                                break;
                            }
                        }
                    }
                    if let Err(e) = ws_write.send(Message::Binary(ct)).await {
                        warn!("ws send error: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("stdout read error: {}", e);
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(ws_to_agent, agent_to_ws);
    Ok(())
}

