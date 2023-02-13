//! Waku [filter](https://rfc.vac.dev/spec/36/#waku-filter) protocol related methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates

// internal
use crate::general::Result;
use crate::general::{FilterSubscription, JsonResponse, PeerId};

/// Creates a subscription in a lightnode for messages that matches a content filter and optionally a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_subscribechar-filterjson-char-peerid-int-timeoutms)
pub fn waku_filter_subscribe(
    filter_subscription: &FilterSubscription,
    peer_id: PeerId,
    timeout: Duration,
) -> Result<()> {
    let filter_subscription = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("FilterSubscription should always be able to be serialized");
    let peer_id = CString::new(peer_id).expect("PeerId should always be able to be serialized");

    let result_ptr = unsafe {
        let filter_subscription_ptr = filter_subscription.into_raw();
        let peer_id_ptr = peer_id.into_raw();
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
    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    unsafe {
        waku_sys::waku_utils_free(result_ptr);
    }
    Result::from(response).map(|_| ())
}

/// Removes subscriptions in a light node matching a content filter and, optionally, a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_unsubscribechar-filterjson-int-timeoutms)
pub fn waku_filter_unsubscribe(
    filter_subscription: &FilterSubscription,
    timeout: Duration,
) -> Result<()> {
    let filter_subscription = CString::new(
        serde_json::to_string(filter_subscription)
            .expect("FilterSubscription should always succeed to serialize"),
    )
    .expect("CString should build properly from the serialized filter subscription");
    let result_ptr = unsafe {
        let filter_subscription_ptr = filter_subscription.into_raw();
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
    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    unsafe { waku_sys::waku_utils_free(result_ptr) };
    Result::from(response).map(|_| ())
}
