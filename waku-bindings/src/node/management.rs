//! Node lifcycle [mangement](https://rfc.vac.dev/spec/36/#node-management) related methods

// std
use multiaddr::Multiaddr;
use std::ffi::CString;
// crates
use libc::*;
// internal
use super::config::WakuNodeConfig;
use crate::general::{PeerId, Result};
use crate::utils::{get_trampoline, handle_json_response, handle_no_response, handle_response};

/// Instantiates a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<()> {
    let config = config.unwrap_or_default();

    let config_ptr = CString::new(
        serde_json::to_string(&config)
            .expect("Serialization from properly built NodeConfig should never fail"),
    )
    .expect("CString should build properly from the config")
    .into_raw();

    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_new(config_ptr, cb, &mut closure as *mut _ as *mut c_void);

        drop(CString::from_raw(config_ptr));

        out
    };

    handle_no_response(code, &error)
}

/// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
pub fn waku_start() -> Result<()> {
    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_start(cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, &error)
}

/// Stops a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_stop() -> Result<()> {
    let mut error: String = Default::default();
    let error_cb = |v: &str| error = v.to_string();
    let code = unsafe {
        let mut closure = error_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_stop(cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_no_response(code, &error)
}

/// If the execution is successful, the result is the peer ID as a string (base58 encoded)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_peer_id() -> Result<PeerId> {
    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_peerid(cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_response(code, &result)
}

/// Get the multiaddresses the Waku node is listening to
/// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
pub fn waku_listen_addresses() -> Result<Vec<Multiaddr>> {
    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_listen_addresses(cb, &mut closure as *mut _ as *mut c_void)
    };

    handle_json_response(code, &result)
}

#[cfg(test)]
mod test {
    use super::waku_new;
    use crate::node::management::{waku_listen_addresses, waku_peer_id, waku_start, waku_stop};
    use crate::node::peers::waku_peer_count;
    use serial_test::serial;

    #[test]
    #[serial]
    fn waku_flow() {
        waku_new(None).unwrap();
        waku_start().unwrap();
        // test peer id call, since we cannot start different instances of the node
        let id = waku_peer_id().unwrap();
        dbg!(&id);
        assert!(!id.is_empty());

        let peer_cnt = waku_peer_count().unwrap();
        dbg!(peer_cnt);

        // test addresses, since we cannot start different instances of the node
        let addresses = waku_listen_addresses().unwrap();
        dbg!(&addresses);
        assert!(!addresses.is_empty());

        waku_stop().unwrap();
    }
}
