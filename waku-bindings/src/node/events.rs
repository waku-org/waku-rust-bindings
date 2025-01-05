//! Waku message [event](https://rfc.vac.dev/spec/36/#events) related items
//!
//! Asynchronous events require a callback to be registered.
//! An example of an asynchronous event that might be emitted is receiving a message.
//! When an event is emitted, this callback will be triggered receiving an [`WakuEvent`]

// crates
use serde::{Deserialize, Serialize};
// internal
use crate::general::WakuMessage;
use std::str;

use crate::MessageHash;

/// Waku event
/// For now just WakuMessage is supported
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "eventType", rename_all = "camelCase")]
pub enum WakuEvent {
    #[serde(rename = "message")]
    WakuMessage(WakuMessageEvent),

    #[serde(rename = "relay_topic_health_change")]
    RelayTopicHealthChange(TopicHealthEvent),

    Unrecognized(serde_json::Value),
}

/// Type of `event` field for a `message` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessageEvent {
    /// The pubsub topic on which the message was received
    pub pubsub_topic: String,
    /// The message hash
    pub message_hash: MessageHash,
    /// The message in [`WakuMessage`] format
    pub waku_message: WakuMessage,
}

/// Type of `event` field for a `message` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TopicHealthEvent {
    /// The pubsub topic on which the message was received
    pub pubsub_topic: String,
    /// The message hash
    pub topic_health: String,
}

#[cfg(test)]
mod tests {
    use crate::WakuEvent;
    use crate::WakuEvent::RelayTopicHealthChange;

    #[test]
    fn deserialize_message_event() {
        let s = "{\"eventType\":\"message\",\"messageHash\":[91, 70, 26, 8, 141, 232, 150, 200, 26, 206, 224, 175, 249, 74, 61, 140, 231, 126, 224, 160, 91, 80, 162, 65, 250, 171, 84, 149, 133, 110, 214, 101],\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}";
        let evt: WakuEvent = serde_json::from_str(s).unwrap();
        assert!(matches!(evt, WakuEvent::WakuMessage(_)));
    }

    #[test]
    fn deserialize_topic_health_change_event() {
        let s = "{\"eventType\":\"relay_topic_health_change\", \"pubsubTopic\":\"/waku/2/rs/16/1\",\"topicHealth\":\"MinimallyHealthy\"}";
        let evt: WakuEvent = serde_json::from_str(s).unwrap();
        match evt {
            RelayTopicHealthChange(topic_health_event) => {
                assert_eq!(topic_health_event.pubsub_topic, "/waku/2/rs/16/1");
                assert_eq!(topic_health_event.topic_health, "MinimallyHealthy");
            }
            _ => panic!("Expected RelayTopicHealthChange event, but got {:?}", evt),
        }
    }
}
