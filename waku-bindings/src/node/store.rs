//! Waku store protocol related methods

// std
use std::ffi::CString;
// crates
use libc::*;
use std::sync::Arc;
use tokio::sync::Notify;
// internal
use crate::general::{
    contenttopic::WakuContentTopic, messagehash::MessageHash, pubsubtopic::PubsubTopic, Result,
    WakuStoreRespMessage,
};
use crate::node::context::WakuNodeContext;
use crate::utils::{get_trampoline, handle_response, LibwakuResponse, WakuDecode};
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PagingOptions {
    pub page_size: usize,
    pub cursor: Option<MessageHash>,
    pub forward: bool,
}

/// Criteria used to retrieve historical messages
#[derive(Clone, Serialize, Debug)]
struct StoreQueryRequest {
    /// if true, the store-response will include the full message content. If false,
    /// the store-response will only include a list of message hashes.
    request_id: String,
    include_data: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pubsub_topic: Option<PubsubTopic>,
    content_topics: Vec<WakuContentTopic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_hashes: Option<Vec<MessageHash>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination_cursor: Option<MessageHash>, // Message hash (key) from where to start query (exclusive)
    pagination_forward: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination_limit: Option<u64>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoreWakuMessageResponse {
    pub message_hash: MessageHash,
    pub message: WakuStoreRespMessage,
    pub pubsub_topic: String,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoreResponse {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: u32,
    pub status_desc: String,

    /// Array of retrieved historical messages in [`WakuMessage`] format
    // #[serde(default)]
    pub messages: Vec<StoreWakuMessageResponse>,
    /// Paging information in [`PagingOptions`] format from which to resume further historical queries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_cursor: Option<MessageHash>,
}

// Implement WakuDecode for Vec<Multiaddr>
impl WakuDecode for StoreResponse {
    fn decode(input: &str) -> Result<Self> {
        Ok(serde_json::from_str(input).expect("could not parse store resp"))
    }
}

pub async fn waku_store_query(
    ctx: &WakuNodeContext,
    request_id: String,
    include_data: bool,
    pubsub_topic: Option<PubsubTopic>,
    content_topics: Vec<WakuContentTopic>,
    time_start: Option<usize>,
    time_end: Option<usize>,
    message_hashes: Option<Vec<MessageHash>>,
    pagination_cursor: Option<MessageHash>, // Message hash (key) from where to start query (exclusive)
    pagination_forward: bool,
    pagination_limit: Option<u64>,
    peer_addr: &str,
    timeout_millis: Option<i32>,
) -> Result<StoreResponse> {
    let query = StoreQueryRequest {
        request_id,
        include_data,
        pubsub_topic,
        content_topics,
        time_start,
        time_end,
        message_hashes,
        pagination_cursor,
        pagination_forward,
        pagination_limit,
    };

    let json_query = CString::new(
        serde_json::to_string(&query).expect("StoreQuery should always be able to be serialized"),
    )
    .expect("CString should build properly from the serialized filter subscription");
    let json_query_ptr = json_query.as_ptr();

    peer_addr
        .parse::<Multiaddr>()
        .expect("correct multiaddress in store query");
    let peer_addr = CString::new(peer_addr).expect("peer_addr CString should be created");
    let peer_addr_ptr = peer_addr.as_ptr();

    let timeout_millis = timeout_millis.unwrap_or(10000i32);

    let mut result = LibwakuResponse::default();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let result_cb = |r: LibwakuResponse| {
        result = r;
        notify_clone.notify_one(); // Notify that the value has been updated
    };
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        waku_sys::waku_store_query(
            ctx.get_ptr(),
            json_query_ptr,
            peer_addr_ptr,
            timeout_millis,
            cb,
            &mut closure as *mut _ as *mut c_void,
        )
    };

    notify.notified().await; // Wait until a result is received
    handle_response(code, result)
}
