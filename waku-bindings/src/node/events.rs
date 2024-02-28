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
use crate::node::context::WakuNodeContext;
use crate::utils::{get_trampoline, LibwakuResponse};
use crate::MessageId;

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
    pub pubsub_topic: String,
    /// The message id
    pub message_id: MessageId,
    /// The message in [`WakuMessage`] format
    pub waku_message: WakuMessage,
}

/// Register callback to act as event handler and receive application events,
/// which are used to react to asynchronous events in Waku
pub fn waku_set_event_callback<F: FnMut(Event) + Send + Sync>(ctx: &WakuNodeContext, mut f: F) {
    let cb = |response: LibwakuResponse| {
        if let LibwakuResponse::Success(v) = response {
            let data: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");
            f(data);
        };
    };

    unsafe {
        let mut closure = cb;
        let cb = get_trampoline(&closure);

        waku_sys::waku_set_event_callback(ctx.obj_ptr, cb, &mut closure as *mut _ as *mut c_void)
    };
}

#[cfg(test)]
mod tests {
    use crate::Event;

    #[test]
    fn deserialize_message_event() {
        let s = "{\"eventType\":\"message\",\"messageId\":\"0x26ff3d7fbc950ea2158ce62fd76fd745eee0323c9eac23d0713843b0f04ea27c\",\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}";
        let evt: Event = serde_json::from_str(s).unwrap();
        assert!(matches!(evt, Event::WakuMessage(_)));
    }
}
