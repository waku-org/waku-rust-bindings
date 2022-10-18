//! Waku [peer handling and connection](https://rfc.vac.dev/spec/36/#connecting-to-peers) methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates
use multiaddr::Multiaddr;
use serde::Deserialize;
// internal
use crate::general::{JsonResponse, PeerId, ProtocolId, Result};

/// Add a node multiaddress and protocol to the waku node’s peerstore.
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_add_peerchar-address-char-protocolid)
pub fn waku_add_peers(address: &Multiaddr, protocol_id: ProtocolId) -> Result<PeerId> {
    let response = unsafe {
        CStr::from_ptr(waku_sys::waku_add_peer(
            CString::new(address.to_string())
                .expect("CString should build properly from the address")
                .into_raw(),
            CString::new(protocol_id.to_string())
                .expect("CString should build properly from the protocol id")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("&str should build properly from the returning response");

    let result: JsonResponse<PeerId> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    result.into()
}

/// Dial peer using a multiaddress
/// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
/// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
/// Use 0 for no timeout
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
pub fn waku_connect_peer_with_address(
    address: &Multiaddr,
    timeout: Option<Duration>,
) -> Result<()> {
    let response = unsafe {
        CStr::from_ptr(waku_sys::waku_connect(
            CString::new(address.to_string())
                .expect("CString should build properly from multiaddress")
                .into_raw(),
            timeout
                .map(|duration| duration.as_millis().try_into().unwrap_or(i32::MAX))
                .unwrap_or(0),
        ))
    }
    .to_str()
    .expect("&str should build properly from the returning response");

    let result: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    Result::from(result).map(|_| ())
}

/// Dial peer using a peer id
/// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
/// The peer must be already known.
/// It must have been added before with [`waku_add_peers`] or previously dialed with [`waku_connect_peer_with_address`]
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peeridchar-peerid-int-timeoutms)
pub fn waku_connect_peer_with_id(peer_id: PeerId, timeout: Option<Duration>) -> Result<()> {
    let response = unsafe {
        CStr::from_ptr(waku_sys::waku_connect_peerid(
            CString::new(peer_id)
                .expect("CString should build properly from peer id")
                .into_raw(),
            timeout
                .map(|duration| duration.as_millis().try_into().unwrap_or(i32::MAX))
                .unwrap_or(0),
        ))
    }
    .to_str()
    .expect("&str should build properly from the returning response");

    let result: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    Result::from(result).map(|_| ())
}

/// Disconnect a peer using its peer id
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_disconnect_peerchar-peerid)
pub fn waku_disconnect_peer_with_id(peer_id: &PeerId) -> Result<()> {
    let response = unsafe {
        CStr::from_ptr(waku_sys::waku_disconnect(
            CString::new(peer_id.as_bytes())
                .expect("CString should build properly from peer id")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("&str should build properly from the returning response");

    let result: JsonResponse<bool> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    Result::from(result).map(|_| ())
}

/// Get number of connected peers
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_peer_count)
pub fn waku_peer_count() -> Result<usize> {
    let response = unsafe { CStr::from_ptr(waku_sys::waku_peer_cnt()) }
        .to_str()
        .expect("&str should build properly from the returning response");

    let result: JsonResponse<usize> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    result.into()
}

/// Waku peer supported protocol
///
/// Examples:
/// `"/ipfs/id/1.0.0"`
/// `"/vac/waku/relay/2.0.0"`
/// `"/ipfs/ping/1.0.0"`
pub type Protocol = String;

/// Peer data from known/connected waku nodes
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WakuPeerData {
    /// Waku peer id
    #[serde(alias = "peerID")]
    peer_id: PeerId,
    /// Supported node protocols
    protocols: Vec<Protocol>,
    /// Node available addresses
    #[serde(alias = "addrs")]
    addresses: Vec<Multiaddr>,
    /// Already connected flag
    connected: bool,
}

impl WakuPeerData {
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn protocols(&self) -> &[Protocol] {
        &self.protocols
    }

    pub fn addresses(&self) -> &[Multiaddr] {
        &self.addresses
    }

    pub fn connected(&self) -> bool {
        self.connected
    }
}

/// List of [`WakuPeerData`], return value from [`waku_peers`] funtion
pub type WakuPeers = Vec<WakuPeerData>;

/// Retrieve the list of peers known by the Waku node
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_peers)
pub fn waku_peers() -> Result<WakuPeers> {
    let response = unsafe { CStr::from_ptr(waku_sys::waku_peers()) }
        .to_str()
        .expect("&str should build properly from the returning response");

    let result: JsonResponse<WakuPeers> =
        serde_json::from_str(response).expect("JsonResponse should always succeed to deserialize");

    result.into()
}

#[cfg(test)]
mod tests {
    use crate::node::peers::WakuPeerData;

    #[test]
    fn deserialize_waku_peer_data() {
        let json_str = r#"{
      "peerID": "16Uiu2HAmJb2e28qLXxT5kZxVUUoJt72EMzNGXB47RedcBafeDCBA",
      "protocols": [
        "/ipfs/id/1.0.0",
        "/vac/waku/relay/2.0.0",
        "/ipfs/ping/1.0.0"
      ],
      "addrs": [
        "/ip4/1.2.3.4/tcp/30303"
      ],
      "connected": true
    }"#;
        let _: WakuPeerData = serde_json::from_str(json_str).unwrap();
    }
}
