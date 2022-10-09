//! Waku [store](https://rfc.vac.dev/spec/36/#waku-store) handling methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates
// internal
use crate::general::{JsonResponse, PeerId, Result, StoreQuery, StoreResponse};

/// Retrieves historical messages on specific content topics. This method may be called with [`PagingOptions`](`crate::general::PagingOptions`),
/// to retrieve historical messages on a per-page basis. If the request included [`PagingOptions`](`crate::general::PagingOptions`),
/// the node must return messages on a per-page basis and include [`PagingOptions`](`crate::general::PagingOptions`) in the response.
/// These [`PagingOptions`](`crate::general::PagingOptions`) must contain a cursor pointing to the Index from which a new page can be requested
pub fn waku_store_query(
    query: &StoreQuery,
    peer_id: PeerId,
    timeout: Duration,
) -> Result<StoreResponse> {
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_store_query(
            CString::new(
                serde_json::to_string(query)
                    .expect("StoreQuery should always be able to be serialized"),
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

    let response: JsonResponse<StoreResponse> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    response.into()
}
