use multiaddr::Multiaddr;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::SystemTime;
use waku::{
    waku_new, waku_set_event_callback, Encoding, Event, ProtocolId, WakuContentTopic, WakuMessage,
    WakuNodeConfig,
};

const NODES: &[&str] = &[
    "/dns4/node-01.ac-cn-hongkong-c.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAkvWiyFsgRhuJEb9JfjYxEkoHLgnUQmr1N5mKWnYjxYRVm",
    "/dns4/node-01.do-ams3.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmPLe7Mzm8TsYUubgCAW1aJoeFScxrLj8ppHFivPo97bUZ",
    "/dns4/node-01.gc-us-central1-a.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmJb2e28qLXxT5kZxVUUoJt72EMzNGXB47Rxx5hw3q4YjS"
];

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
    };
    let node = waku_new(Some(config))?;
    let node = node.start()?;
    println!("Node peer id: {}", node.peer_id()?);

    for node_address in NODES {
        let address: Multiaddr = node_address.parse().unwrap();
        let peer_id = node.add_peer(&address, ProtocolId::Relay)?;
        // TODO: use conenct_peeri_with_id when [329](https://github.com/status-im/go-waku/pull/329) is fixed
        // node.connect_peer_with_id(peer_id, None)?;
        node.connect_peer_with_address(&address, None)?;
    }

    assert!(node.peers()?.len() >= NODES.len());
    assert!(node.peer_count()? >= NODES.len());

    assert!(node.relay_enough_peers(None)?);

    waku_set_event_callback(|signal| match signal.event() {
        Event::WakuMessage(message) => {
            println!("Message with id [{}] received", message.message_id());
        }
        _ => {
            println!("Wtf is this event?");
        }
    });

    // subscribe to default channel
    node.relay_subscribe(None)?;
    let content_topic = WakuContentTopic {
        application_name: "toychat".to_string(),
        version: 2,
        content_topic_name: "huilong".to_string(),
        encoding: Encoding::Proto,
    };

    let message = WakuMessage::new(
        "Hi from 🦀!",
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

    node.stop()?;
    Ok(())
}
