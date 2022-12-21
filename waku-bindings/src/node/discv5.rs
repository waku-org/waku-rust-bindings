// std
use std::ffi::CStr;

// crates
use crate::general::JsonResponse;
use crate::Result;

/// Starts the DiscoveryV5 service to discover and connect to new peers
pub fn waku_discv5_start() -> Result<bool> {
    let result = unsafe { CStr::from_ptr(waku_sys::waku_discv5_start()) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    response.into()
}

/// Stops the DiscoveryV5 service
pub fn waku_discv5_stop() -> Result<bool> {
    let result = unsafe { CStr::from_ptr(waku_sys::waku_discv5_stop()) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    response.into()
}
