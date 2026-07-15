//! Waku store protocol types.
//!
//! The query itself goes through `LogosDeliveryCtx::waku_store_query`, which
//! takes and returns JSON; these are the typed shapes on either side of it.

// crates
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// internal
use crate::general::time::get_now_in_nanosecs;
use crate::general::{
    contenttopic::WakuContentTopic, messagehash::MessageHash, pubsubtopic::PubsubTopic,
    WakuStoreRespMessage,
};

// #[derive(Clone, Serialize, Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
// pub struct PagingOptions {
//     pub page_size: usize,
//     pub cursor: Option<MessageHash>,
//     pub forward: bool,
// }

/// Criteria used to retrieve historical messages
#[derive(Clone, Serialize, Debug)]
pub struct StoreQueryRequest {
    /// if true, the store-response will include the full message content. If false,
    /// the store-response will only include a list of message hashes.
    #[serde(rename = "request_id")]
    request_id: String,
    #[serde(rename = "include_data")]
    include_data: bool,
    #[serde(rename = "pubsub_topic", skip_serializing_if = "Option::is_none")]
    pubsub_topic: Option<PubsubTopic>,
    #[serde(rename = "content_topics")]
    content_topics: Vec<WakuContentTopic>,
    #[serde(rename = "time_start", skip_serializing_if = "Option::is_none")]
    time_start: Option<u64>,
    #[serde(rename = "time_end", skip_serializing_if = "Option::is_none")]
    time_end: Option<u64>,
    #[serde(rename = "message_hashes", skip_serializing_if = "Option::is_none")]
    message_hashes: Option<Vec<MessageHash>>,
    #[serde(rename = "pagination_cursor", skip_serializing_if = "Option::is_none")]
    pagination_cursor: Option<MessageHash>, // Message hash (key) from where to start query (exclusive)
    #[serde(rename = "pagination_forward")]
    pagination_forward: bool,
    #[serde(rename = "pagination_limit", skip_serializing_if = "Option::is_none")]
    pagination_limit: Option<u64>,
}

impl StoreQueryRequest {
    pub fn new() -> Self {
        StoreQueryRequest {
            request_id: Uuid::new_v4().to_string(),
            include_data: true,
            pubsub_topic: None,
            content_topics: Vec::new(),
            time_start: Some(get_now_in_nanosecs()),
            time_end: Some(get_now_in_nanosecs()),
            message_hashes: None,
            pagination_cursor: None,
            pagination_forward: true,
            pagination_limit: Some(25),
        }
    }

    pub fn with_include_data(mut self, include_data: bool) -> Self {
        self.include_data = include_data;
        self
    }

    pub fn with_pubsub_topic(mut self, pubsub_topic: Option<PubsubTopic>) -> Self {
        self.pubsub_topic = pubsub_topic;
        self
    }

    pub fn with_content_topics(mut self, content_topics: Vec<WakuContentTopic>) -> Self {
        self.content_topics = content_topics;
        self
    }

    pub fn with_time_start(mut self, time_start: Option<u64>) -> Self {
        self.time_start = time_start;
        self
    }

    pub fn with_time_end(mut self, time_end: Option<u64>) -> Self {
        self.time_end = time_end;
        self
    }

    #[allow(dead_code)]
    pub fn with_message_hashes(mut self, message_hashes: Vec<MessageHash>) -> Self {
        self.message_hashes = Some(message_hashes);
        self
    }

    pub fn with_pagination_cursor(mut self, pagination_cursor: Option<MessageHash>) -> Self {
        self.pagination_cursor = pagination_cursor;
        self
    }

    pub fn with_pagination_forward(mut self, pagination_forward: bool) -> Self {
        self.pagination_forward = pagination_forward;
        self
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoreWakuMessageResponse {
    pub message_hash: MessageHash,
    pub message: Option<WakuStoreRespMessage>, // None if include_data == false
    pub pubsub_topic: String,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoreResponse {
    #[allow(unused)]
    pub request_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(unused)]
    pub status_code: u32,

    #[allow(unused)]
    pub status_desc: String,

    /// Array of retrieved historical messages in [`WakuMessage`] format
    // #[serde(default)]
    pub messages: Vec<StoreWakuMessageResponse>,
    /// Paging information in [`PagingOptions`] format from which to resume further historical queries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_cursor: Option<MessageHash>,
}
