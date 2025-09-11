use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser, ValueEnum};
use serde_json::{json, Value};
use std::io::Write;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Clone, Debug, ValueEnum)]
enum Scenario {
    HappyPathEdit,
    FailurePath,
    ImagesAndThoughts,
    CommandsUpdate,
}

#[derive(Parser, Debug)]
#[command(name = "sim_agent", about = "Deterministic ACP simulator over stdio")]
struct Cli {
    #[arg(long, value_enum, default_value_t = Scenario::HappyPathEdit)]
    scenario: Scenario,

    #[arg(long, default_value = "fast")] // slomo|normal|fast|max or numeric multiplier later
    speed: String,

    #[arg(long, default_value_t = 0)]
    seed: u64,

    #[arg(long, action = ArgAction::SetFalse)]
    loop_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let speed_mult = parse_speed(&cli.speed);
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let mut reader = BufReader::new(&mut stdin);
    let mut line = String::new();

    let mut next_session_id: u64 = 1;
    let mut active_session: Option<String> = None;
    let mut cancelling: bool = false;

    
    
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let v: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = v.get("id").cloned();
        let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("");
        match method {
            "initialize" => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": 1,
                        "agentCapabilities": {
                            "loadSession": true,
                            "promptCapabilities": {
                                "image": true,
                                "audio": false,
                                "embeddedContext": true
                            }
                        },
                        "authMethods": []
                    }
                });
                write_json(&mut stdout, &resp).await?;
            }
            "authenticate" => {
                let resp = json!({"jsonrpc":"2.0","id": id, "result": Value::Null});
                write_json(&mut stdout, &resp).await?;
            }
            "session/new" => {
                let sid = format!("sim-{}", next_session_id);
                next_session_id += 1;
                active_session = Some(sid.clone());
                let resp = json!({"jsonrpc":"2.0","id": id, "result": {"sessionId": sid}});
                write_json(&mut stdout, &resp).await?;
            }
            "session/load" => {
                // Minimal: no history; reply null after a beat
                sleep_scaled(50, speed_mult).await;
                let resp = json!({"jsonrpc":"2.0","id": id, "result": Value::Null});
                write_json(&mut stdout, &resp).await?;
            }
            "session/prompt" => {
                cancelling = false;
                let Some(sid) = active_session.clone() else {
                    let err = error_response(id, -32000, "no active session");
                    write_json(&mut stdout, &err).await?;
                    continue;
                };
                match cli.scenario {
                    Scenario::HappyPathEdit => {
                        run_happy_path(&sid, &mut stdout, speed_mult, &mut reader).await?;
                    }
                    Scenario::FailurePath => {
                        run_failure_path(&sid, &mut stdout, speed_mult).await?;
                    }
                    Scenario::ImagesAndThoughts => {
                        run_images_and_thoughts(&sid, &mut stdout, speed_mult).await?;
                    }
                    Scenario::CommandsUpdate => {
                        run_commands_update(&sid, &mut stdout, speed_mult).await?;
                    }
                }
                let stop = json!({
                    "jsonrpc":"2.0","id": id,
                    "result": {"stopReason": if cancelling {"cancelled"} else {"end_turn"}}
                });
                write_json(&mut stdout, &stop).await?;
            }
            "session/cancel" => {
                cancelling = true;
                // No direct response per ACP; the prompt response will carry cancelled stopReason
            }
            _ => {
                // Unknown or notifications; ignore gracefully
            }
        }
    }

    Ok(())
}

fn parse_speed(s: &str) -> f32 {
    match s {
        "slomo" => 0.25,
        "normal" => 1.0,
        "fast" => 2.0,
        "max" => 100.0,
        _ => s.parse::<f32>().unwrap_or(1.0),
    }
}

async fn write_json<W: AsyncWriteExt + Unpin>(mut w: W, v: &Value) -> Result<()> {
    let s = serde_json::to_string(v)?;
    w.write_all(s.as_bytes()).await?;
    w.write_all(b"\n").await?;
    w.flush().await?;
    Ok(())
}

fn error_response(id: Option<Value>, code: i64, msg: &str) -> Value {
    json!({"jsonrpc":"2.0","id": id, "error": {"code": code, "message": msg}})
}

async fn sleep_scaled(ms: u64, speed: f32) {
    let scaled = if speed <= 0.0 { ms } else { ((ms as f32) / speed) as u64 };
    if scaled == 0 {
        tokio::task::yield_now().await;
    } else {
        tokio::time::sleep(Duration::from_millis(scaled)).await;
    }
}

async fn send_update(stdout: &mut (impl AsyncWriteExt + Unpin), session_id: &str, update: Value) -> Result<()> {
    let notif = json!({
        "jsonrpc":"2.0",
        "method":"session/update",
        "params": {
            "sessionId": session_id,
            "update": update
        }
    });
    write_json(stdout, &notif).await
}

