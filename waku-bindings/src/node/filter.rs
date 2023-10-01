//! Waku [filter](https://rfc.vac.dev/spec/36/#waku-filter) protocol related methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
// internal
use crate::general::Result;
use crate::general::{FilterSubscription, PeerId};
use crate::utils::{get_trampoline, handle_response, handle_no_response};

/// Creates a subscription in a lightnode for messages that matches a content filter and optionally a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_subscribechar-filterjson-char-peerid-int-timeoutms)
pub fn waku_filter_subscribe(
    filter_subscription: &FilterSubscription,
    peer_id: Option<PeerId>,
    timeout: Duration,
) -> Result<PeerId> {
    let filter_subscription_ptr = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("FilterSubscription should always be able to be serialized")
    .into_raw();
    let peer_id_ptr = match peer_id {
        None => CString::new(""),
        Some(t) => CString::new(t),
    }
    .expect("CString should build properly from peer id")
    .into_raw();

    let mut response: String = Default::default();
    let response_cb = |v: &str| response = v.to_string();
    let code = unsafe {
        let mut closure = response_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_subscribe(
            filter_subscription_ptr,
            peer_id_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(filter_subscription_ptr));
        drop(CString::from_raw(peer_id_ptr));

        out
    };

    TODO: extract the peerID from here?
    handle_response(code, &response) 
}

/// Used to know if a service node has an active subscription for this client
/// peerID should contain the ID of a peer we are subscribed to, supporting the filter protocol
pub fn waku_filter_ping(peer_id: PeerId, timeout: Duration) -> Result<()> {
    let peer_id_ptr = CString::new(peer_id)
        .expect("PeerId should always be able to be serialized")
        .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_ping(
            peer_id_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(peer_id_ptr));

        out
    };

    handle_no_response(code, &error)
}

/// Sends a requests to a service node to stop pushing messages matching this filter to this client.
/// It might be used to modify an existing subscription by providing a subset of the original filter
/// criteria
pub fn waku_filter_unsubscribe(
    filter_subscription: &FilterSubscription,
    peer_id: PeerId,
    timeout: Duration,
) -> Result<()> {
    let filter_subscription_ptr = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("CString should build properly from the serialized filter subscription")
    .into_raw();
    let peer_id_ptr = CString::new(peer_id)
        .expect("PeerId should always be able to be serialized")
        .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_unsubscribe(
            filter_subscription_ptr,
            peer_id_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(filter_subscription_ptr));
        drop(CString::from_raw(peer_id_ptr));

        out
    };

    handle_no_response(code, &error)
}

/// Sends a requests to a service node (or all service nodes) to stop pushing messages
/// peerID should contain the ID of a peer this client is subscribed to, or can be None
/// to stop all active subscriptions
pub fn waku_filter_unsubscribe_all(peer_id: Option<PeerId>, timeout: Duration) -> Result<()> {
    let peer_id_ptr = match peer_id {
        None => CString::new(""),
        Some(t) => CString::new(t),
    }
    .expect("CString should build properly from peer id")
    .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_filter_unsubscribe_all(
            peer_id_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(peer_id_ptr));

        out
    };

    handle_no_response(code, &error)
}
