use crate::general::{JsonResponse, Result};
use serde::de::DeserializeOwned;
use std::ffi::{c_char, CStr};

/// Safety: The caller is responsible for ensuring that the pointer is valid for the duration of the call.
/// This takes a pointer to a C string coming from the waku lib, that data is consumed and then freed using [`waku_sys::waku_utils_free`].
pub fn decode_and_free_response<T: DeserializeOwned>(response_ptr: *mut c_char) -> Result<T> {
    let response = unsafe { CStr::from_ptr(response_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<T> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    unsafe {
        waku_sys::waku_utils_free(response_ptr);
    }

    response.into()
}
