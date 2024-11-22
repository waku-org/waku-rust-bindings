//! Node lifcycle [mangement](https://rfc.vac.dev/spec/36/#node-management) related methods

// std
use std::ffi::CString;
// crates
use libc::c_void;
use multiaddr::Multiaddr;
// internal
use super::config::WakuNodeConfig;
use crate::general::Result;
use crate::node::context::WakuNodeContext;
use crate::utils::LibwakuResponse;
use crate::utils::WakuDecode;
use crate::utils::{get_trampoline, handle_json_response, handle_no_response, handle_response};

/// Instantiates a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeContext> {
    unsafe {
        waku_sys::waku_setup();
    }

    let config = config.unwrap_or_default();
    let config_ptr = CString::new(
        serde_json::to_string(&config)
            .expect("Serialization from properly built NodeConfig should never fail"),
    )
    .expect("CString should build properly from the config")
    .into_raw();

    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let obj_ptr = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_new(config_ptr, cb, &mut closure as *mut _ as *mut c_void);

        drop(CString::from_raw(config_ptr));

        out
    };

    match result {
        LibwakuResponse::MissingCallback => panic!("callback is required"),
        LibwakuResponse::Failure(v) => Err(v),
        _ => Ok(WakuNodeContext::new(obj_ptr)),
    }
}

pub fn waku_destroy(ctx: &WakuNodeContext) -> Result<()> {
    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_destroy(ctx.get_ptr(), cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, result)
}

/// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
pub fn waku_start(ctx: &WakuNodeContext) -> Result<()> {
    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_start(ctx.get_ptr(), cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, result)
}

/// Stops a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_stop(ctx: &WakuNodeContext) -> Result<()> {
    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_stop(ctx.get_ptr(), cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, result)
}

/// nwaku version
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn waku_version(ctx: &WakuNodeContext) -> Result<String> {
    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_version(ctx.get_ptr(), cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_response(code, result)
}

// Implement WakuDecode for Vec<Multiaddr>
impl WakuDecode for Vec<Multiaddr> {
    fn decode(input: &str) -> Result<Self> {
        input
            .split(',')
            .map(|s| s.trim().parse::<Multiaddr>().map_err(|err| err.to_string()))
            .collect::<Result<Vec<Multiaddr>>>() // Collect results into a Vec
            .map_err(|err| format!("could not parse Multiaddr: {}", err))
    }
}

/// Get the multiaddresses the Waku node is listening to
/// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
pub fn waku_listen_addresses(ctx: &WakuNodeContext) -> Result<Vec<Multiaddr>> {
    let mut result: LibwakuResponse = Default::default();
    let result_cb = |r: LibwakuResponse| result = r;
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_listen_addresses(ctx.get_ptr(), cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_json_response(code, result)
}

#[cfg(test)]
mod test {
    use super::waku_new;
    use crate::node::management::{waku_listen_addresses, waku_start, waku_stop, waku_version};
    use serial_test::serial;

    #[test]
    #[serial]
    fn waku_flow() {
        let node = waku_new(None).unwrap();

        waku_start(&node).unwrap();

        // test addresses
        let addresses = waku_listen_addresses(&node).unwrap();
        dbg!(&addresses);
        assert!(!addresses.is_empty());

        waku_stop(&node).unwrap();
    }

    #[test]
    #[serial]
    fn nwaku_version() {
        let node = waku_new(None).unwrap();
        let version = waku_version(&node).expect("should return the version");
        assert!(!version.is_empty());
    }
}
