use multiaddr::Multiaddr;
use std::net::IpAddr;
use std::str::FromStr;
use waku::{waku_new, ProtocolId, WakuNodeConfig};

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
        node.connect_peer_with_id(peer_id, None)?;
    }

    node.stop()?;
    Ok(())
}
