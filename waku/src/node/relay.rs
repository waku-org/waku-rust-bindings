use crate::general::{Encoding, WakuContentTopic, WakuPubSubTopic};
use std::ffi::{CStr, CString};

pub fn waku_create_content_topic(
    application_name: &str,
    application_version: usize,
    content_topic_name: &str,
    enconding: Encoding,
) -> WakuContentTopic {
    unsafe {
        CStr::from_ptr(waku_sys::waku_content_topic(
            CString::new(application_name)
                .expect("Application name should always transform to CString")
                .into_raw(),
            application_version
                .try_into()
                .expect("Version should fit within an u32"),
            CString::new(content_topic_name)
                .expect("Conmtent topic should always transform to CString")
                .into_raw(),
            CString::new(enconding.to_string())
                .expect("Encoding should always transform to CString")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("&str from result should always be")
    .parse()
    .expect("Content topic data should be always parseable")
}

pub fn waku_create_pubsub_topic(topic_name: &str, enconding: Encoding) -> WakuPubSubTopic {
    unsafe {
        CStr::from_ptr(waku_sys::waku_pubsub_topic(
            CString::new(topic_name)
                .expect("Topic name should always transform to CString")
                .into_raw(),
            CString::new(enconding.to_string())
                .expect("Encoding should always transform to CString")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("&str from result should always be")
    .parse()
    .expect("Pubsub topic data should be always parseable")
}
