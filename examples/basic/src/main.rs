use base64::Engine;
use std::io::Error;
use std::str::from_utf8;
use tokio::time::{sleep, Duration};
use waku::{
    Encoding, LogosDeliveryCtx, ReceivedMessagePayload, WakuContentTopic, WakuMessage,
    WakuNodeConfig,
};

const TOPIC: &str = "test";
const TIMEOUT: Duration = Duration::from_secs(30);

fn new_node(tcp_port: usize) -> LogosDeliveryCtx {
    let config = serde_json::to_string(&WakuNodeConfig {
        tcp_port: Some(tcp_port),
        ..Default::default()
    })
    .expect("config should serialise");

    LogosDeliveryCtx::create(config, TIMEOUT).expect("should instantiate")
}

fn print_received(node_name: &'static str) -> impl Fn(&ReceivedMessagePayload) {
    move |event| {
        let payload = base64::engine::general_purpose::STANDARD
            .decode(&event.waku_message.payload)
            .expect("payload should be base64");
        let msg = from_utf8(&payload).expect("should be valid message");
        println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
        println!("Message Received in {node_name}: {msg}");
        println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let node1 = new_node(60010); // TODO: use any available port.
    let node2 = new_node(60020); // TODO: use any available port.

    // ========================================================================
    // Registering a listener to be executed each time a message is received
    node2.add_on_received_message_listener(print_received("NODE 2"));
    node1.add_on_received_message_listener(print_received("NODE 1"));

    node1.start_node_async().await.expect("node1 should start");
    node2.start_node_async().await.expect("node2 should start");

    // ========================================================================
    // Subscribe to pubsub topic

    node1
        .waku_relay_subscribe_async(TOPIC.to_string())
        .await
        .expect("node1 should subscribe");

    node2
        .waku_relay_subscribe_async(TOPIC.to_string())
        .await
        .expect("node2 should subscribe");

    // ========================================================================
    // Connect nodes with each other

    let addresses2 = node2
        .waku_listen_addresses_async()
        .await
        .expect("should obtain the addresses");
    let address2 = addresses2
        .split(',')
        .next()
        .expect("node2 should report a listen address");

    node1
        .waku_connect_async(address2.to_string(), 10_000)
        .await
        .expect("node1 should connect to node2");

    // ========================================================================
    // Wait for gossipsub mesh to form

    sleep(Duration::from_secs(2)).await;

    // ========================================================================
    // Publish a message

    let content_topic = WakuContentTopic::new("waku", "2", "test", Encoding::Proto);
    let message = WakuMessage::new("Hello world", content_topic, 0, Vec::new(), false);
    let message = serde_json::to_string(&message).expect("message should serialise");
    node1
        .waku_relay_publish_async(TOPIC.to_string(), message, 10_000)
        .await
        .expect("should have sent the message");

    // ========================================================================
    // Waiting for message to arrive

    sleep(Duration::from_secs(1)).await;

    // ========================================================================
    // Stop both instances

    node1.stop_node_async().await.expect("should stop");
    node2.stop_node_async().await.expect("should stop");

    // Resources are freed by LogosDeliveryCtx's Drop impl.

    Ok(())
}
