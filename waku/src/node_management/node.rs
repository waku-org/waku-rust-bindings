// std
use multiaddr::Multiaddr;
use std::ffi::{CStr, CString};
// crates
// internal
use super::config::NodeConfig;
use crate::general::{JsonResponse, Result};

/// Instantiates a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<NodeConfig>) -> Result<bool> {
    let config = config.unwrap_or_default();
    let s_config = serde_json::to_string(&config)
        .expect("Serialization from properly built NodeConfig should never fail");
    let result: &str = unsafe {
        CStr::from_ptr(waku_sys::waku_new(
            CString::new(s_config)
                .expect("CString should build properly from the serialized node config")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");
    let json_response: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    json_response.into()
}

/// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
pub fn waku_start() -> Result<bool> {
    let response = unsafe { CStr::from_ptr(waku_sys::waku_start()) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");
    json_response.into()
}

/// Stops a Waku node
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_stop() -> Result<bool> {
    let response = unsafe { CStr::from_ptr(waku_sys::waku_start()) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");
    json_response.into()
}

/// If the execution is successful, the result is the peer ID as a string (base58 encoded)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
pub fn waku_peer_id() -> Result<String> {
    let response = unsafe { CStr::from_ptr(waku_sys::waku_peerid()) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<String> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    json_response.into()
}

/// Get the multiaddresses the Waku node is listening to
/// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
pub fn waku_listen_addressses() -> Result<Vec<Multiaddr>> {
    let response = unsafe { CStr::from_ptr(waku_sys::waku_listen_addresses()) }
        .to_str()
        .expect("Response should always succeed to load to a &str");

    let json_response: JsonResponse<Vec<Multiaddr>> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    json_response.into()
}

#[cfg(test)]
mod test {
    use super::waku_new;
    use crate::node_management::node::{
        waku_listen_addressses, waku_peer_id, waku_start, waku_stop,
    };

    #[test]
    fn waku_new_default() {
        let config = Default::default();
        assert!(waku_new(&config).unwrap());
    }

    #[test]
    fn waku_flow() {
        let config = Default::default();
        waku_new(&config).unwrap();
        waku_start().unwrap();
        waku_stop().unwrap();
    }

    #[test]
    fn waku_id() {
        let config = Default::default();
        waku_new(&config).unwrap();
        let id = waku_peer_id().unwrap();
        dbg!(&id);
        assert!(!id.is_empty());
    }

    #[test]
    fn waku_address() {
        let config = Default::default();
        waku_new(&config).unwrap();
        let addresses = waku_listen_addressses().unwrap();
        dbg!(&addresses);
        assert!(!addresses.is_empty());
    }
}
