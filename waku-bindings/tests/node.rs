use multiaddr::Multiaddr;
use regex::Regex;
use secp256k1::SecretKey;
use serial_test::serial;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::{collections::HashSet, str::from_utf8};
use tokio::time;
use tokio::time::sleep;
use waku_bindings::node::PubsubTopic;
use waku_bindings::{
    waku_new, Encoding, Event, Initialized, MessageHash, WakuContentTopic, WakuMessage,
    WakuNodeConfig, WakuNodeHandle,
};
use waku_bindings::{LibwakuResponse, Running};
const ECHO_TIMEOUT: u64 = 1000;
const ECHO_MESSAGE: &str = "Hi from 🦀!";
const TEST_PUBSUBTOPIC: &str = "test";

fn try_publish_relay_messages(
    node: &WakuNodeHandle<Running>,
    msg: &WakuMessage,
) -> Result<HashSet<MessageHash>, String> {
    Ok(HashSet::from([node.relay_publish_message(
        msg,
        &PubsubTopic::new(TEST_PUBSUBTOPIC),
        None,
    )?]))
}

async fn test_echo_messages(
    node1: WakuNodeHandle<Initialized>,
    node2: WakuNodeHandle<Initialized>,
    content: &'static str,
    content_topic: WakuContentTopic,
) -> Result<(), String> {
    // setting a naïve event handler to avoid appearing ERR messages in logs
    node1
        .set_event_callback(&|_| {})
        .expect("set event call back working");

    let rx_waku_message: Arc<Mutex<WakuMessage>> = Arc::new(Mutex::new(WakuMessage::default()));

    let rx_waku_message_cloned = rx_waku_message.clone();
    let closure = move |response| {
        if let LibwakuResponse::Success(v) = response {
            let event: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

            match event {
                Event::WakuMessage(evt) => {
                    if let Ok(mut msg_lock) = rx_waku_message_cloned.lock() {
                        *msg_lock = evt.waku_message;
                    }
                }
                Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
                _ => panic!("event case not expected"),
            };
        }
    };

    println!("Before setting event callback");

    node2
        .set_event_callback(closure)
        .expect("set event call back working"); // Set the event callback with the closure

    let node1 = node1.start()?;
    let node2 = node2.start()?;

    node1
        .relay_subscribe(&PubsubTopic::new(TEST_PUBSUBTOPIC))
        .unwrap();
    node2
        .relay_subscribe(&PubsubTopic::new(TEST_PUBSUBTOPIC))
        .unwrap();

    sleep(Duration::from_secs(3)).await;

    // Interconnect nodes
    // Replace all matches with 127.0.0.1 to avoid issue with NAT or firewall.
    let addresses1 = node1.listen_addresses().unwrap();
    let addresses1 = &addresses1[0].to_string();

    let re = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
    let addresses1 = re.replace_all(addresses1, "127.0.0.1").to_string();

    let addresses1 = addresses1.parse::<Multiaddr>().expect("parse multiaddress");

    println!("Connecting node1 to node2: {}", addresses1);
    node2.connect(&addresses1, None).unwrap();

    // Wait for mesh to form
    sleep(Duration::from_secs(3)).await;

    dbg!("Before publish");
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
    let _ids = try_publish_relay_messages(&node1, &message).expect("send relay messages");

    // Wait for the msg to arrive
    let rx_waku_message_cloned = rx_waku_message.clone();
    for _ in 0..50 {
        if let Ok(msg) = rx_waku_message_cloned.lock() {
            // dbg!("The waku message value is: {:?}", msg);
            let payload = msg.payload.to_vec();
            let payload_str = from_utf8(&payload).expect("should be valid message");
            if payload_str == ECHO_MESSAGE {
                node1.stop()?;
                node2.stop()?;
                return Ok(());
            }
        } else {
            sleep(Duration::from_millis(100)).await;
        }
    }

    let node1 = node1.stop()?;
    let node2 = node2.stop()?;

    node1.waku_destroy()?;
    node2.waku_destroy()?;

    return Err("Unexpected test ending".to_string());
}

#[tokio::test]
#[serial]
async fn default_echo() -> Result<(), String> {
    println!("Test default_echo");
    let node1 = waku_new(Some(WakuNodeConfig {
        tcp_port: Some(60010),
        ..Default::default()
    }))?;
    let node2 = waku_new(Some(WakuNodeConfig {
        tcp_port: Some(60020),
        ..Default::default()
    }))?;

    let content_topic = WakuContentTopic::new("toychat", "2", "huilong", Encoding::Proto);

    let sleep = time::sleep(Duration::from_secs(ECHO_TIMEOUT));
    tokio::pin!(sleep);

    // Send and receive messages. Waits until all messages received.
    let got_all = tokio::select! {
        _ = sleep => false,
        _ = test_echo_messages(node1, node2, ECHO_MESSAGE, content_topic) => true,
    };

    assert!(got_all);

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
        let node = node.start().expect("node should start with valid config");
        let node = node.stop().expect("node should stop");
        node.waku_destroy().expect("free resources");
    }
}
