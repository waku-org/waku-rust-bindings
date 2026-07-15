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

    #[serde(rename = "connection_change")]
    ConnectionChange(ConnectionChangeEvent),

    #[serde(rename = "connection_status_change")]
    ConnectionStatusChange(ConnectionStatusChangeEvent),

    #[serde(rename = "message_sent")]
    MessageSent(MessageSentEvent),

    #[serde(rename = "message_error")]
    MessageError(MessageErrorEvent),

    #[serde(rename = "message_propagated")]
    MessagePropagated(MessagePropagatedEvent),

    #[serde(rename = "message_received")]
    MessageReceived(MessageReceivedEvent),

    #[serde(rename = "channel_message_received")]
    ChannelMessageReceived(ChannelMessageReceivedEvent),

    #[serde(rename = "channel_message_sent")]
    ChannelMessageSent(ChannelMessageSentEvent),

    #[serde(rename = "channel_message_error")]
    ChannelMessageError(ChannelMessageErrorEvent),

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

/// Type of `event` field for a `topic health` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TopicHealthEvent {
    /// The pubsub topic on which the message was received
    pub pubsub_topic: String,
    /// The message hash
    pub topic_health: String,
}

/// Type of `event` field for a `connection change` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionChangeEvent {
    /// The pubsub topic on which the message was received
    pub peer_id: String,
    /// The message hash
    pub peer_event: String,
}

/// Type of `event` field for a `message sent` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageSentEvent {
    /// The id returned by the originating send
    pub request_id: String,
    /// The message hash
    pub message_hash: String,
}

/// Type of `event` field for a `message error` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageErrorEvent {
    /// The id returned by the originating send
    pub request_id: String,
    /// The message hash
    pub message_hash: String,
    /// Why the send failed
    pub error: String,
}

/// Type of `event` field for a `message propagated` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagePropagatedEvent {
    /// The id returned by the originating send
    pub request_id: String,
    /// The message hash
    pub message_hash: String,
}

/// Type of `event` field for a `message received` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageReceivedEvent {
    /// The message hash
    pub message_hash: String,
    /// The message in [`WakuMessage`] format
    pub message: WakuMessage,
}

/// Type of `event` field for a `connection status change` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatusChangeEvent {
    /// Whether the node is online, and how it is connected
    pub connection_status: String,
}

/// Type of `event` field for a `channel message received` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessageReceivedEvent {
    /// The channel the message arrived on
    pub channel_id: String,
    /// The participant that sent the message
    pub sender_id: String,
    /// The message contents
    #[serde(with = "base64_payload")]
    pub payload: Vec<u8>,
}

/// Type of `event` field for a `channel message sent` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessageSentEvent {
    /// The channel the message was sent on
    pub channel_id: String,
    /// The id returned by the originating send
    pub request_id: String,
}

/// Type of `event` field for a `channel message error` event
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessageErrorEvent {
    /// The channel the message was sent on
    pub channel_id: String,
    /// The id returned by the originating send
    pub request_id: String,
    /// Why the send failed
    pub error: String,
}

mod base64_payload {
    use base64::Engine;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(payload: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        base64::engine::general_purpose::STANDARD
            .encode(payload)
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_string = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(as_string)
            .map_err(|e| D::Error::custom(format!("{e}")))
    }
}

#[cfg(test)]
mod tests {
    use crate::WakuEvent;
    use crate::WakuEvent::{ConnectionChange, RelayTopicHealthChange};

    #[test]
    fn deserialize_message_event() {
        let s = "{\"eventType\":\"message\",\"messageHash\":\"0xd40aa51bbb4867fe40329a255575cfc9ef4000358cc7321b2668b008cba94b30\",\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}";
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

    #[test]
    fn deserialize_channel_message_received_event() {
        let s = "{\"eventType\":\"channel_message_received\",\"channelId\":\"my-channel\",\"senderId\":\"alice\",\"payload\":\"SGkgZnJvbSDwn6aAIQ==\"}";
        let evt: WakuEvent = serde_json::from_str(s).unwrap();
        match evt {
            WakuEvent::ChannelMessageReceived(event) => {
                assert_eq!(event.channel_id, "my-channel");
                assert_eq!(event.sender_id, "alice");
                assert_eq!(event.payload, "Hi from 🦀!".as_bytes());
            }
            _ => panic!("Expected ChannelMessageReceived event, but got {:?}", evt),
        }
    }

    #[test]
    fn deserialize_channel_message_error_event() {
        let s = "{\"eventType\":\"channel_message_error\",\"channelId\":\"my-channel\",\"requestId\":\"req-1\",\"error\":\"boom\"}";
        let evt: WakuEvent = serde_json::from_str(s).unwrap();
        match evt {
            WakuEvent::ChannelMessageError(event) => {
                assert_eq!(event.channel_id, "my-channel");
                assert_eq!(event.request_id, "req-1");
                assert_eq!(event.error, "boom");
            }
            _ => panic!("Expected ChannelMessageError event, but got {:?}", evt),
        }
    }

    #[test]
    fn deserialize_connection_status_change_event() {
        let s = "{\"eventType\":\"connection_status_change\",\"connectionStatus\":\"Online\"}";
        let evt: WakuEvent = serde_json::from_str(s).unwrap();
        match evt {
            WakuEvent::ConnectionStatusChange(event) => {
                assert_eq!(event.connection_status, "Online");
            }
            _ => panic!("Expected ConnectionStatusChange event, but got {:?}", evt),
        }
    }

    #[test]
    fn deserialize_connection_change_event() {
        let s = "{\"eventType\":\"connection_change\", \"peerId\":\"16Uiu2HAmAR24Mbb6VuzoyUiGx42UenDkshENVDj4qnmmbabLvo31\",\"peerEvent\":\"Joined\"}";
        let evt: WakuEvent = serde_json::from_str(s).unwrap();
        match evt {
            ConnectionChange(conn_change_event) => {
                assert_eq!(
                    conn_change_event.peer_id,
                    "16Uiu2HAmAR24Mbb6VuzoyUiGx42UenDkshENVDj4qnmmbabLvo31"
                );
                assert_eq!(conn_change_event.peer_event, "Joined");
            }
            _ => panic!("Expected RelayTopicHealthChange event, but got {:?}", evt),
        }
    }
}
