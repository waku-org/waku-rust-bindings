//! Waku filter protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
// internal
use crate::general::contenttopic::WakuContentTopic;
use crate::general::pubsubtopic::PubsubTopic;
use crate::general::Result;
use crate::node::context::WakuNodeContext;
use crate::utils::{get_trampoline, handle_no_response, LibwakuResponse};

pub fn waku_filter_subscribe(
    ctx: &WakuNodeContext,
    pubsub_topic: &PubsubTopic,
    content_topics: Vec<WakuContentTopic>,
) -> Result<()> {
    let content_topics = WakuContentTopic::join_content_topics(content_topics);

    let pubsub_topic_ptr = CString::new(String::from(pubsub_topic))
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
            ctx.get_ptr(),
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
    pubsub_topic: &PubsubTopic,
    content_topics: Vec<WakuContentTopic>, // comma-separated list of content topics
) -> Result<()> {
    let content_topics_topics = WakuContentTopic::join_content_topics(content_topics);

    let pubsub_topic_ptr = CString::new(String::from(pubsub_topic))
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
            ctx.get_ptr(),
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
        waku_sys::waku_filter_unsubscribe_all(
            ctx.get_ptr(),
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    handle_no_response(code, result)
}
