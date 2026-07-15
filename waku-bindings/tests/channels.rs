use serial_test::serial;
use std::sync::{Arc, Mutex};
use waku_bindings::{waku_new, LibwakuResponse, WakuEvent, WakuNodeConfig};

const TEST_CHANNEL_ID: &str = "test-channel";
const TEST_CONTENT_TOPIC: &str = "/test/1/channels/proto";
const TEST_SENDER_ID: &str = "test-sender";

#[tokio::test]
#[serial]
async fn channel_create_send_close() {
    let node = waku_new(Some(WakuNodeConfig {
        tcp_port: Some(60070),
        ..Default::default()
    }))
    .await
    .expect("node should instantiate");

    // Channel traffic is reported through events, so capture them to prove the
    // channel event payloads decode into the WakuEvent variants.
    let events: Arc<Mutex<Vec<WakuEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_cloned = events.clone();
    node.set_event_callback(move |response| {
        if let LibwakuResponse::Success(Some(v)) = response {
            let event: WakuEvent = serde_json::from_str(&v).expect("event should parse");
            events_cloned.lock().unwrap().push(event);
        }
    })
    .expect("event callback should be set");

    let node = node.start().await.expect("node should start");

    let channel_id = node
        .channel_create(TEST_CHANNEL_ID, TEST_CONTENT_TOPIC, TEST_SENDER_ID)
        .await
        .expect("channel should be created");
    assert_eq!(channel_id, TEST_CHANNEL_ID);

    let request_id = node
        .channel_send(TEST_CHANNEL_ID, b"Hi from a reliable channel!", false)
        .await
        .expect("channel send should succeed");
    assert!(!request_id.is_empty(), "send should return a request id");

    // Give the send a moment to finalise and emit its outcome event.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    node.channel_close(TEST_CHANNEL_ID)
        .await
        .expect("channel should close");

    // Every event this node emitted parsed into a known variant rather than
    // falling through to Unrecognized. Scoped so the guard is dropped before the
    // awaits below.
    {
        let events = events.lock().unwrap();
        println!("events observed: {:?}", events);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, WakuEvent::Unrecognized(_))),
            "no event should be Unrecognized, got: {events:?}"
        );
    }

    let node = node.stop().await.expect("node should stop");
    node.waku_destroy().await.expect("node should be destroyed");
}
