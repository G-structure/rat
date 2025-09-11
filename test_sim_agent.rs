use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[tokio::test]
async fn test_sim_agent_direct() {
    // Start sim_agent directly
    let mut child = Command::new("cargo")
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
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn sim_agent");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send initialize
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true}}}}"#;
    stdin.write_all(init.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Read response
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    println!("Initialize response: {}", line.trim());

    // Send session/new
    let new_session = r#"{"jsonrpc":"2.0","id":2,"method":"session/new","params":{}}"#;
    stdin.write_all(new_session.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Read response
    line.clear();
    reader.read_line(&mut line).await.unwrap();
    println!("New session response: {}", line.trim());

    // Extract session ID
    let session_response: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    let session_id = session_response["result"]["sessionId"].as_str().unwrap();

    // Send prompt
    let prompt = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{{"sessionId":"{}","prompt":[{{"type":"text","text":"create a simple rust function"}}]}}}}"#,
        session_id
    );
    stdin.write_all(prompt.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Read responses
    println!("Reading responses from sim_agent:");
    for i in 0..20 {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                if !line.trim().is_empty() {
                    println!("Response {}: {}", i + 1, line.trim());
                }
            }
            Err(e) => {
                println!("Error reading: {}", e);
                break;
            }
        }
    }

    let _ = child.kill().await;
}