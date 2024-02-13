use aes_gcm::{Aes256Gcm, KeyInit};
use multiaddr::Multiaddr;
use rand::thread_rng;
use secp256k1::SecretKey;
use serial_test::serial;
use std::time::{Duration, SystemTime};
use std::{collections::HashSet, str::from_utf8};
use tokio::sync::mpsc::{self, Sender};
use tokio::time;
use waku_bindings::{
    waku_new, waku_set_event_callback, Encoding, Event, Key, MessageId, WakuContentTopic,
    WakuMessage, WakuNodeConfig, WakuNodeHandle, WakuPubSubTopic,
};

const ECHO_TIMEOUT: u64 = 10;
const ECHO_MESSAGE: &str = "Hi from 🦀!";

const NODES: &[&str] = &[
    "/dns4/node-01.ac-cn-hongkong-c.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAkvWiyFsgRhuJEb9JfjYxEkoHLgnUQmr1N5mKWnYjxYRVm",
    "/dns4/node-01.do-ams3.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmPLe7Mzm8TsYUubgCAW1aJoeFScxrLj8ppHFivPo97bUZ",
    "/dns4/node-01.gc-us-central1-a.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmJb2e28qLXxT5kZxVUUoJt72EMzNGXB47Rxx5hw3q4YjS"
];

fn try_publish_relay_messages(
    node: &WakuNodeHandle,
    msg: &WakuMessage,
) -> Result<HashSet<MessageId>, String> {
    Ok(HashSet::from(
        [node.relay_publish_message(msg, None, None)?],
    ))
}

#[derive(Debug)]
struct Response {
    id: MessageId,
    payload: Vec<u8>,
}

fn set_callback(tx: Sender<Response>, sk: SecretKey, ssk: Key<Aes256Gcm>) {
    waku_set_event_callback(move |signal| {
        if let Event::WakuMessage(message) = signal.event() {
            let id = message.message_id();
            let message = message.waku_message();

            let payload = message.payload().to_vec();

            futures::executor::block_on(tx.send(Response {
                id: id.to_string(),
                payload,
            }))
            .expect("send response to the receiver");
        }
    });
}

async fn test_echo_messages(
    node: &WakuNodeHandle,
    content: &'static str,
    content_topic: WakuContentTopic,
    sk: SecretKey,
    ssk: Key<Aes256Gcm>,
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

    let (tx, mut rx) = mpsc::channel(1);
    set_callback(tx, sk, ssk);

    let mut ids = try_publish_relay_messages(node, &message).expect("send relay messages");

    while let Some(res) = rx.recv().await {
        if ids.take(&res.id).is_some() {
            let msg = from_utf8(&res.payload).expect("should be valid message");
            assert_eq!(content, msg);
        }

        if ids.is_empty() {
            break;
        }
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn default_echo() -> Result<(), String> {
    let config = WakuNodeConfig {
        ..Default::default()
    };

    let node = waku_new(Some(config))?;

    node.start()?;

    for node_address in NODES {
        let address: Multiaddr = node_address.parse().unwrap();
        node.connect(&address, None)?;
    }
    let sk = SecretKey::new(&mut thread_rng());
    let ssk = Aes256Gcm::generate_key(&mut thread_rng());

    // subscribe to default channel
    node.relay_subscribe(&content_filter)?;

    let content_topic = WakuContentTopic::new("toychat", "2", "huilong", Encoding::Proto);

    let sleep = time::sleep(Duration::from_secs(ECHO_TIMEOUT));
    tokio::pin!(sleep);

    // Send and receive messages. Waits until all messages received.
    let got_all = tokio::select! {
        _ = sleep => false,
        _ = test_echo_messages(&node, ECHO_MESSAGE, content_topic, sk, ssk) => true,
    };

    assert!(got_all);

    node.stop()?;
    Ok(())
}

#[test]
#[serial]
fn node_restart() {
    let config = WakuNodeConfig::default();

    for _ in 0..3 {
        let node = waku_new(config.clone().into()).expect("default config should be valid");

        node.start().expect("node should start with valid config");

        node.stop().expect("node should stop");
    }
}
