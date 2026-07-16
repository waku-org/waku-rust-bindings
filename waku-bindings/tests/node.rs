use base64::Engine;
use multiaddr::Multiaddr;
use regex::Regex;
use secp256k1::SecretKey;
use serial_test::serial;
use std::str::FromStr;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;
use tokio::time::sleep;
use waku_bindings::{
    Encoding, LogosDeliveryCtx, WakuContentTopic, WakuMessage, WakuNodeConfig,
};

const ECHO_TIMEOUT: u64 = 1000;
const ECHO_MESSAGE: &str = "Hi from 🦀!";
const TEST_PUBSUBTOPIC: &str = "test";
const TIMEOUT: Duration = Duration::from_secs(30);

/// Cargo runs test binaries in parallel and serial_test only serialises within
/// one, so each binary needs its own persistency root. Every node in a binary
/// must share it: the singleton refuses to be re-targeted.
const STORAGE_PATH: &str = "./data-node-test";

fn new_node(config: WakuNodeConfig) -> Result<LogosDeliveryCtx, String> {
    let config = WakuNodeConfig {
        local_storage_path: Some(STORAGE_PATH.to_string()),
        ..config
    };
    let config_json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    LogosDeliveryCtx::create(config_json, TIMEOUT)
}

async fn test_echo_messages(
    node1: LogosDeliveryCtx,
    node2: LogosDeliveryCtx,
    content: &'static str,
    content_topic: WakuContentTopic,
) -> Result<(), String> {
    // The relay push arrives as a typed payload, so the test only has to decode
    // the base64 body rather than match on an event enum.
    let rx_payload: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    let rx_payload_cloned = rx_payload.clone();
    node2.add_on_received_message_listener(move |event| {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&event.waku_message.payload)
            .expect("event payload should be base64");
        if let Ok(mut payload) = rx_payload_cloned.lock() {
            *payload = decoded;
        }
    });

    node1.start_node_async().await?;
    node2.start_node_async().await?;

    node1
        .waku_relay_subscribe_async(TEST_PUBSUBTOPIC.to_string())
        .await?;
    node2
        .waku_relay_subscribe_async(TEST_PUBSUBTOPIC.to_string())
        .await?;

    sleep(Duration::from_secs(5)).await;

    // Interconnect nodes
    // Replace all matches with 127.0.0.1 to avoid issue with NAT or firewall.
    let addresses1 = node1.waku_listen_addresses_async().await?;
    let addresses1 = addresses1
        .split(',')
        .next()
        .ok_or("node1 should report a listen address")?;

    let re = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
    let addresses1 = re.replace_all(addresses1, "127.0.0.1").to_string();

    // Parsed only to prove the address is well formed before handing it over.
    let addresses1 = addresses1
        .parse::<Multiaddr>()
        .expect("parse multiaddress")
        .to_string();

    println!("Connecting node1 to node2: {addresses1}");
    node2.waku_connect_async(addresses1, 10_000).await?;

    // Wait for mesh to form
    sleep(Duration::from_secs(3)).await;

    dbg!("Before publish");
    let message = WakuMessage::new(content, content_topic, 1, Vec::new(), false);
    let message = serde_json::to_string(&message).map_err(|e| e.to_string())?;
    node1
        .waku_relay_publish_async(TEST_PUBSUBTOPIC.to_string(), message, 10_000)
        .await
        .expect("send relay messages");

    // Wait for the msg to arrive
    for _ in 0..50 {
        let message_received = if let Ok(payload) = rx_payload.lock() {
            !payload.is_empty()
                && from_utf8(&payload).expect("should be valid message") == ECHO_MESSAGE
        } else {
            false
        };
        if message_received {
            node1.stop_node_async().await?;
            node2.stop_node_async().await?;
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    node1.stop_node_async().await?;
    node2.stop_node_async().await?;

    Err("Unexpected test ending".to_string())
}

#[tokio::test]
#[serial]
async fn default_echo() -> Result<(), String> {
    println!("Test default_echo");
    let node1 = new_node(WakuNodeConfig {
        tcp_port: Some(60010),
        ..Default::default()
    })?;
    let node2 = new_node(WakuNodeConfig {
        tcp_port: Some(60020),
        ..Default::default()
    })?;

    let content_topic = WakuContentTopic::new("toychat", "2", "huilong", Encoding::Proto);

    let sleep = time::sleep(Duration::from_secs(ECHO_TIMEOUT));
    tokio::pin!(sleep);

    // Send and receive messages. Waits until all messages received.
    // The echo's Result decides the outcome: completing is not the same as
    // having received the message.
    let got_all = tokio::select! {
        _ = sleep => Err("timed out waiting for the echo".to_string()),
        result = test_echo_messages(node1, node2, ECHO_MESSAGE, content_topic) => result,
    };

    got_all?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn node_restart() {
    let config = WakuNodeConfig {
        node_key: Some(
            SecretKey::from_str("05f381866cc21f6c1e2e80e07fa732008e36d942dce3206ad6dcd6793c98d609")
                .unwrap(),
        ),
        ..Default::default()
    };

    for _ in 0..3 {
        let node = new_node(config.clone()).expect("default config should be valid");
        node.start_node_async()
            .await
            .expect("node should start with valid config");
        node.stop_node_async().await.expect("node should stop");
        // Resources are freed by LogosDeliveryCtx's Drop impl.
    }
}
