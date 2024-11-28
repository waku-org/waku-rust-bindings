use std::io::Error;
use std::str::from_utf8;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use waku::{
    general::pubsubtopic::PubsubTopic, waku_new, Encoding, Event, LibwakuResponse,
    WakuContentTopic, WakuMessage, WakuNodeConfig,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let node1 = waku_new(Some(WakuNodeConfig {
        tcp_port: Some(60010), // TODO: use any available port.
        ..Default::default()
    }))
    .expect("should instantiate");

    let node2 = waku_new(Some(WakuNodeConfig {
        tcp_port: Some(60020), // TODO: use any available port.
        ..Default::default()
    }))
    .expect("should instantiate");

    // ========================================================================
    // Setting an event callback to be executed each time a message is received
    node2
        .set_event_callback(|response| {
            if let LibwakuResponse::Success(v) = response {
                let event: Event =
                    serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

                match event {
                    Event::WakuMessage(evt) => {
                        println!("WakuMessage event received: {:?}", evt.waku_message);
                        let message = evt.waku_message;
                        let payload = message.payload.to_vec();
                        let msg = from_utf8(&payload).expect("should be valid message");
                        println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                        println!("Message Received in NODE 2: {}", msg);
                        println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                    }
                    Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
                    _ => panic!("event case not expected"),
                };
            }
        })
        .expect("set event call back working");

    node1
        .set_event_callback(|response| {
            if let LibwakuResponse::Success(v) = response {
                let event: Event =
                    serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

                match event {
                    Event::WakuMessage(evt) => {
                        println!("WakuMessage event received: {:?}", evt.waku_message);
                        let message = evt.waku_message;
                        let payload = message.payload.to_vec();
                        let msg = from_utf8(&payload).expect("should be valid message");
                        println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                        println!("Message Received in NODE 1: {}", msg);
                        println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                    }
                    Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
                    _ => panic!("event case not expected"),
                };
            }
        })
        .expect("set event call back working");

    let node1 = node1.start().expect("node1 should start");
    let node2 = node2.start().expect("node2 should start");

    // ========================================================================
    // Subscribe to pubsub topic
    let topic = PubsubTopic::new("test");

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
    node1.waku_destroy().expect("should deallocate");
    node2.waku_destroy().expect("should deallocate");

    Ok(())
}
