use secp256k1::SecretKey;
use serial_test::serial;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{collections::HashSet, str::from_utf8};
use std::cell::OnceCell;
use tokio::sync::broadcast::{self, Sender};
use waku_bindings::LibwakuResponse;
use std::sync::{Arc, OnceLock, Mutex}; // Import Arc and Mutex
use tokio::time;
use tokio::time::sleep;
use waku_bindings::utils;
use waku_bindings::{
    waku_destroy, waku_new, Encoding, Event, MessageHash, Running, WakuContentTopic, WakuMessage,
    WakuNodeConfig, WakuNodeHandle,
};
use std::ffi::c_void;
const ECHO_TIMEOUT: u64 = 1000;
const ECHO_MESSAGE: &str = "Hi from 🦀!";
const TEST_PUBSUBTOPIC: &str = "test";

fn try_publish_relay_messages(
    node: &WakuNodeHandle<Running>,
    msg: &WakuMessage,
) -> Result<HashSet<MessageHash>, String> {
    let topic = TEST_PUBSUBTOPIC.to_string();
    Ok(HashSet::from([
        node.relay_publish_message(msg, &topic, None)?
    ]))
}

#[derive(Debug, Clone)]
struct Response {
    hash: MessageHash,
    payload: Vec<u8>,
}

async fn test_echo_messages(
    node1: &WakuNodeHandle<Running>,
    node2: &WakuNodeHandle<Running>,
    content: &'static str,
    content_topic: WakuContentTopic,
) {
    // setting a naïve event handler to avoid appearing ERR messages in logs
    node1.set_event_callback(|_LibwakuResponse| {});

    let rx_waku_message: OnceCell<WakuMessage> = OnceCell::new();

    let closure = |response: LibwakuResponse| {
        if let LibwakuResponse::Success(v) = response {
            let event: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

            match event {
                Event::WakuMessage(evt) => {
                    println!("WakuMessage event received: {:?}", evt.waku_message);
                    // rx_waku_message = evt.waku_message; // Use the shared reference
                    let _ = rx_waku_message.set(evt.waku_message);
                }
                Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
                _ => panic!("event case not expected"),
            };
        }
    };

    println!("Before setting event callback");

    unsafe {
        let cb = utils::get_trampoline(&closure);
        waku_sys::waku_set_event_callback(node2.ctx.obj_ptr, cb, &closure as *const _ as *mut c_void)
    };
    
    // node2.set_event_callback(closure); // Set the event callback with the closure

    let topic = TEST_PUBSUBTOPIC.to_string();
    node1.relay_subscribe(&topic).unwrap();
    node2.relay_subscribe(&topic).unwrap();

    sleep(Duration::from_secs(3)).await;

    // Interconnect nodes
    println!("Connecting node1 to node2");
    let addresses1 = node1.listen_addresses().unwrap();
    node2.connect(&addresses1[0], None).unwrap();

    // Wait for mesh to form
    sleep(Duration::from_secs(3)).await;

    println!("Before publish");
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
    let ids = try_publish_relay_messages(node1, &message).expect("send relay messages");
    println!("After publish");

    // Wait for the msg to arrive
    for _ in 0..50 {
        if let Some(value) = rx_waku_message.get() {
            println!("The waku message value is: {:?}", value);
            break;
        } else {
            sleep(Duration::from_millis(100)).await;
        }
    }

    if let None = rx_waku_message.get() {
        println!("ERROR could not get waku message");
    }
}

#[tokio::test]
#[serial]
async fn default_echo() -> Result<(), String> {
    println!("Test default_echo");
    let node1 = waku_new(Some(WakuNodeConfig {
        port: Some(60010),
        ..Default::default()
    }))?;
    let node2 = waku_new(Some(WakuNodeConfig {
        port: Some(60020),
        ..Default::default()
    }))?;

    let node1 = node1.start()?;
    let node2 = node2.start()?;

    let waku_version = node2.version()?;
    let content_topic = WakuContentTopic::new("toychat", "2", "huilong", Encoding::Proto);

    let sleep = time::sleep(Duration::from_secs(ECHO_TIMEOUT));
    tokio::pin!(sleep);

    // Send and receive messages. Waits until all messages received.
    let got_all = tokio::select! {
        _ = sleep => false,
        _ = test_echo_messages(&node1, &node2, ECHO_MESSAGE, content_topic) => true,
    };

    assert!(got_all);

    let node2 = node2.stop()?;
    let node1 = node1.stop()?;

    let sleep = time::sleep(Duration::from_secs(5));
    tokio::pin!(sleep);

    waku_destroy(node1)?;
    waku_destroy(node2)?;

    Ok(())
}

#[test]
#[serial]
fn node_restart() {
    println!("Test node_restart");
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

        node.stop().expect("node should stop");
    }
}
