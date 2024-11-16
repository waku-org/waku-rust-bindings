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
use std::str;

use crate::utils::{get_trampoline, LibwakuResponse};
use crate::MessageHash;

use std::sync::{Arc, Mutex};

use crate::node::Observer;

pub struct WakuNodeContext {
    pub obj_ptr: *mut c_void,
    msg_observers: Arc<Mutex<Vec<Arc<dyn Observer + Send + Sync>>>>, // List of observers
}

/// Waku event
/// For now just WakuMessage is supported
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "eventType", rename_all = "camelCase")]
pub enum Event {
    #[serde(rename = "message")]
    WakuMessage(WakuMessageEvent),
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

impl WakuNodeContext {
    pub fn new(
        obj_ptr: *mut c_void
    ) -> Self {
        Self {
            obj_ptr: obj_ptr,
            msg_observers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_msg_observer(&mut self, observer: Arc<dyn Observer + Send + Sync>) {
        let mut observers = self.msg_observers.lock().expect("Failed to lock observers");
        observers.push(observer);
    }

    pub fn notify_observers(&self, msg: &WakuMessage) {
        let observers = self.msg_observers.lock().expect("Failed to lock observers");
        for observer in observers.iter() {
            observer.on_message_received(msg);
        }
    }

    fn event_callback(response: LibwakuResponse) {
        if let LibwakuResponse::Success(v) = response {
            let event: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

            // let mut game_state = self.game_state.lock().unwrap();
            match event {
                Event::WakuMessage(evt) => {
                    println!("WakuMessage event received: {:?}", evt.waku_message);
                },
                Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
            };
        }
    }

    /// Register callback to act as event handler and receive application events,
    /// which are used to react to asynchronous events in Waku
    pub fn waku_set_event_callback(&self) {
        let mut closure = WakuNodeContext::event_callback;
        unsafe {
            let cb = get_trampoline(&closure);
            waku_sys::waku_set_event_callback(self.obj_ptr, cb, &mut closure as *mut _ as *mut c_void)
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::Event;

    #[test]
    fn deserialize_message_event() {
        let s = "{\"eventType\":\"message\",\"messageHash\":\"0x26ff3d7fbc950ea2158ce62fd76fd745eee0323c9eac23d0713843b0f04ea27c\",\"pubsubTopic\":\"/waku/2/default-waku/proto\",\"wakuMessage\":{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}}";
        let evt: Event = serde_json::from_str(s).unwrap();
        assert!(matches!(evt, Event::WakuMessage(_)));
    }
}
