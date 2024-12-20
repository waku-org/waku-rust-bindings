//! Waku [relay](https://rfc.vac.dev/spec/36/#waku-relay) protocol related methods

// std
use std::ffi::CString;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
// crates
use libc::*;
// internal
use crate::general::contenttopic::{Encoding, WakuContentTopic};
use crate::general::pubsubtopic::PubsubTopic;
use crate::general::{messagehash::MessageHash, Result, WakuMessage};
use crate::node::context::WakuNodeContext;
use crate::utils::{get_trampoline, handle_no_response, handle_response, LibwakuResponse};

/// Create a content topic according to [RFC 23](https://rfc.vac.dev/spec/23/)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_content_topicchar-applicationname-unsigned-int-applicationversion-char-contenttopicname-char-encoding)
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub async fn waku_create_content_topic(
    ctx: &WakuNodeContext,
    application_name: &str,
    application_version: u32,
    content_topic_name: &str,
    encoding: Encoding,
) -> WakuContentTopic {
    let application_name = CString::new(application_name)
        .expect("Application name should always transform to CString");
    let application_name_ptr = application_name.as_ptr();

    let content_topic_name =
        CString::new(content_topic_name).expect("Content topic should always transform to CString");
    let content_topic_name_ptr = content_topic_name.as_ptr();

    let encoding =
        CString::new(encoding.to_string()).expect("Encoding should always transform to CString");
    let encoding_ptr = encoding.as_ptr();

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
        waku_sys::waku_content_topic(
            ctx.get_ptr(),
            application_name_ptr,
            application_version,
            content_topic_name_ptr,
            encoding_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_response(code, result).expect("&str from result should always be extracted")
}

/// Publish a message using Waku Relay
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
pub async fn waku_relay_publish_message(
    ctx: &WakuNodeContext,
    message: &WakuMessage,
    pubsub_topic: &PubsubTopic,
    timeout: Option<Duration>,
) -> Result<MessageHash> {
    let message = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message");

    let pubsub_topic = CString::new(String::from(pubsub_topic))
        .expect("CString should build properly from pubsub topic");

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
        waku_sys::waku_relay_publish(
            ctx.get_ptr(),
            pubsub_topic.as_ptr(),
            message.as_ptr(),
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
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_response(code, result)
}

pub async fn waku_relay_subscribe(ctx: &WakuNodeContext, pubsub_topic: &PubsubTopic) -> Result<()> {
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
        waku_sys::waku_relay_subscribe(
            ctx.get_ptr(),
            pubsub_topic_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_no_response(code, result)
}

pub async fn waku_relay_unsubscribe(
    ctx: &WakuNodeContext,
    pubsub_topic: &PubsubTopic,
) -> Result<()> {
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
        waku_sys::waku_relay_subscribe(
            ctx.get_ptr(),
            pubsub_topic_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_no_response(code, result)
}
