//! Waku [peer handling and connection](https://rfc.vac.dev/spec/36/#connecting-to-peers) methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use multiaddr::Multiaddr;
use serde::Deserialize;
// internal
use crate::general::{PeerId, ProtocolId, Result};
use crate::utils::decode_and_free_response;

/// Add a node multiaddress and protocol to the waku node’s peerstore.
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_add_peerchar-address-char-protocolid)
pub fn waku_add_peers(address: &Multiaddr, protocol_id: ProtocolId) -> Result<PeerId> {
    let address_ptr = CString::new(address.to_string())
        .expect("CString should build properly from the address")
        .into_raw();
    let protocol_id_ptr = CString::new(protocol_id.to_string())
        .expect("CString should build properly from the protocol id")
        .into_raw();

    let response_ptr = unsafe {
        let res = waku_sys::waku_add_peer(address_ptr, protocol_id_ptr);
        drop(CString::from_raw(address_ptr));
        drop(CString::from_raw(protocol_id_ptr));
        res
    };

    decode_and_free_response(response_ptr)
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
    let address_ptr = CString::new(address.to_string())
        .expect("CString should build properly from multiaddress")
        .into_raw();
    let response_ptr = unsafe {
        let res = waku_sys::waku_connect(
            address_ptr,
            timeout
                .map(|duration| duration.as_millis().try_into().unwrap_or(i32::MAX))
                .unwrap_or(0),
        );
        drop(CString::from_raw(address_ptr));
        res
    };

    decode_and_free_response::<bool>(response_ptr).map(|_| ())
}

/// Dial peer using a peer id
/// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
/// The peer must be already known.
/// It must have been added before with [`waku_add_peers`] or previously dialed with [`waku_connect_peer_with_address`]
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peeridchar-peerid-int-timeoutms)
pub fn waku_connect_peer_with_id(peer_id: &PeerId, timeout: Option<Duration>) -> Result<()> {
    let peer_id_ptr = CString::new(peer_id.as_bytes())
        .expect("CString should build properly from peer id")
        .into_raw();
    let result_ptr = unsafe {
        let res = waku_sys::waku_connect_peerid(
            peer_id_ptr,
            timeout
                .map(|duration| duration.as_millis().try_into().unwrap_or(i32::MAX))
                .unwrap_or(0),
        );
        drop(CString::from_raw(peer_id_ptr));
        res
    };

    decode_and_free_response::<bool>(result_ptr).map(|_| ())
}

/// Disconnect a peer using its peer id
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_disconnect_peerchar-peerid)
pub fn waku_disconnect_peer_with_id(peer_id: &PeerId) -> Result<()> {
    let peer_id_ptr = CString::new(peer_id.as_bytes())
        .expect("CString should build properly from peer id")
        .into_raw();

    let response_ptr = unsafe {
        let res = waku_sys::waku_disconnect(peer_id_ptr);
        drop(CString::from_raw(peer_id_ptr));
        res
    };
    decode_and_free_response::<bool>(response_ptr).map(|_| ())
}

/// Get number of connected peers
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_peer_count)
pub fn waku_peer_count() -> Result<usize> {
    let response_ptr = unsafe { waku_sys::waku_peer_cnt() };
    let num_str = decode_and_free_response::<String>(response_ptr)?;
    let num = num_str
        .parse::<u32>()
        .map_err(|_| "could not convert peer count into u32".to_string())?;
    usize::try_from(num).map_err(|_| "could not convert peer count into usize".to_string())
}

/// Waku peer supported protocol
///
/// Examples:
/// `"/ipfs/id/1.0.0"`
/// `"/vac/waku/relay/2.0.0"`
/// `"/ipfs/ping/1.0.0"`
pub type Protocol = String;

/// Peer data from known/connected waku nodes
#[derive(Deserialize, Clone, Debug)]
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

/// List of [`WakuPeerData`]
pub type WakuPeers = Vec<WakuPeerData>;

/// Retrieve the list of peers known by the Waku node
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_peers)
pub fn waku_peers() -> Result<WakuPeers> {
    let response_ptr = unsafe { waku_sys::waku_peers() };
    decode_and_free_response(response_ptr)
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
