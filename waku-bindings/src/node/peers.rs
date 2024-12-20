//! Waku [peer handling and connection](https://rfc.vac.dev/spec/36/#connecting-to-peers) methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use libc::*;
use multiaddr::Multiaddr;
use std::sync::Arc;
use tokio::sync::Notify;
// internal
use crate::general::Result;
use crate::node::context::WakuNodeContext;
use crate::utils::LibwakuResponse;
use crate::utils::{get_trampoline, handle_no_response};

/// Dial peer using a multiaddress
/// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
/// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
/// Use 0 for no timeout
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
pub async fn waku_connect(
    ctx: &WakuNodeContext,
    address: &Multiaddr,
    timeout: Option<Duration>,
) -> Result<()> {
    let address =
        CString::new(address.to_string()).expect("CString should build properly from multiaddress");

    let address_ptr = address.as_ptr();

    let mut result = LibwakuResponse::default();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let result_cb = |r: LibwakuResponse| {
        result = r;
        notify_clone.notify_one(); // Notify that the value has been updated
    };
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_connect(
            ctx.get_ptr(),
            address_ptr,
            timeout
                .map(|duration| duration.as_millis().try_into().unwrap_or(u32::MAX))
                .unwrap_or(0),
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_no_response(code, result)
}
