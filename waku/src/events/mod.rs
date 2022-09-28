// std
use std::ffi::{c_char, CStr};
// crates
use serde::{Deserialize, Serialize};
// internal
use crate::general::{PubsubTopic, WakuMessage};

#[derive(Serialize, Deserialize)]
pub struct Signal {
    #[serde(alias = "type")]
    _type: String,
    event: Event,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "untagged", rename_all = "camelCase")]
pub enum Event {
    WakuMessage(WakuMessageEvent),
}

/// Type of `event` field for a `message` event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessageEvent {
    /// The pubsub topic on which the message was received
    pubsub_topic: PubsubTopic,
    /// The message id
    message_id: String,
    /// The message in [`WakuMessage`] format
    waku_message: WakuMessage,
}

pub fn waku_set_event_callback<F: FnMut(Signal)>(mut callback: F) {
    let mut callback = move |data: *const c_char| {
        let raw_response = unsafe { CStr::from_ptr(data) }
            .to_str()
            .expect("Not null ptr");
        let data: Signal = serde_json::from_str(raw_response).expect("Parsing signal to succeed");
        callback(data);
    };
    unsafe { waku_sys::waku_set_event_callback(&mut callback as *mut _ as *mut std::ffi::c_void) };
}
