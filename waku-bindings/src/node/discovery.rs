// std
use std::ffi::CString;
use std::time::Duration;
// crates
use multiaddr::Multiaddr;
use url::{Host, Url};
// internal
use crate::utils::decode_and_free_response;
use crate::Result;

/// RetrieveNodes returns a list of multiaddress given a url to a DNS discoverable ENR tree.
/// The nameserver can optionally be specified to resolve the enrtree url. Otherwise uses the default system dns.
pub fn waku_dns_discovery(
    url: &Url,
    server: Option<&Host>,
    timeout: Option<Duration>,
) -> Result<Vec<Multiaddr>> {
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
