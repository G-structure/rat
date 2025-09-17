use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use log::{info, error};
use reqwest::Client;
use serde_json::json;
use snow::{Builder, params::NoiseParams};
use std::env;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async_with_config, tungstenite::Message};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::http::header::{HeaderName, HeaderValue};
// Use snow to manage Noise keypair; avoid direct x25519_dalek here
use url::Url;

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
    let res = client.post(url)
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

    // Noise XX initiator (MVP stub): build if possible, but do not fail pairing if unavailable
    if let Ok(noise_params) = "Noise_XX_25519_ChaChaPoly_BLAKE2s".parse::<NoiseParams>() {
        let _ = Builder::new(noise_params)
            .prologue(b"rat-acp-pairing")
            .local_private_key(&keypair.private)
            .build_initiator();
    }

    // Send Noise init (subprotocol already negotiated)
    let init_msg = json!({
        "type": "noise_init",
        "role": "initiator",
        "pubkey": pubkey_b64
    });
    ws_write.send(Message::Text(init_msg.to_string())).await?;

    // Send a small binary probe to validate relay bridge (MVP)
    let _ = ws_write.send(Message::Binary(vec![0xA5, 0x5A])).await;

    // Handle Noise handshake & relay ACP (stub: loop reading messages)
    while let Some(msg) = ws_read.next().await {
        match msg? {
            Message::Text(text) => {
                info!("Received text: {}", text);  // Handle noise_resp, then ciphertext
                // In full impl: Process Noise handshake, then encrypt ACP messages
            }
            Message::Binary(data) => {
                info!("Received ciphertext: {} bytes", data.len());  // Passthrough ACP
                // Decrypt with noise, handle ACP, re-encrypt if needed
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    info!("Pairing session ended.");
    Ok(())
}
