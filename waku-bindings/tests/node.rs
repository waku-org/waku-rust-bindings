use secp256k1::SecretKey;
use serial_test::serial;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{collections::HashSet, str::from_utf8};
use tokio::sync::broadcast::{self, Sender};
use tokio::time;
use tokio::time::sleep;
use waku_bindings::{
    waku_new, Encoding, Event, MessageId, WakuContentTopic, WakuMessage, WakuNodeConfig,
    WakuNodeHandle,
};
const ECHO_TIMEOUT: u64 = 10;
const ECHO_MESSAGE: &str = "Hi from 🦀!";
const TEST_PUBSUBTOPIC: &str = "test";

fn try_publish_relay_messages(
    node: &WakuNodeHandle,
    msg: &WakuMessage,
) -> Result<HashSet<MessageId>, String> {
    let topic = TEST_PUBSUBTOPIC.to_string();
    Ok(HashSet::from([
        node.relay_publish_message(msg, &topic, None)?
    ]))
}

#[derive(Debug, Clone)]
struct Response {
    id: MessageId,
    payload: Vec<u8>,
}

fn set_callback(node: &WakuNodeHandle, tx: Sender<Response>) {
    node.set_event_callback(move |event| {
        if let Event::WakuMessage(message) = event {
            let id = message.message_id();
            let message = message.waku_message();
            let payload = message.payload().to_vec();

            tx.send(Response {
                id: id.to_string(),
                payload,
            })
            .expect("send response to the receiver");
        }
    });
}

async fn test_echo_messages(
    node1: &WakuNodeHandle,
    node2: &WakuNodeHandle,
    content: &'static str,
    content_topic: WakuContentTopic,
) {
    let message = WakuMessage::new(
        content,
        content_topic,
        1,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap(),
        Vec::new(),
        false,
    );

    node1.set_event_callback(move |_event| {});

    let (tx, mut rx) = broadcast::channel(1);
    set_callback(node2, tx);

    let mut ids = try_publish_relay_messages(node1, &message).expect("send relay messages");

    while let Ok(res) = rx.recv().await {
        if ids.take(&res.id).is_some() {
            let msg = from_utf8(&res.payload).expect("should be valid message");
            assert_eq!(content, msg);
        }

        if ids.is_empty() {
            break;
        }
    }
}

#[tokio::test]
#[serial]
async fn default_echo() -> Result<(), String> {
    let node1 = waku_new(Some(WakuNodeConfig {
        port: Some(60010),
        ..Default::default()
    }))?;
    let node2 = waku_new(Some(WakuNodeConfig {
        port: Some(60020),
        ..Default::default()
    }))?;

    node1.start()?;
    node2.start()?;

    let addresses1 = node1.listen_addresses()?;
    node2.connect(&addresses1[0], None)?;

    let topic = TEST_PUBSUBTOPIC.to_string();

    node1.relay_subscribe(&topic)?;
    node2.relay_subscribe(&topic)?;

    // Wait for mesh to form
    sleep(Duration::from_secs(5)).await;

    let content_topic = WakuContentTopic::new("toychat", "2", "huilong", Encoding::Proto);

    let sleep = time::sleep(Duration::from_secs(ECHO_TIMEOUT));
    tokio::pin!(sleep);

    // Send and receive messages. Waits until all messages received.
    let got_all = tokio::select! {
        _ = sleep => false,
        _ = test_echo_messages(&node1, &node2, ECHO_MESSAGE, content_topic) => true,
    };

    assert!(got_all);

   
    node2.stop()?;
    node1.stop()?;
    
    Ok(())
}

#[test]
#[serial]
fn node_restart() {
    let config = WakuNodeConfig {
        node_key: Some(
            SecretKey::from_str("05f381866cc21f6c1e2e80e07fa732008e36d942dce3206ad6dcd6793c98d609")
                .unwrap(),
        ),
        ..Default::default()
    };

    for _ in 0..3 {
        let node = waku_new(config.clone().into()).expect("default config should be valid");

        node.start().expect("node should start with valid config");

        node.stop().expect("node should stop");
    }
}
