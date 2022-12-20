use aes_gcm::{Aes256Gcm, KeyInit};
use multiaddr::Multiaddr;
use rand::thread_rng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use waku_bindings::{
    waku_new, waku_set_event_callback, Encoding, Event, ProtocolId, WakuContentTopic, WakuLogLevel,
    WakuMessage, WakuNodeConfig,
};

const NODES: &[&str] = &[
    "/dns4/node-01.ac-cn-hongkong-c.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAkvWiyFsgRhuJEb9JfjYxEkoHLgnUQmr1N5mKWnYjxYRVm",
    "/dns4/node-01.do-ams3.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmPLe7Mzm8TsYUubgCAW1aJoeFScxrLj8ppHFivPo97bUZ",
    "/dns4/node-01.gc-us-central1-a.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmJb2e28qLXxT5kZxVUUoJt72EMzNGXB47Rxx5hw3q4YjS"
];

#[ignore]
#[test]
pub fn main() -> Result<(), String> {
    let config = WakuNodeConfig {
        host: IpAddr::from_str("0.0.0.0").ok(),
        port: None,
        advertise_addr: None,
        node_key: None,
        keep_alive_interval: None,
        relay: None,
        min_peers_to_publish: None,
        filter: None,
        log_level: Some(WakuLogLevel::Error),
        discv5: Some(false),
        discv5_udp_port: Some(9000),
        discv5_bootstrap_nodes: Vec::new(),
    };
    let node = waku_new(Some(config))?;
    let node = node.start()?;
    println!("Node peer id: {}", node.peer_id()?);

    for node_address in NODES {
        let address: Multiaddr = node_address.parse().unwrap();
        let peer_id = node.add_peer(&address, ProtocolId::Relay)?;
        node.connect_peer_with_id(peer_id, None)?;
    }

    assert!(node.peers()?.len() >= NODES.len());
    assert!(node.peer_count()? >= NODES.len());

    assert!(node.relay_enough_peers(None)?);
    let sk = SecretKey::new(&mut thread_rng());
    let pk = PublicKey::from_secret_key(&Secp256k1::new(), &sk);
    let ssk = Aes256Gcm::generate_key(&mut thread_rng());

    let content = "Hi from 🦀!";
    let content_callback = content.clone();

    waku_set_event_callback(move |signal| match signal.event() {
        Event::WakuMessage(message) => {
            println!("Message with id [{}] received", message.message_id());
            let message = message.waku_message();
            let payload = if let Ok(message) = message
                .try_decode_asymmetric(&sk)
                .map_err(|e| println!("{e}"))
            {
                println!("Asymmetryc message");
                message.data().to_vec()
            } else if let Ok(message) = message
                .try_decode_symmetric(&ssk)
                .map_err(|e| println!("{e}"))
            {
                println!("Symmetryc message");
                message.data().to_vec()
            } else {
                println!("Unencoded message");
                message.payload().to_vec()
            };
            let message_content: String =
                String::from_utf8(payload).expect("Message should be able to be read");
            println!("Message content: {message_content}");
            assert_eq!(message_content, content_callback);
        }
        _ => {
            println!("Wtf is this event?");
        }
    });

    // subscribe to default channel
    node.relay_subscribe(None)?;
    let content_topic = WakuContentTopic::new("toychat", 2, "huilong", Encoding::Proto);

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
    );

    node.relay_publish_message(&message, None, None)?;
    node.relay_publish_encrypt_asymmetric(&message, None, &pk, None, None)?;
    node.relay_publish_encrypt_symmetric(&message, None, &ssk, None, None)?;
    node.relay_publish_encrypt_asymmetric(&message, None, &pk, Some(&sk), None)?;
    node.relay_publish_encrypt_symmetric(&message, None, &ssk, Some(&sk), None)?;

    let peer_id = node
        .peers()
        .unwrap()
        .iter()
        .map(|peer| peer.peer_id())
        .filter(|id| id.as_str() != node.peer_id().unwrap().as_str())
        .next()
        .unwrap()
        .clone();

    node.lightpush_publish(&message, None, peer_id.clone(), None)?;
    node.lightpush_publish_encrypt_asymmetric(&message, None, peer_id.clone(), &pk, None, None)?;
    node.lightpush_publish_encrypt_asymmetric(
        &message,
        None,
        peer_id.clone(),
        &pk,
        Some(&sk),
        None,
    )?;
    node.lightpush_publish_encrypt_symmetric(&message, None, peer_id.clone(), &ssk, None, None)?;
    node.lightpush_publish_encrypt_symmetric(
        &message,
        None,
        peer_id.clone(),
        &ssk,
        Some(&sk),
        None,
    )?;

    for node_data in node.peers()? {
        if node_data.peer_id() != &node.peer_id()? {
            node.disconnect_peer_with_id(node_data.peer_id())?;
        }
    }

    std::thread::sleep(Duration::from_secs(2));
    node.stop()?;
    Ok(())
}
