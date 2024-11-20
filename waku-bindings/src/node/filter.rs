//! Waku filter protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
// internal
use crate::general::Result;
use crate::node::events::WakuNodeContext;
use crate::utils::{get_trampoline, handle_no_response, LibwakuResponse};

pub fn waku_filter_subscribe(
    ctx: &WakuNodeContext,
    pubsub_topic: &str,
    content_topics: &str, // comma-separated list of content topics
) -> Result<()> {
    let pubsub_topic = pubsub_topic.to_string();
    let content_topics = content_topics.to_string();

    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();
    let content_topics_ptr = CString::new(content_topics)
        .expect("CString should build properly from content topic")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_subscribe(
            ctx.obj_ptr,
            pubsub_topic_ptr,
            content_topics_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(pubsub_topic_ptr));
        drop(CString::from_raw(content_topics_ptr));

        out
    };

    handle_no_response(code, result)
}

pub fn waku_filter_unsubscribe(
    ctx: &WakuNodeContext,
    pubsub_topic: &str,
    content_topics_topics: &str, // comma-separated list of content topics
) -> Result<()> {
    let pubsub_topic = pubsub_topic.to_string();
    let content_topics_topics = content_topics_topics.to_string();

    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();
    let content_topics_topics_ptr = CString::new(content_topics_topics)
        .expect("CString should build properly from content topic")
        .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_unsubscribe(
            ctx.obj_ptr,
            pubsub_topic_ptr,
            content_topics_topics_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(pubsub_topic_ptr));
        drop(CString::from_raw(content_topics_topics_ptr));

        out
    };

    handle_no_response(code, result)
}

pub fn waku_filter_unsubscribe_all(ctx: &WakuNodeContext) -> Result<()> {
    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_unsubscribe_all(
            ctx.obj_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        out
    };

    handle_no_response(code, result)
}
