use chrono::Utc;
use once_cell::sync::Lazy;
use prost::Message;

use std::{error::Error, str::FromStr, thread::sleep, time::Duration};
use waku::{
    waku_new, waku_set_event_callback, Encoding, Multiaddr, ProtocolId, Running, Signal,
    WakuContentTopic, WakuMessage, WakuNodeHandle, WakuPubSubTopic,
};

#[derive(Clone, Message)]
pub struct BasicMessage {
    #[prost(string, tag = "1")]
    payload: String,
}

impl BasicMessage {
    fn new(payload: String) -> Self {
        BasicMessage { payload }
    }
}

fn setup_node_handle(graphcast_topic: Option<WakuPubSubTopic>) -> std::result::Result<WakuNodeHandle<Running>, Box<dyn Error>> {
    const NODES: &[&str] = &[
    "/dns4/node-01.ac-cn-hongkong-c.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAkvWiyFsgRhuJEb9JfjYxEkoHLgnUQmr1N5mKWnYjxYRVm",
    "/dns4/node-01.do-ams3.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmPLe7Mzm8TsYUubgCAW1aJoeFScxrLj8ppHFivPo97bUZ",
    "/dns4/node-01.gc-us-central1-a.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmJb2e28qLXxT5kZxVUUoJt72EMzNGXB47Rxx5hw3q4YjS"
];

    let node_handle = waku_new(None)?;
    let node_handle = node_handle.start()?;
    for address in NODES
        .iter()
        .map(|a| Multiaddr::from_str(a).expect("Could not parse address"))
    {
        let peerid = node_handle.add_peer(&address, ProtocolId::Relay)?;
        node_handle.connect_peer_with_id(peerid, None)?;
    }
    node_handle.relay_subscribe(graphcast_topic)?;
    Ok(node_handle)
}

pub static BASIC_TOPIC: Lazy<WakuContentTopic> = Lazy::new(|| WakuContentTopic {
    application_name: String::from("poi-crosschecker"),
    version: 0,
    content_topic_name: String::from("blaaaah"),
    encoding: Encoding::Proto,
});

fn main() {
    let app_name:String = String::from("graphcast");
    let graphcast_topic = Some(WakuPubSubTopic {
        topic_name: app_name,
        encoding: Encoding::Proto,
    });

    let node_handle = setup_node_handle(graphcast_topic.clone()).unwrap();

    waku_set_event_callback(move |signal: Signal| match signal.event() {
        waku::Event::WakuMessage(event) => {
            match <BasicMessage as Message>::decode(event.waku_message().payload()) {
                Ok(basic_message) => {
                    println!("New message received! \n{:?}", basic_message);
                }
                Err(e) => {
                    println!("Error occurred!\n {:?}", e);
                }
            }
        }
        waku::Event::Unrecognized(data) => {
            println!("Unrecognized event!\n {:?}", data);
        }
        _ => {
            println!("signal! {:?}", serde_json::to_string(&signal));
        }
    });

    let payload = String::from("This is the message payload");
    let message = BasicMessage::new(payload);
    let mut buff = Vec::new();
    Message::encode(&message, &mut buff).expect("Could not encode :(");

    let waku_message = WakuMessage::new(
        buff,
        BASIC_TOPIC.clone(),
        2,
        Utc::now().timestamp() as usize,
    );

    let res = node_handle
        .relay_publish_message(&waku_message, graphcast_topic, None)
        .expect("Could not send message.");

    println!("{:?}", res);

    loop {
        sleep(Duration::new(1, 0));
    }
}
