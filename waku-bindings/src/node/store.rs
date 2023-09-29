//! Waku [store](https://rfc.vac.dev/spec/36/#waku-store) handling methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
// internal
use crate::general::{PeerId, Result, StoreQuery, StoreResponse};
use crate::utils::{get_trampoline, handle_json_response};

/// Retrieves historical messages on specific content topics. This method may be called with [`PagingOptions`](`crate::general::PagingOptions`),
/// to retrieve historical messages on a per-page basis. If the request included [`PagingOptions`](`crate::general::PagingOptions`),
/// the node must return messages on a per-page basis and include [`PagingOptions`](`crate::general::PagingOptions`) in the response.
/// These [`PagingOptions`](`crate::general::PagingOptions`) must contain a cursor pointing to the Index from which a new page can be requested
pub fn waku_store_query(
    query: &StoreQuery,
    peer_id: &PeerId,
    timeout: Option<Duration>,
) -> Result<StoreResponse> {
    let query_ptr = CString::new(
        serde_json::to_string(query).expect("StoreQuery should always be able to be serialized"),
    )
    .expect("CString should build properly from the serialized filter subscription")
    .into_raw();
    let peer_id_ptr = CString::new(peer_id.clone())
        .expect("CString should build properly from peer id")
        .into_raw();

    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_store_query(
            query_ptr,
            peer_id_ptr,
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(query_ptr));
        drop(CString::from_raw(peer_id_ptr));

        out
    };

    handle_json_response(code, &result)
}

/// Retrieves locally stored historical messages on specific content topics from the local archive system. This method may be called with [`PagingOptions`](`crate::general::PagingOptions`),
/// to retrieve historical messages on a per-page basis. If the request included [`PagingOptions`](`crate::general::PagingOptions`),
/// the node must return messages on a per-page basis and include [`PagingOptions`](`crate::general::PagingOptions`) in the response.
/// These [`PagingOptions`](`crate::general::PagingOptions`) must contain a cursor pointing to the Index from which a new page can be requested
pub fn waku_local_store_query(query: &StoreQuery) -> Result<StoreResponse> {
    let query_ptr = CString::new(
        serde_json::to_string(query).expect("StoreQuery should always be able to be serialized"),
    )
    .expect("CString should build properly from the serialized filter subscription")
    .into_raw();

    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out =
            waku_sys::waku_store_local_query(query_ptr, cb, &mut closure as *mut _ as *mut c_void);

        drop(CString::from_raw(query_ptr));

        out
    };

    handle_json_response(code, &result)
}
