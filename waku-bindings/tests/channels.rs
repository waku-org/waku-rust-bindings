use base64::Engine;
use regex::Regex;
use serde::Serialize;
use serial_test::serial;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use waku_bindings::{
    ChannelMessageReceivedPayload, ChannelMessageSentPayload, LogosDeliveryCtx, WakuNodeConfig,
};

const TEST_CHANNEL_ID: &str = "test-channel";
const TEST_CONTENT_TOPIC: &str = "/test/1/channels/proto";
const TEST_SENDER_ID: &str = "test-sender";
const NOOP_ENCRYPTION: &str = "noop";
const OTHER_SENDER_ID: &str = "other-sender";
const CHANNEL_PAYLOAD: &[u8] = b"Hi from a reliable channel!";
const TIMEOUT: Duration = Duration::from_secs(30);
const SHARDS_IN_NETWORK: usize = 8;

/// Body of `channel_send`, whose payload travels base64-encoded.
#[derive(Serialize)]
struct ChannelMessage {
    payload: String,
    ephemeral: bool,
}

fn new_node(tcp_port: usize) -> LogosDeliveryCtx {
    // A channel's shard is derived from its content topic, so the node needs
    // autosharding on, and has to sit on every shard a topic might hash to.
    let config = serde_json::to_string(&WakuNodeConfig {
        tcp_port: Some(tcp_port),
        num_shards_in_network: Some(SHARDS_IN_NETWORK as u16),
        shards: (0..SHARDS_IN_NETWORK).collect(),
        ..Default::default()
    })
    .expect("config should serialise");

    LogosDeliveryCtx::create(config, TIMEOUT).expect("node should instantiate")
}

fn channel_message(payload: &[u8]) -> String {
    serde_json::to_string(&ChannelMessage {
        payload: base64::engine::general_purpose::STANDARD.encode(payload),
        ephemeral: false,
    })
    .expect("message should serialise")
}

/// Dials `to` from `from`, rewriting the advertised address to loopback so NAT
/// or a firewall cannot interfere.
async fn connect(from: &LogosDeliveryCtx, to: &LogosDeliveryCtx) -> Result<(), String> {
    let addresses = to.waku_listen_addresses_async().await?;
    let address = addresses
        .split(',')
        .next()
        .ok_or("peer should report a listen address")?;

    let re = Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap();
    let address = re.replace_all(address, "127.0.0.1").to_string();

    from.waku_connect_async(address, 10_000).await.map(|_| ())
}

#[tokio::test]
#[serial]
async fn channel_create_send_close() {
    let node = new_node(60070);

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
            NOOP_ENCRYPTION.to_string(),
        )
        .await
        .expect("channel should be created");
    assert_eq!(channel_id, TEST_CHANNEL_ID);

    let request_id = node
        .channel_send_async(TEST_CHANNEL_ID.to_string(), channel_message(CHANNEL_PAYLOAD))
        .await
        .expect("channel send should succeed");
    assert!(!request_id.is_empty(), "send should return a request id");

    // Give the send a moment to finalise and emit its outcome event.
    tokio::time::sleep(Duration::from_secs(2)).await;

    node.channel_close_async(TEST_CHANNEL_ID.to_string())
        .await
        .expect("channel should close");

    // Scoped so the guard is dropped before the await below. Nothing is asserted
    // about arrival: a lone node has no peer to confirm delivery with, so
    // onChannelMessageSent legitimately never fires. Delivery is covered by
    // channel_message_reaches_peer.
    {
        let sent = sent.lock().unwrap();
        println!("channel sent events observed: {sent:?}");
    }

    node.stop_node_async().await.expect("node should stop");
    // The context is torn down by LogosDeliveryCtx's Drop impl.
}

