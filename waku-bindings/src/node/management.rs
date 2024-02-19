//! Node lifcycle [mangement](https://rfc.vac.dev/spec/36/#node-management) related methods

// std
use std::ffi::CString;
// crates
use libc::c_void;
// internal
use super::config::WakuNodeConfig;
use crate::general::Result;
use crate::utils::{get_trampoline, handle_json_response, handle_no_response, handle_response};

/// Instantiates a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<*mut c_void> {
    let config = config.unwrap_or_default();

    let config_ptr = CString::new(
        serde_json::to_string(&config)
            .expect("Serialization from properly built NodeConfig should never fail"),
    )
    .expect("CString should build properly from the config")
    .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let node_ptr = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_new(config_ptr, cb, &mut closure as *mut _ as *mut c_void);

        drop(CString::from_raw(config_ptr));

        out
    };

    // TODO: create error handler function, format of err message is
    // {"message":"The actual message","eventType":"error"}
    if !error.is_empty() {
        return Err(error);
    }

    Ok(node_ptr)
}

/// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
pub fn waku_start(ctx: *mut c_void) -> Result<()> {
    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_start(ctx, cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, &error)
}

/// Stops a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_stop(ctx: *mut c_void) -> Result<()> {
    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_stop(ctx, cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, &error)
}

/// nwaku version
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn waku_version(ctx: *mut c_void) -> Result<String> {
    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_version(ctx, cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_response(code, &result)
}

#[cfg(test)]
mod test {
    use super::waku_new;
    use crate::node::management::{waku_start, waku_stop, waku_version};
    use serial_test::serial;

    #[test]
    #[serial]
    fn waku_flow() {
        let node = waku_new(None).unwrap();

        waku_start(node).unwrap();

        waku_stop(node).unwrap();
    }

    #[test]
    #[serial]
    fn nwaku_version() {
        let node = waku_new(None).unwrap();
        let version = waku_version(node).expect("should return the version");
        assert!(!version.is_empty());
    }
}
