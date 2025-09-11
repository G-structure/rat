use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn sim_agent_happy_path_jsonrpc() {
    // Spawn the example agent via cargo run to avoid managing binaries
    let mut child = tokio::process::Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--example",
            "sim_agent",
            "--",
            "--scenario",
            "happy-path-edit",
            "--speed",
            "max",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn sim_agent");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // initialize
    let init = json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":1, "clientCapabilities": {"fs":{"readTextFile":true,"writeTextFile":true}}}
    });
    let s = serde_json::to_string(&init).unwrap();
    stdin.write_all(s.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    let v: Value = serde_json::from_str(line.trim()).unwrap();
    assert_eq!(v.get("id").and_then(|x| x.as_i64()), Some(1));
    assert!(v.get("result").is_some());

    // session/new
    line.clear();
    let new_sess = json!({"jsonrpc":"2.0","id":2,"method":"session/new","params":{}});
    stdin
        .write_all(serde_json::to_string(&new_sess).unwrap().as_bytes())
        .await
        .unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();
    line.clear();
    reader.read_line(&mut line).await.unwrap();
    let v: Value = serde_json::from_str(line.trim()).unwrap();
    let sid = v
        .get("result")
        .and_then(|r| r.get("sessionId"))
        .and_then(|s| s.as_str())
        .unwrap()
        .to_string();
    assert!(sid.starts_with("sim-"));

    // prompt
    let prompt = json!({
        "jsonrpc":"2.0","id":3,"method":"session/prompt",
        "params":{"sessionId": sid, "prompt":[{"type":"text","text":"go"}]}
    });
    stdin
        .write_all(serde_json::to_string(&prompt).unwrap().as_bytes())
        .await
        .unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Expect a sequence of updates and a permission request; respond to it
    let mut saw_plan = false;
    let mut saw_tool_call = false;
    let mut saw_tc_completed = false;
    let mut saw_agent_chunks = 0;
    let mut sent_permission_reply = false;

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let Ok(v) = serde_json::from_str::<Value>(trimmed) else { continue };
                if v.get("method").and_then(|m| m.as_str()) == Some("session/update") {
                    let u = v.get("params").and_then(|p| p.get("update")).unwrap();
                    match u.get("type").and_then(|t| t.as_str()) {
                        Some("plan") => saw_plan = true,
                        Some("tool_call") => saw_tool_call = true,
                        Some("tool_call_update") => {
                            if u.get("toolCallUpdate").and_then(|x| x.get("status")).and_then(|s| s.as_str()) == Some("completed") {
                                saw_tc_completed = true;
                            }
                        }
                        Some("agent_message_chunk") => saw_agent_chunks += 1,
                        _ => {}
                    }
                } else if v.get("method").and_then(|m| m.as_str()) == Some("session/request_permission") {
                    // Respond selecting the first option
                    let id = v.get("id").unwrap().clone();
                    let reply = json!({"jsonrpc":"2.0","id": id, "result": {"outcome":{"outcome":"selected","optionId":"allow-once"}}});
                    stdin
                        .write_all(serde_json::to_string(&reply).unwrap().as_bytes())
                        .await
                        .unwrap();
                    stdin.write_all(b"\n").await.unwrap();
                    stdin.flush().await.unwrap();
                    sent_permission_reply = true;
                } else if v.get("id").and_then(|x| x.as_i64()) == Some(3) {
                    // stopReason
                    let stop = v.get("result").and_then(|r| r.get("stopReason")).and_then(|s| s.as_str());
                    assert_eq!(stop, Some("end_turn"));
                    break;
                }
            }
            Err(_) => break,
        }
    }

    assert!(saw_plan && saw_tool_call && saw_tc_completed);
    assert!(saw_agent_chunks >= 1);
    assert!(sent_permission_reply);

    let _ = child.kill().await;
}

