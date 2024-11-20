//! Waku lightpush protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
// internal
use crate::general::{MessageHash, Result, WakuMessage};
use crate::node::events::WakuNodeContext;
use crate::utils::{get_trampoline, handle_response, LibwakuResponse};

pub fn waku_lightpush_publish_message(
    ctx: &WakuNodeContext,
    message: &WakuMessage,
    pubsub_topic: &str,
) -> Result<MessageHash> {
    let pubsub_topic = pubsub_topic.to_string();

    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_lightpush_publish(
            ctx.obj_ptr,
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
