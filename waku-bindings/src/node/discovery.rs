// std
use std::ffi::CString;
use std::time::Duration;
// crates
use enr::Enr;
use multiaddr::Multiaddr;
use serde::Deserialize;
use url::{Host, Url};
// internal
use crate::utils::decode_and_free_response;
use crate::{PeerId, Result};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DnsInfo {
    #[serde(alias = "peerID")]
    pub peer_id: PeerId,
    #[serde(default, alias = "multiaddrs")]
    pub addresses: Vec<Multiaddr>,
    pub enr: Option<Enr<enr::secp256k1::SecretKey>>,
}

/// RetrieveNodes returns a list of multiaddress given a url to a DNS discoverable ENR tree.
/// The nameserver can optionally be specified to resolve the enrtree url. Otherwise uses the default system dns.
pub fn waku_dns_discovery(
    url: &Url,
    server: Option<&Host>,
    timeout: Option<Duration>,
) -> Result<Vec<DnsInfo>> {
    let url = CString::new(url.to_string())
        .expect("CString should build properly from a valid Url")
        .into_raw();
    let server = CString::new(
        server
            .map(|host| host.to_string())
            .unwrap_or_else(|| "".to_string()),
    )
    .expect("CString should build properly from a String nameserver")
    .into_raw();
    let result_ptr = unsafe {
        let res = waku_sys::waku_dns_discovery(
            url,
            server,
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        );
        // Recover strings and drop them
        drop(CString::from_raw(url));
        drop(CString::from_raw(server));
        res
    };

    decode_and_free_response(result_ptr)
}

/// Update the bootnodes used by DiscoveryV5 by passing a list of ENRs
pub fn waku_discv5_update_bootnodes(
    bootnodes: Vec<String>
) -> Result<()> {
    let bootnodes_ptr = CString::new(
        serde_json::to_string(&bootnodes)
            .expect("Serialization from properly built bootnode array should never fail"),
    )
    .expect("CString should build properly from the string vector")
    .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_discv5_update_bootnodes(bootnodes_ptr);
        drop(CString::from_raw(bootnodes_ptr));
        res
    };

    decode_and_free_response::<bool>(result_ptr).map(|_| ())
}

#[cfg(test)]
mod test {
    use url::Url;

    #[test]
    fn test_dns_discovery() {
        let enrtree: Url =
            "enrtree://AOGECG2SPND25EEFMAJ5WF3KSGJNSGV356DSTL2YVLLZWIV6SAYBM@test.waku.nodes.status.im".parse().unwrap();
        let result = super::waku_dns_discovery(&enrtree, None, None);
        assert!(result.is_ok());
        assert!(!result.as_ref().unwrap().is_empty());
        println!("{result:?}");
    }
}
