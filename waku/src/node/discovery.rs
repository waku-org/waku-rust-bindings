// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates
use multiaddr::Multiaddr;
use url::Url;
// internal
use crate::general::JsonResponse;
use crate::Result;

/// RetrieveNodes returns a list of multiaddress given a url to a DNS discoverable ENR tree.
/// The nameserver can optionally be specified to resolve the enrtree url. Otherwise uses the default system dns.
pub fn waku_dns_discovery(
    url: &Url,
    server: Option<&str>,
    timeout: Option<Duration>,
) -> Result<Vec<Multiaddr>> {
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_dns_discovery(
            CString::new(url.to_string())
                .expect("CString should build properly from a valid Url")
                .into_raw(),
            CString::new(server.unwrap_or(""))
                .expect("CString should build properly from a String nameserver")
                .into_raw(),
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<Vec<Multiaddr>> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    response.into()
}
