//! Waku [filter](https://rfc.vac.dev/spec/36/#waku-filter) protocol related methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates

// internal
use crate::general::Result;
use crate::general::{FilterSubscription, JsonResponse, MessageId, PeerId};

/// Creates a subscription in a lightnode for messages that matches a content filter and optionally a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_subscribechar-filterjson-char-peerid-int-timeoutms)
pub fn waku_filter_subscribe(
    filter_subscription: &FilterSubscription,
    peer_id: PeerId,
    timeout: Duration,
) -> Result<()> {
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_filter_subscribe(
            CString::new(
                serde_json::to_string(filter_subscription)
                    .expect("FilterSubscription should always be able to be serialized"),
            )
            .expect("CString should build properly from the serialized filter subscription")
            .into_raw(),
            CString::new(peer_id)
                .expect("CString should build properly from peer id")
                .into_raw(),
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    Result::from(response).map(|_| ())
}

/// Removes subscriptions in a light node matching a content filter and, optionally, a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_unsubscribechar-filterjson-int-timeoutms)
pub fn waku_filter_unsubscribe(
    filter_subscription: &FilterSubscription,
    timeout: Duration,
) -> Result<()> {
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_filter_unsubscribe(
            CString::new(
                serde_json::to_string(filter_subscription)
                    .expect("FilterSubscription should always be able to be serialized"),
            )
            .expect("CString should build properly from the serialized filter subscription")
            .into_raw(),
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    Result::from(response).map(|_| ())
}