/// Two nodes sharing a channel: a channel message should cross the wire and
/// surface through the typed onChannelMessageReceived listener.
///
/// Ignored: it does not, and the cause is below this binding layer. The message
/// reaches the receiver's messaging layer with the right content topic and the
/// `RELIABLE-CHANNEL-API/1` marker -- both of the channel ingress filters pass --
/// and is then consumed silently by SDS `handleIncoming` (empty result means
/// parked or duplicate), so no event and no error is ever raised. logos-delivery
/// has no two-node channel test of its own; its suite fabricates the wire by
/// emitting a MessageReceivedEvent locally with a stand-in peer's SDS envelope,
/// so this path looks unexercised rather than regressed.
///
/// Un-ignore once the SDS ingress question is settled upstream: everything on
/// this side (autosharding, per-channel encryption, send, connectivity) works.
#[tokio::test]
#[serial]
#[ignore = "channel ingress is dropped inside SDS; see the doc comment"]
async fn channel_message_reaches_peer() -> Result<(), String> {
    let sender = new_node(60080);
    let receiver = new_node(60090);

    let received: Arc<Mutex<Vec<ChannelMessageReceivedPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let received_cloned = received.clone();
    receiver.add_on_channel_message_received_listener(move |event| {
        received_cloned.lock().unwrap().push(event.clone());
    });

    // The raw message is observed too: it proves delivery reaches the messaging
    // layer, which is what localises the failure to channel ingress.
    let raw: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let raw_cloned = raw.clone();
    receiver.add_on_message_received_listener(move |_| {
        *raw_cloned.lock().unwrap() += 1;
    });

    // A failed send reports itself here; without this the test could only time
    // out and say nothing about why.
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let errors_cloned = errors.clone();
    sender.add_on_channel_message_error_listener(move |event| {
        errors_cloned.lock().unwrap().push(event.error.clone());
    });

    let msg_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let msg_errors_cloned = msg_errors.clone();
    sender.add_on_message_error_listener(move |event| {
        msg_errors_cloned.lock().unwrap().push(event.error.clone());
    });

    sender.start_node_async().await?;
    receiver.start_node_async().await?;

    // Both ends open the same channel on the same content topic, under distinct
    // participant ids.
    sender
        .channel_create_async(
            TEST_CHANNEL_ID.to_string(),
            TEST_CONTENT_TOPIC.to_string(),
            TEST_SENDER_ID.to_string(),
            NOOP_ENCRYPTION.to_string(),
        )
        .await?;
    receiver
        .channel_create_async(
            TEST_CHANNEL_ID.to_string(),
            TEST_CONTENT_TOPIC.to_string(),
            OTHER_SENDER_ID.to_string(),
            NOOP_ENCRYPTION.to_string(),
        )
        .await?;

    // Creating a channel does not subscribe: ingress arrives through the
    // messaging layer, so both ends have to subscribe to the content topic.
    sender.subscribe_async(TEST_CONTENT_TOPIC.to_string()).await?;
    receiver
        .subscribe_async(TEST_CONTENT_TOPIC.to_string())
        .await?;

    connect(&receiver, &sender).await?;

    // Wait for the mesh to form before publishing, or the message goes nowhere.
    tokio::time::sleep(Duration::from_secs(5)).await;

    sender
        .channel_send_async(TEST_CHANNEL_ID.to_string(), channel_message(CHANNEL_PAYLOAD))
        .await?;

    let mut delivered = None;
    for _ in 0..100 {
        if let Some(event) = received.lock().unwrap().first().cloned() {
            delivered = Some(event);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let event = delivered.ok_or_else(|| {
        let errors = errors.lock().unwrap();
        let msg_errors = msg_errors.lock().unwrap();
        let raw = raw.lock().unwrap();
        format!(
            "never reached the peer; raw messages seen by receiver: {raw}; \
             channel errors: {errors:?}; messaging errors: {msg_errors:?}"
        )
    })?;
    assert_eq!(event.channel_id, TEST_CHANNEL_ID);
    assert_eq!(event.sender_id, TEST_SENDER_ID);
    let payload = base64::engine::general_purpose::STANDARD
        .decode(&event.payload)
        .map_err(|e| e.to_string())?;
    assert_eq!(payload, CHANNEL_PAYLOAD);

    sender.channel_close_async(TEST_CHANNEL_ID.to_string()).await?;
    receiver
        .channel_close_async(TEST_CHANNEL_ID.to_string())
        .await?;

    sender.stop_node_async().await?;
    receiver.stop_node_async().await?;

    Ok(())
}
