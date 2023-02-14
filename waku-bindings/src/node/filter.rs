//! Waku [filter](https://rfc.vac.dev/spec/36/#waku-filter) protocol related methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates

// internal
use crate::general::Result;
use crate::general::{FilterSubscription, PeerId};
use crate::utils::decode_and_free_response;

/// Creates a subscription in a lightnode for messages that matches a content filter and optionally a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_subscribechar-filterjson-char-peerid-int-timeoutms)
pub fn waku_filter_subscribe(
    filter_subscription: &FilterSubscription,
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

    let result_ptr = unsafe {
        let result_ptr = waku_sys::waku_filter_subscribe(
            filter_subscription_ptr,
            peer_id_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        );
        drop(CString::from_raw(filter_subscription_ptr));
        drop(CString::from_raw(peer_id_ptr));
        result_ptr
    };
    decode_and_free_response::<bool>(result_ptr).map(|_| ())
}

/// Removes subscriptions in a light node matching a content filter and, optionally, a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_unsubscribechar-filterjson-int-timeoutms)
pub fn waku_filter_unsubscribe(
    filter_subscription: &FilterSubscription,
    timeout: Duration,
) -> Result<()> {
    let filter_subscription_ptr = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("CString should build properly from the serialized filter subscription")
    .into_raw();
    let result_ptr = unsafe {
        let res = waku_sys::waku_filter_unsubscribe(
            filter_subscription_ptr,
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        );
        drop(CString::from_raw(filter_subscription_ptr));
        res
    };

    decode_and_free_response::<bool>(result_ptr).map(|_| ())
}
