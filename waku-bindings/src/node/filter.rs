//! Waku filter protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
use std::sync::Arc;
use tokio::sync::Notify;
// internal
use crate::general::contenttopic::WakuContentTopic;
use crate::general::libwaku_response::{handle_no_response, LibwakuResponse};
use crate::general::pubsubtopic::PubsubTopic;
use crate::general::Result;
use crate::macros::get_trampoline;
use crate::node::context::WakuNodeContext;

pub async fn waku_filter_subscribe(
    ctx: &WakuNodeContext,
    pubsub_topic: &PubsubTopic,
    content_topics: Vec<WakuContentTopic>,
) -> Result<()> {
    let content_topics = WakuContentTopic::join_content_topics(content_topics);

    let pubsub_topic = CString::new(String::from(pubsub_topic))
        .expect("CString should build properly from pubsub topic");
    let pubsub_topic_ptr = pubsub_topic.as_ptr();

    let content_topics =
        CString::new(content_topics).expect("CString should build properly from content topic");
    let content_topics_ptr = content_topics.as_ptr();

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
        waku_sys::waku_filter_subscribe(
            ctx.get_ptr(),
            pubsub_topic_ptr,
            content_topics_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_no_response(code, result)
}

pub async fn waku_filter_unsubscribe(
    ctx: &WakuNodeContext,
    pubsub_topic: &PubsubTopic,
    content_topics: Vec<WakuContentTopic>, // comma-separated list of content topics
) -> Result<()> {
    let content_topics_topics = WakuContentTopic::join_content_topics(content_topics);

    let pubsub_topic = CString::new(String::from(pubsub_topic))
        .expect("CString should build properly from pubsub topic");
    let pubsub_topic_ptr = pubsub_topic.as_ptr();

    let content_topics_topics = CString::new(content_topics_topics)
        .expect("CString should build properly from content topic");
    let content_topics_topics_ptr = content_topics_topics.as_ptr();

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
        waku_sys::waku_filter_unsubscribe(
            ctx.get_ptr(),
            pubsub_topic_ptr,
            content_topics_topics_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_no_response(code, result)
}

pub async fn waku_filter_unsubscribe_all(ctx: &WakuNodeContext) -> Result<()> {
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
        waku_sys::waku_filter_unsubscribe_all(
            ctx.get_ptr(),
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_no_response(code, result)
}
