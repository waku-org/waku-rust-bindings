//! # Waku
//!
//! Implementation on top of [`waku-bindings`](https://rfc.vac.dev/spec/36/)
mod decrypt;
mod events;
mod general;
mod node;

pub use node::{
    waku_create_content_topic, waku_create_pubsub_topic, waku_dafault_pubsub_topic, waku_new,
    waku_store_query, Initialized, Protocol, Running, WakuNodeConfig, WakuNodeHandle, WakuPeerData,
    WakuPeers,
};

pub use general::{
    ContentFilter, DecodedPayload, Encoding, FilterSubscription, MessageId, MessageIndex,
    PagingOptions, PeerId, ProtocolId, Result, StoreQuery, StoreResponse, WakuContentTopic,
    WakuMessage, WakuMessageVersion, WakuPubSubTopic,
};

pub use events::{waku_set_event_callback, Event, Signal, WakuMessageEvent};

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use waku_sys::waku_content_topic;

    #[test]
    fn content_topic() {
        let topic = unsafe {
            waku_content_topic(
                "foo_bar".as_ptr() as *mut c_char,
                1,
                "foo_topic".as_ptr() as *mut c_char,
                "rfc26".as_ptr() as *mut c_char,
            )
        };

        let topic_str = unsafe { CStr::from_ptr(topic) }
            .to_str()
            .expect("Decoded return");
        println!("{}", topic_str);
    }
}
