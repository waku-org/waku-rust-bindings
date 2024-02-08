//! Waku [peer handling and connection](https://rfc.vac.dev/spec/36/#connecting-to-peers) methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
use multiaddr::Multiaddr;
// internal
use crate::general::Result;
use crate::utils::{get_trampoline, handle_json_response, handle_no_response, handle_response};

/// Dial peer using a multiaddress
/// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
/// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
/// Use 0 for no timeout
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
pub fn waku_connect(address: &Multiaddr, timeout: Option<Duration>) -> Result<()> {
    let address_ptr = CString::new(address.to_string())
        .expect("CString should build properly from multiaddress")
        .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_connect(
            address_ptr,
            timeout
                .map(|duration| duration.as_millis().try_into().unwrap_or(i32::MAX))
                .unwrap_or(0),
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(address_ptr));

        out
    };

    handle_no_response(code, &error)
}
