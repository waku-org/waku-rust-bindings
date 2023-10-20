//! Waku [filter](https://rfc.vac.dev/spec/36/#waku-filter) protocol related methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
// internal
use crate::general::Result;
use crate::general::{LegacyFilterSubscription, PeerId};
use crate::utils::{get_trampoline, handle_no_response};

/// Creates a subscription in a lightnode for messages that matches a content filter and optionally a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_legacy_filter_subscribechar-filterjson-char-peerid-int-timeoutms)
pub fn waku_legacy_filter_subscribe(
    filter_subscription: &LegacyFilterSubscription,
    peer_id: PeerId,
    timeout: Duration,
) -> Result<()> {
    let filter_subscription_ptr = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("FilterSubscription should always be able to be serialized")
    .into_raw();
    let peer_id_ptr = CString::new(peer_id)
        .expect("PeerId should always be able to be serialized")
        .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_legacy_filter_subscribe(
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

/// Removes subscriptions in a light node matching a content filter and, optionally, a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_unsubscribechar-filterjson-int-timeoutms)
pub fn waku_legacy_filter_unsubscribe(
    filter_subscription: &LegacyFilterSubscription,
    timeout: Duration,
) -> Result<()> {
    let filter_subscription_ptr = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("CString should build properly from the serialized filter subscription")
    .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_legacy_filter_unsubscribe(
            filter_subscription_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(filter_subscription_ptr));

        out
    };

    handle_no_response(code, &error)
}
