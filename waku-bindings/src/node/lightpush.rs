//! Waku [lightpush](https://rfc.vac.dev/spec/36/#waku-lightpush) protocol related methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
// internal
use crate::general::{MessageId, PeerId, Result, WakuMessage, WakuPubSubTopic};
use crate::node::waku_default_pubsub_topic;
use crate::utils::{get_trampoline, handle_response};

/// Publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publishchar-messagejson-char-topic-char-peerid-int-timeoutms)
pub fn waku_lightpush_publish(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    peer_id: PeerId,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_default_pubsub_topic)
        .to_string();
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();
    let peer_id_ptr = CString::new(peer_id)
        .expect("CString should build properly from peer id")
        .into_raw();

    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_lightpush_publish(
            message_ptr,
            topic_ptr,
            peer_id_ptr,
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(topic_ptr));
        drop(CString::from_raw(peer_id_ptr));

        out
    };

    handle_response(code, &result)
}
