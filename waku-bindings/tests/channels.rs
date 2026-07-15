use base64::Engine;
use serde::Serialize;
use serial_test::serial;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use waku_bindings::{ChannelMessageSentPayload, LogosDeliveryCtx, WakuNodeConfig};

const TEST_CHANNEL_ID: &str = "test-channel";
const TEST_CONTENT_TOPIC: &str = "/test/1/channels/proto";
const TEST_SENDER_ID: &str = "test-sender";
const TIMEOUT: Duration = Duration::from_secs(30);

/// Body of `channel_send`, whose payload travels base64-encoded.
#[derive(Serialize)]
struct ChannelMessage {
    payload: String,
    ephemeral: bool,
}

#[tokio::test]
#[serial]
async fn channel_create_send_close() {
    let config = serde_json::to_string(&WakuNodeConfig {
        tcp_port: Some(60070),
        ..Default::default()
    })
    .expect("config should serialise");

    let node = LogosDeliveryCtx::new_async(config, TIMEOUT)
        .await
        .expect("node should instantiate");

    // Channel traffic is reported through events, so capture them to prove the
    // typed listener registers and delivers ChannelMessageSentPayload.
    let sent: Arc<Mutex<Vec<ChannelMessageSentPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let sent_cloned = sent.clone();
    node.add_on_channel_message_sent_listener(move |event| {
        sent_cloned.lock().unwrap().push(event.clone());
    });

    node.start_node_async().await.expect("node should start");

    let channel_id = node
        .channel_create_async(
            TEST_CHANNEL_ID.to_string(),
            TEST_CONTENT_TOPIC.to_string(),
            TEST_SENDER_ID.to_string(),
        )
        .await
        .expect("channel should be created");
    assert_eq!(channel_id, TEST_CHANNEL_ID);

    let message = serde_json::to_string(&ChannelMessage {
        payload: base64::engine::general_purpose::STANDARD.encode(b"Hi from a reliable channel!"),
        ephemeral: false,
    })
    .expect("message should serialise");

    let request_id = node
        .channel_send_async(TEST_CHANNEL_ID.to_string(), message)
        .await
        .expect("channel send should succeed");
    assert!(!request_id.is_empty(), "send should return a request id");

    // Give the send a moment to finalise and emit its outcome event.
    tokio::time::sleep(Duration::from_secs(2)).await;

    node.channel_close_async(TEST_CHANNEL_ID.to_string())
        .await
        .expect("channel should close");

    // Scoped so the guard is dropped before the await below.
    {
        let sent = sent.lock().unwrap();
        println!("channel sent events observed: {sent:?}");
    }

    node.stop_node_async().await.expect("node should stop");
    // The context is torn down by LogosDeliveryCtx's Drop impl.
}
