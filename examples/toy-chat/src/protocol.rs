use chrono::{DateTime, LocalResult, TimeZone, Utc};
use once_cell::sync::Lazy;
use prost::Message;
use waku_bindings::{Encoding, WakuContentTopic};

pub static TOY_CHAT_CONTENT_TOPIC: Lazy<WakuContentTopic> = Lazy::new(|| WakuContentTopic {
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
    pub fn new(nick: &str, payload: &str) -> Self {
        Self {
            timestamp: Utc::now().timestamp() as u64,
            nick: nick.to_string(),
            payload: payload.as_bytes().to_vec(),
        }
    }
    pub fn message(&self) -> String {
        String::from_utf8(self.payload.clone()).unwrap()
    }

    pub fn nick(&self) -> &str {
        &self.nick
    }

    pub fn timestamp(&self) -> LocalResult<DateTime<Utc>> {
        Utc.timestamp_opt(self.timestamp as i64, 0)
    }
}
