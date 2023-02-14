//! Node lifcycle [mangement](https://rfc.vac.dev/spec/36/#node-management) related methods

// std
use multiaddr::Multiaddr;
use std::ffi::{CStr, CString};
// crates
// internal
use super::config::WakuNodeConfig;
use crate::general::{JsonResponse, PeerId, Result};
use crate::utils::decode_and_free_response;

/// Instantiates a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<bool> {
    let config = config.unwrap_or_default();

    let config_ptr = CString::new(
        serde_json::to_string(&config)
            .expect("Serialization from properly built NodeConfig should never fail"),
    )
    .expect("CString should build properly from the config")
    .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_new(config_ptr);
        drop(CString::from_raw(config_ptr));
        res
    };

    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    unsafe {
        waku_sys::waku_utils_free(result_ptr);
    }

    json_response.into()
}

/// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
pub fn waku_start() -> Result<bool> {
    let response_ptr = unsafe { waku_sys::waku_start() };
    let response = unsafe { CStr::from_ptr(response_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    unsafe {
        waku_sys::waku_utils_free(response_ptr);
    }

    json_response.into()
}

/// Stops a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_stop() -> Result<bool> {
    let response_ptr = unsafe { waku_sys::waku_stop() };
    let response = unsafe { CStr::from_ptr(response_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    unsafe {
        waku_sys::waku_utils_free(response_ptr);
    }

    json_response.into()
}

/// If the execution is successful, the result is the peer ID as a string (base58 encoded)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_peer_id() -> Result<PeerId> {
    let response_ptr = unsafe { waku_sys::waku_peerid() };
    let response = unsafe { CStr::from_ptr(response_ptr) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<String> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    unsafe {
        waku_sys::waku_utils_free(response_ptr);
    }

    json_response.into()
}

/// Get the multiaddresses the Waku node is listening to
/// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
pub fn waku_listen_addresses() -> Result<Vec<Multiaddr>> {
    let response_ptr = unsafe { waku_sys::waku_listen_addresses() };
    decode_and_free_response(response_ptr)
}

#[cfg(test)]
mod test {
    use super::waku_new;
    use crate::node::management::{waku_listen_addresses, waku_peer_id, waku_start, waku_stop};
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

        // test addresses, since we cannot start different instances of the node
        let addresses = waku_listen_addresses().unwrap();
        dbg!(&addresses);
        assert!(!addresses.is_empty());

        waku_stop().unwrap();
    }
}
