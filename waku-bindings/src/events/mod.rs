//! Waku message [event](https://rfc.vac.dev/spec/36/#events) related items
//!
//! Asynchronous events require a callback to be registered.
//! An example of an asynchronous event that might be emitted is receiving a message.
//! When an event is emitted, this callback will be triggered receiving a [`Signal`]

// std
use std::ffi::{c_char, c_int, c_void, CStr};
use std::ops::Deref;
use std::sync::Mutex;
// crates
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
// internal
use crate::general::{WakuMessage, WakuPubSubTopic};
use crate::MessageId;

/// Event signal
#[derive(Serialize, Deserialize)]
pub struct Signal {
    /// Type of signal being emitted. Currently, only message is available
    #[serde(alias = "type")]
    _type: String,
    /// Format depends on the type of signal
    event: Event,
}

impl Signal {
    pub fn event(&self) -> &Event {
        &self.event
    }
}

/// Waku event
/// For now just WakuMessage is supported
#[non_exhaustive]
#[derive(Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Event {
    WakuMessage(WakuMessageEvent),
    Unrecognized(serde_json::Value),
}

/// Type of `event` field for a `message` event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessageEvent {
    /// The pubsub topic on which the message was received
    pubsub_topic: WakuPubSubTopic,
    /// The message id
    message_id: MessageId,
    /// The message in [`WakuMessage`] format
    waku_message: WakuMessage,
}

impl WakuMessageEvent {
    pub fn pubsub_topic(&self) -> &WakuPubSubTopic {
        &self.pubsub_topic
    }

    pub fn message_id(&self) -> &String {
        &self.message_id
    }

    pub fn waku_message(&self) -> &WakuMessage {
        &self.waku_message
    }
}


/// Wrapper callback, it transformst the `*const c_char` into a [`Signal`]
/// and executes the [`CALLBACK`] funtion with it
extern "C" fn callback(_ret_code: c_int, data: *const c_char, _user_data: *mut c_void) {
    let raw_response = unsafe { CStr::from_ptr(data) }
        .to_str()
        .expect("Not null ptr");
    let data: Signal = serde_json::from_str(raw_response).expect("Parsing signal to succeed");
    (CALLBACK
        .deref()
        .lock()
        .expect("Access to the shared callback")
        .as_mut())(data)
}

/// Register callback to act as event handler and receive application signals,
/// which are used to react to asynchronous events in Waku
pub fn waku_set_event_callback<F: FnMut(Signal) + Send + Sync + 'static>(f: F) {
    // TODO: 
    unsafe { waku_sys::waku_set_event_callback(Some(callback)) };
}

#[cfg(test)]
mod tests {
    use crate::events::waku_set_event_callback;
    use crate::{Event, Signal};

    // TODO: how to actually send a signal and check if the callback is run?
    #[test]
    fn set_event_callback() {
        waku_set_event_callback(|_signal| {});
    }

    #[test]
    fn deserialize_signal() {
        let s = "{\"type\":\"message\",\"event\":{\"messageId\":\"0x26ff3d7fbc950ea2158ce62fd76fd745eee0323c9eac23d0713843b0f04ea27c\",\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}}";
        let _: Signal = serde_json::from_str(s).unwrap();
    }

    #[test]
    fn deserialize_event() {
        let e = "{\"messageId\":\"0x26ff3d7fbc950ea2158ce62fd76fd745eee0323c9eac23d0713843b0f04ea27c\",\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}";
        let _: Event = serde_json::from_str(e).unwrap();
    }
}
