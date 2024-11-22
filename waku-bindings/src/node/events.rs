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

pub struct WakuNodeContext {
    obj_ptr: *mut c_void,
    msg_observer: Arc<Mutex<Box<dyn FnMut(LibwakuResponse) + Send + Sync>>>,
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
    pub fn new(obj_ptr: *mut c_void) -> Self {
        Self {
            obj_ptr: obj_ptr,
            msg_observer: Arc::new(Mutex::new(Box::new(|_response| {
                println!("msg observer not set")
            }))),
        }
    }

    pub fn get_ptr(&self) -> *mut c_void {
        self.obj_ptr
    }

    /// Register callback to act as event handler and receive application events,
    /// which are used to react to asynchronous events in Waku
    pub fn waku_set_event_callback<F: FnMut(LibwakuResponse) + 'static + Sync + Send>(
        &self,
        closure: F,
    ) -> Result<(), String> {
        if let Ok(mut boxed_closure) = self.msg_observer.lock() {
            *boxed_closure = Box::new(closure);
            unsafe {
                let cb = get_trampoline(&(*boxed_closure));
                waku_sys::waku_set_event_callback(
                    self.obj_ptr,
                    cb,
                    &mut (*boxed_closure) as *mut _ as *mut c_void,
                )
            };
            Ok(())
        } else {
            Err(format!(
                "Failed to acquire lock in waku_set_event_callback!"
            ))
        }
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
