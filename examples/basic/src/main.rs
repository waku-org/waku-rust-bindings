use std::io::Error;
use std::str::from_utf8;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use waku::{
    waku_destroy, waku_new, Encoding, Event, WakuContentTopic, WakuMessage, WakuNodeConfig,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let node1 = waku_new(Some(WakuNodeConfig {
        port: Some(60010), // TODO: use any available port.
        ..Default::default()
    }))
    .expect("should instantiate");

    let node2 = waku_new(Some(WakuNodeConfig {
        port: Some(60020), // TODO: use any available port.
        ..Default::default()
    }))
    .expect("should instantiate");

    let node1 = node1.start().expect("node1 should start");
    let node2 = node2.start().expect("node2 should start");

    // ========================================================================
    // Setting an event callback to be executed each time a message is received
    node2.set_event_callback(move |event| {
        if let Event::WakuMessage(message) = event {
            let message = message.waku_message;
            let payload = message.payload.to_vec();
            let msg = from_utf8(&payload).expect("should be valid message");
            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
            println!("Message Received in NODE 2: {}", msg);
            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
        }
    });

    node1.set_event_callback(move |event| {
        if let Event::WakuMessage(message) = event {
            let message = message.waku_message;
            let payload = message.payload.to_vec();
            let msg = from_utf8(&payload).expect("should be valid message");
            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
            println!("Message Received in NODE 1: {}", msg);
            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
        }
    });

    // ========================================================================
    // Subscribe to pubsub topic
    let topic = "test".to_string();

    node1
        .relay_subscribe(&topic)
        .expect("node1 should subscribe");

    node2
        .relay_subscribe(&topic)
        .expect("node2 should subscribe");

    // ========================================================================
    // Connect nodes with each other

    let addresses2 = node2
        .listen_addresses()
        .expect("should obtain the addresses");

    node1
        .connect(&addresses2[0], None)
        .expect("node1 should connect to node2");

    // ========================================================================
    // Wait for gossipsub mesh to form

    sleep(Duration::from_secs(2)).await;

    // ========================================================================
    // Publish a message

    let content_topic = WakuContentTopic::new("waku", "2", "test", Encoding::Proto);
    let message = WakuMessage::new(
        "Hello world",
        content_topic,
        0,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap(),
        Vec::new(),
        false,
    );
    node1
        .relay_publish_message(&message, &topic, None)
        .expect("should have sent the message");

    // ========================================================================
    // Waiting for message to arrive

    sleep(Duration::from_secs(1)).await;

    // ========================================================================
    // Stop both instances

    let node1 = node1.stop().expect("should stop");
    let node2 = node2.stop().expect("should stop");

    // ========================================================================
    // Free resources
    waku_destroy(node1).expect("should deallocate");
    waku_destroy(node2).expect("should deallocate");

    Ok(())
}