async fn run_happy_path(
    session_id: &str,
    stdout: &mut (impl AsyncWriteExt + Unpin),
    speed: f32,
    reader: &mut BufReader<&mut tokio::io::Stdin>,
) -> Result<()> {
    // Plan
    let plan = json!({
        "type":"plan",
        "plan": {
            "entries": [
                {"content":"Open file src/lib.rs","priority":"medium","status":"in_progress"},
                {"content":"Apply small refactor","priority":"low","status":"pending"}
            ]
        }
    });
    send_update(stdout, session_id, plan).await?;
    sleep_scaled(120, speed).await;

    // Tool call announce with diff
    let tool_call_id = "call_edit_1";
    let diff = json!({
        "type":"diff",
        "path":"/workspace/src/lib.rs",
        "oldText": null,
        "newText":"pub fn greet() -> &'static str {\n    \"hello, world\"\n}\n"
    });
    let tool_call = json!({
        "type":"tool_call",
        "toolCall": {
            "toolCallId": tool_call_id,
            "title":"Edit src/lib.rs",
            "kind":"edit",
            "status":"in_progress",
            "content":[ {"type":"diff", "path": "/workspace/src/lib.rs", "oldText": null, "newText": "pub fn greet() -> &'static str {\n    \\\"hello, world\\\"\n}\n"} ],
            "locations":[{"path":"/workspace/src/lib.rs","line":1}]
        }
    });
    send_update(stdout, session_id, tool_call).await?;
    sleep_scaled(60, speed).await;

    // Request permission from client before completing
    let perm_req = json!({
        "jsonrpc":"2.0",
        "id": 42,
        "method":"session/request_permission",
        "params":{
            "sessionId": session_id,
            "toolCall": {"toolCallId": tool_call_id},
            "options":[
                {"optionId":"allow-once","name":"Allow once","kind":"allow_once"},
                {"optionId":"reject-once","name":"Reject","kind":"reject_once"}
            ]
        }
    });
    write_json(&mut *stdout, &perm_req).await?;

    // Wait for a matching response id=42
    let mut buf = String::new();
    loop {
        buf.clear();
        let n = reader.read_line(&mut buf).await?;
        if n == 0 { break; }
        let Ok(v) = serde_json::from_str::<Value>(buf.trim()) else { continue };
        if v.get("id").and_then(|x| x.as_i64()) == Some(42) {
            break;
        }
    }

    // Tool call complete
    let tc_update = json!({
        "type":"tool_call_update",
        "toolCallUpdate": {"toolCallId": tool_call_id, "status":"completed"}
    });
    send_update(stdout, session_id, tc_update).await?;
    sleep_scaled(80, speed).await;

    // Agent message chunks
    let chunk1 = json!({"type":"agent_message_chunk","chunk":{"type":"text","text":"Applied the change to src/lib.rs. "}});
    let chunk2 = json!({"type":"agent_message_chunk","chunk":{"type":"text","text":"Anything else?"}});
    send_update(stdout, session_id, chunk1).await?;
    sleep_scaled(40, speed).await;
    send_update(stdout, session_id, chunk2).await?;

    Ok(())
}

async fn run_failure_path(
    session_id: &str,
    stdout: &mut (impl AsyncWriteExt + Unpin),
    speed: f32,
) -> Result<()> {
    let tool_call_id = "call_fail_1";
    let tool_call = json!({
        "type":"tool_call",
        "toolCall": {
            "toolCallId": tool_call_id,
            "title":"Search project",
            "kind":"search",
            "status":"in_progress",
            "content":[ {"type":"content", "content": {"type":"text","text":"Searching..."}} ]
        }
    });
    send_update(stdout, session_id, tool_call).await?;
    sleep_scaled(60, speed).await;
    let tc_update = json!({
        "type":"tool_call_update",
        "toolCallUpdate": {"toolCallId": tool_call_id, "status":"failed", "content":[{"type":"content","content":{"type":"text","text":"No matches found"}}]}
    });
    send_update(stdout, session_id, tc_update).await?;
    let explain = json!({"type":"agent_message_chunk","chunk":{"type":"text","text":"Search failed; please refine your query."}});
    sleep_scaled(40, speed).await;
    send_update(stdout, session_id, explain).await?;
    Ok(())
}

async fn run_images_and_thoughts(
    session_id: &str,
    stdout: &mut (impl AsyncWriteExt + Unpin),
    speed: f32,
) -> Result<()> {
    let thought1 = json!({"type":"agent_thought_chunk","chunk":{"type":"text","text":"Hmm, outlining approach…"}});
    send_update(stdout, session_id, thought1).await?;
    sleep_scaled(50, speed).await;
    // A tiny 1x1 PNG (transparent) base64 data URL-ish payload; keep it short.
    let img_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAuMB9s8O8d8AAAAASUVORK5CYII=";
    let img_chunk = json!({
        "type":"agent_message_chunk",
        "chunk": {"type":"image","data": img_data, "mimeType":"image/png"}
    });
    send_update(stdout, session_id, img_chunk).await?;
    sleep_scaled(40, speed).await;
    let msg = json!({"type":"agent_message_chunk","chunk":{"type":"text","text":"Here’s a quick sketch."}});
    send_update(stdout, session_id, msg).await?;
    Ok(())
}

async fn run_commands_update(
    session_id: &str,
    stdout: &mut (impl AsyncWriteExt + Unpin),
    speed: f32,
) -> Result<()> {
    let cmds = json!({
        "type":"available_commands_update",
        "availableCommandsUpdate": {
            "commands": [
                {"id":"new-session","name":"New Session","description":"Create a new session"},
                {"id":"cancel-turn","name":"Cancel Turn","description":"Send session/cancel"}
            ]
        }
    });
    send_update(stdout, session_id, cmds).await?;
    sleep_scaled(20, speed).await;
    let msg = json!({"type":"agent_message_chunk","chunk":{"type":"text","text":"Commands updated."}});
    send_update(stdout, session_id, msg).await?;
    Ok(())
}
