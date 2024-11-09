//! Waku [relay](https://rfc.vac.dev/spec/36/#waku-relay) protocol related methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
// internal
use crate::general::{Encoding, MessageHash, Result, WakuContentTopic, WakuMessage};
use crate::node::events::WakuNodeContext;
use crate::utils::{get_trampoline, handle_no_response, handle_response, LibwakuResponse};

/// Create a content topic according to [RFC 23](https://rfc.vac.dev/spec/23/)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_content_topicchar-applicationname-unsigned-int-applicationversion-char-contenttopicname-char-encoding)
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn waku_create_content_topic(
    ctx: &WakuNodeContext,
    application_name: &str,
    application_version: u32,
    content_topic_name: &str,
    encoding: Encoding,
) -> WakuContentTopic {
    let application_name_ptr = CString::new(application_name)
        .expect("Application name should always transform to CString")
        .into_raw();
    let content_topic_name_ptr = CString::new(content_topic_name)
        .expect("Content topic should always transform to CString")
        .into_raw();
    let encoding_ptr = CString::new(encoding.to_string())
        .expect("Encoding should always transform to CString")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_content_topic(
            ctx.obj_ptr,
            application_name_ptr,
            application_version,
            content_topic_name_ptr,
            encoding_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(application_name_ptr));
        drop(CString::from_raw(content_topic_name_ptr));
        drop(CString::from_raw(encoding_ptr));

        out
    };

    handle_response(code, result).expect("&str from result should always be extracted")
}

/// Publish a message using Waku Relay
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
pub fn waku_relay_publish_message(
    ctx: &WakuNodeContext,
    message: &WakuMessage,
    pubsub_topic: &str,
    timeout: Option<Duration>,
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
        let out = waku_sys::waku_relay_publish(
            ctx.obj_ptr,
            pubsub_topic_ptr,
            message_ptr,
            timeout
                .map(|duration| {
                    duration
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a u32")
                })
                .unwrap_or(0),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(pubsub_topic_ptr));

        out
    };

    handle_response(code, result)
}

pub fn waku_relay_subscribe(ctx: &WakuNodeContext, pubsub_topic: &str) -> Result<()> {
    let pubsub_topic = pubsub_topic.to_string();
    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_relay_subscribe(
            ctx.obj_ptr,
            pubsub_topic_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(pubsub_topic_ptr));

        out
    };

    handle_no_response(code, result)
}

pub fn waku_relay_unsubscribe(ctx: &WakuNodeContext, pubsub_topic: &String) -> Result<()> {
    let pubsub_topic = pubsub_topic.to_string();
    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_relay_subscribe(
            ctx.obj_ptr,
            pubsub_topic_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(pubsub_topic_ptr));

        out
    };

    handle_no_response(code, result)
}
