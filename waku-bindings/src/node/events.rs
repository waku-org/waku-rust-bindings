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
use std::{slice, str};

use crate::utils::LibwakuResponse;
use crate::MessageHash;
use std::ops::Deref;
use std::sync::Mutex;
// crates
use once_cell::sync::Lazy;

pub struct WakuNodeContext {
    pub obj_ptr: *mut c_void,
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

#[allow(clippy::type_complexity)]
static CALLBACK: Lazy<Mutex<Box<dyn FnMut(LibwakuResponse) + Send + Sync>>> =
    Lazy::new(|| Mutex::new(Box::new(|_| {})));

/// Register global callback
fn set_callback<F: FnMut(LibwakuResponse) + Send + Sync + 'static>(f: F) {
    *CALLBACK.lock().unwrap() = Box::new(f);
}

unsafe extern "C" fn callback(
    ret_code: ::std::os::raw::c_int,
    data: *const ::std::os::raw::c_char,
    data_len: usize,
    user_data: *mut ::std::os::raw::c_void,
) {
    let response = if data.is_null() {
        ""
    } else {
        str::from_utf8(slice::from_raw_parts(data as *mut u8, data_len))
            .expect("could not retrieve response")
    };

    let result = LibwakuResponse::try_from((ret_code as u32, response))
        .expect("invalid response obtained from libwaku");

    (CALLBACK
        .deref()
        .lock()
        .expect("Access to the shared callback")
        .as_mut())(result);
}

impl WakuNodeContext {

    fn waku_event_callback(response: LibwakuResponse) {
        if let LibwakuResponse::Success(v) = response {
            let event: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

            // let mut game_state = self.game_state.lock().unwrap();
            match event {
                Event::WakuMessage(evt) => {
                    // println!("WakuMessage event received: {:?}", evt.waku_message);
                    let message = evt.waku_message;
                    let payload = message.payload.to_vec().clone();
                    match from_utf8(&payload) {
                        Ok(msg) => {
                            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                            println!("Message Received: {}", msg);
                            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");

                            // Send the message to the main thread
                            if let Ok(mut tx) = tx_clone.try_lock() {
                                //  Lock succeeded, proceed to send the message
                                if tx.blocking_send(msg.to_string()).is_err() {
                                    eprintln!("Failed to send message to async task");
                                } else {
                                    eprintln!("Sent!!!!");
                                }
                            } else {
                                eprintln!("Failed to acquire lock on tx_clone");
                            }

                            // Deserialize the JSON into the GameState struct
                            // Lock the game_state and update it
                            // match serde_json::from_str::<GameState>(msg) {
                            //     Ok(parsed_value) => {
                            //         // Handle the parsed value here
                            //         // self.game_state = parsed_value;
                            //         println!("Parsed correctly");
                            //     }
                            //     Err(e) => {
                            //         eprintln!("Failed to parse JSON: {}", e);
                            //         // Handle the error as needed, such as retrying, defaulting, etc.
                            //     }
                            // }
                            // *game_state = serde_json::from_str(msg).expect("Failed to deserialize JSON");

                            // let tx_inner = tx_cloned.clone();
                            // let msg_inner = msg.to_string();
                            // tokio::spawn(async move {
                            //     println!("do nothing");
                            // if tx_inner.send(msg_inner.to_string()).await.is_err() {
                            //     eprintln!("Failed to send message");
                            // }
                            // });
                        }
                        Err(e) => {
                            eprintln!("Failed to decode payload as UTF-8: {}", e);
                            // Handle the error as needed, or just log and skip
                        }
                    }
                }
                Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
                _ => panic!("event case not expected"),
            };
        }
    }

    /// Register callback to act as event handler and receive application events,
    /// which are used to react to asynchronous events in Waku
    pub fn waku_set_event_callback<F: FnMut(LibwakuResponse) + 'static + Sync + Send>(
        &self,
        mut closure: F,
    ) {
        set_callback(closure);
        unsafe {
            waku_sys::waku_set_event_callback(
                self.obj_ptr,
                Some(callback),
                callback as *mut c_void,
            );
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
