//! Waku message [event](https://rfc.vac.dev/spec/36/#events) related items
//!
//! Asynchronous events require a callback to be registered.
//! An example of an asynchronous event that might be emitted is receiving a message.
//! When an event is emitted, this callback will be triggered receiving an [`Event`]

// std
use std::ffi::c_void;
// crates
use serde::{Deserialize, Serialize};
// internal
use crate::general::WakuMessage;
use crate::utils::get_trampoline;
use crate::MessageId;
use waku_sys::WakuCallBack;

/// Waku event
/// For now just WakuMessage is supported
#[non_exhaustive]
#[derive(Serialize, Deserialize)]
#[serde(tag = "eventType", rename_all = "camelCase")]
pub enum Event {
    #[serde(rename = "message")]
    WakuMessage(WakuMessageEvent),
    Unrecognized(serde_json::Value),
}

/// Type of `event` field for a `message` event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessageEvent {
    /// The pubsub topic on which the message was received
    pubsub_topic: String,
    /// The message id
    message_id: MessageId,
    /// The message in [`WakuMessage`] format
    waku_message: WakuMessage,
}

impl WakuMessageEvent {
    pub fn pubsub_topic(&self) -> &String {
        &self.pubsub_topic
    }

    pub fn message_id(&self) -> &String {
        &self.message_id
    }

    pub fn waku_message(&self) -> &WakuMessage {
        &self.waku_message
    }
}

/// Wrapper callback, it transformst the `*const c_char` into an [`Event`]
fn callback<F: FnMut(Event) + Send + Sync>(mut f: F) -> WakuCallBack {
    let cb = move |v: &str| {
        let data: Event = serde_json::from_str(v).expect("Parsing event to succeed");
        println!("EXEC CALLBACK");
        f(data);
        println!("SUCCESS!");
    };

    get_trampoline(&cb)
}

/// Register callback to act as event handler and receive application events,
/// which are used to react to asynchronous events in Waku
pub fn waku_set_event_callback<F: FnMut(Event) + Send + Sync>(ctx: *mut c_void, f: F) {
    unsafe { waku_sys::waku_set_event_callback(ctx, callback(f), std::ptr::null_mut()) };
}

#[cfg(test)]
mod tests {
    use crate::node::events::callback;
    use crate::Event;

    // TODO: how to actually send an event and check if the callback is run?
    #[test]
    fn set_callback() {
        callback(|_event| {});
    }

    #[test]
    fn deserialize_message_event() {
        let s = "{\"eventType\":\"message\",\"messageId\":\"0x26ff3d7fbc950ea2158ce62fd76fd745eee0323c9eac23d0713843b0f04ea27c\",\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}";
        let evt: Event = serde_json::from_str(s).unwrap();
        assert!(matches!(evt, Event::WakuMessage(_)));
    }
}
