//! Waku [general](https://rfc.vac.dev/spec/36/#general) types

pub mod contenttopic;
pub mod libwaku_response;
pub mod messagehash;
pub mod pubsubtopic;
pub mod time;
pub mod waku_decode;

// crates
use crate::general::time::get_now_in_nanosecs;
use contenttopic::WakuContentTopic;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

/// Waku message version
pub type WakuMessageVersion = usize;

pub type Result<T> = std::result::Result<T, String>;

// TODO: Properly type and deserialize payload form base64 encoded string
/// Waku message in JSON format.
/// as per the [specification](https://rfc.vac.dev/spec/36/#jsonmessage-type)
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessage {
    #[serde(with = "base64_serde", default = "Vec::new")]
    pub payload: Vec<u8>,
    /// The content topic to be set on the message
    pub content_topic: WakuContentTopic,
    /// The Waku Message version number
    #[serde(default)]
    pub version: WakuMessageVersion,
    /// Unix timestamp in nanoseconds
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub timestamp: u64,
    #[serde(with = "base64_serde", default = "Vec::new")]
    pub meta: Vec<u8>,
    #[serde(default)]
    pub ephemeral: bool,
    // TODO: implement RLN fields
    #[serde(flatten)]
    _extras: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WakuStoreRespMessage {
    // #[serde(with = "base64_serde", default = "Vec::new")]
    pub payload: Vec<u8>,
    /// The content topic to be set on the message
    // #[serde(rename = "contentTopic")]
    pub content_topic: String,
    // #[serde(with = "base64_serde", default = "Vec::new")]
    pub meta: Vec<u8>,
    /// The Waku Message version number
    #[serde(default)]
    pub version: WakuMessageVersion,
    /// Unix timestamp in nanoseconds
    pub timestamp: usize,
    #[serde(default)]
    pub ephemeral: bool,
    pub proof: Vec<u8>,
}

impl WakuMessage {
    pub fn new<PAYLOAD: AsRef<[u8]>, META: AsRef<[u8]>>(
        payload: PAYLOAD,
        content_topic: WakuContentTopic,
        version: WakuMessageVersion,
        meta: META,
        ephemeral: bool,
    ) -> Self {
        let payload = payload.as_ref().to_vec();
        let meta = meta.as_ref().to_vec();

        Self {
            payload,
            content_topic,
            version,
            timestamp: get_now_in_nanosecs(),
            meta,
            ephemeral,
            _extras: Default::default(),
        }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl WakuStoreRespMessage {
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

mod base64_serde {
    use base64::Engine;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        base64::engine::general_purpose::STANDARD
            .encode(value)
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base64_str: String = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(base64_str)
            .map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_waku_message() {
        let message = "{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660,\"ephemeral\":true,\"meta\":\"SGkgZnJvbSDwn6aAIQ==\"}";
        let _: WakuMessage = serde_json::from_str(message).unwrap();
    }
}
