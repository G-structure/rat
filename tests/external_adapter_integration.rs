use rat::acp::AcpClient;
use rat::app::AppMessage;
use tokio::sync::mpsc;

#[tokio::test]
async fn acp_client_can_talk_to_sim_agent_via_cargo() {
    // Build a client that runs the example via cargo
    let (tx, mut _rx) = mpsc::unbounded_channel::<AppMessage>();
    let mut client = AcpClient::new(
        "sim",
        "cargo",
        vec![
            "run".into(),
            "--quiet".into(),
            "--example".into(),
            "sim_agent".into(),
            "--".into(),
            "--scenario".into(),
            "happy-path-edit".into(),
            "--speed".into(),
            "max".into(),
        ],
        None,
        tx,
        None,
    );

    client.start().await.expect("start acp client");
    let sid = client.create_session().await.expect("create session");
    client
        .send_message(&sid, "Hello".to_string())
        .await
        .expect("send message");

    client.stop().await.expect("stop acp client");
}

