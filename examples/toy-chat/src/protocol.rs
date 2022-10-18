use once_cell::sync::{Lazy, OnceCell};
use prost::{
    encoding::{bytes, string, uint64},
    Message,
};
use waku::{Encoding, WakuContentTopic, WakuMessage};

const TOY_CHAT_CONTENT_TOPIC: Lazy<WakuContentTopic> = Lazy::new(|| WakuContentTopic {
    application_name: "toy-chat".into(),
    version: 2,
    content_topic_name: "huilong".into(),
    encoding: Encoding::Proto,
});

#[derive(Clone, Message)]
pub struct Chat2Message {
    #[prost(uint64, tag = "1")]
    timestamp: u64,
    #[prost(string, tag = "2")]
    nick: String,
    #[prost(bytes, tag = "3")]
    payload: Vec<u8>,
}

impl Chat2Message {
    pub fn message(&self) -> String {
        String::from_utf8(self.payload.clone()).unwrap()
    }
}
