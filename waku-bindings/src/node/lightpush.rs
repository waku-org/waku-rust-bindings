//! Waku lightpush protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
use std::sync::Arc;
use tokio::sync::Notify;
// internal
use crate::general::{messagehash::MessageHash, Result, WakuMessage};
use crate::node::context::WakuNodeContext;
use crate::utils::{get_trampoline, handle_response, LibwakuResponse};

use crate::general::pubsubtopic::PubsubTopic;

pub async fn waku_lightpush_publish_message(
    ctx: &WakuNodeContext,
    message: &WakuMessage,
    pubsub_topic: &PubsubTopic,
) -> Result<MessageHash> {
    let message = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message");
    let message_ptr = message.as_ptr();

    let pubsub_topic = CString::new(String::from(pubsub_topic))
        .expect("CString should build properly from pubsub topic");
    let pubsub_topic_ptr = pubsub_topic.as_ptr();

    let mut result = LibwakuResponse::default();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let result_cb = |r: LibwakuResponse| {
        result = r;
        notify_clone.notify_one(); // Notify that the value has been updated
    };

    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_lightpush_publish(
            ctx.get_ptr(),
            pubsub_topic_ptr,
            message_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_response(code, result)
}
