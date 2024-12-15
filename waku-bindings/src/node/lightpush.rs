//! Waku lightpush protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
// internal
use crate::general::{messagehash::MessageHash, Result, WakuMessage};
use crate::node::context::WakuNodeContext;
use crate::utils::{get_trampoline, handle_response, LibwakuResponse};

use crate::general::pubsubtopic::PubsubTopic;

pub fn waku_lightpush_publish_message(
    ctx: &WakuNodeContext,
    message: &WakuMessage,
    pubsub_topic: &PubsubTopic,
) -> Result<MessageHash> {
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let pubsub_topic_ptr = CString::new(String::from(pubsub_topic))
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_lightpush_publish(
            ctx.get_ptr(),
            pubsub_topic_ptr,
            message_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(pubsub_topic_ptr));

        out
    };

    handle_response(code, result)
}
