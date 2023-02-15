//! Waku [general](https://rfc.vac.dev/spec/36/#general) types

use std::borrow::Cow;
// std
use std::fmt::{Display, Formatter};
use std::str::FromStr;
// crates
use aes_gcm::{Aes256Gcm, Key};
use secp256k1::{ecdsa::Signature, PublicKey, SecretKey};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use sscanf::{scanf, RegexRepresentation};
// internal
use crate::decrypt::{waku_decode_asymmetric, waku_decode_symmetric};

/// Waku message version
pub type WakuMessageVersion = usize;
/// Base58 encoded peer id
pub type PeerId = String;
/// Waku message id, hex encoded sha256 digest of the message
pub type MessageId = String;

/// Protocol identifiers
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ProtocolId {
    Store,
    Lightpush,
    Filter,
    Relay,
}

impl ProtocolId {
    pub fn as_string_with_version(&self, version: &str) -> String {
        format!("{self}/{version}")
    }
}

impl Display for ProtocolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            ProtocolId::Store => "/vac/waku/store",
            ProtocolId::Lightpush => "/vac/waku/lightpush",
            ProtocolId::Filter => "/vac/waku/filter",
            ProtocolId::Relay => "/vac/waku/relay",
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        };
        write!(f, "{tag}")
    }
}

/// JsonResponse wrapper.
/// `go-waku` ffi returns this type as a `char *` as per the [specification](https://rfc.vac.dev/spec/36/#jsonresponse-type)
/// This is internal, as it is better to use rust plain `Result` type.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JsonResponse<T> {
    Result(T),
    Error(String),
}

/// Waku response, just a `Result` with an `String` error.
pub type Result<T> = std::result::Result<T, String>;

/// Convenient we can transform a [`JsonResponse`] into a [`std::result::Result`]
impl<T> From<JsonResponse<T>> for Result<T> {
    fn from(response: JsonResponse<T>) -> Self {
        match response {
            JsonResponse::Result(t) => Ok(t),
            JsonResponse::Error(e) => Err(e),
        }
    }
}

// TODO: Properly type and deserialize payload form base64 encoded string
/// Waku message in JSON format.
/// as per the [specification](https://rfc.vac.dev/spec/36/#jsonmessage-type)
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessage {
    #[serde(with = "base64_serde")]
    payload: Vec<u8>,
    /// The content topic to be set on the message
    content_topic: WakuContentTopic,
    // TODO: check if missing default should be 0
    /// The Waku Message version number
    #[serde(default)]
    version: WakuMessageVersion,
    /// Unix timestamp in nanoseconds
    timestamp: usize,
    // TODO: implement RLN fields
    #[serde(flatten)]
    _extras: serde_json::Value,
}

impl WakuMessage {
    pub fn new<PAYLOAD: AsRef<[u8]>>(
        payload: PAYLOAD,
        content_topic: WakuContentTopic,
        version: WakuMessageVersion,
        timestamp: usize,
    ) -> Self {
        let payload = payload.as_ref().to_vec();

        Self {
            payload,
            content_topic,
            version,
            timestamp,
            _extras: Default::default(),
        }
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn content_topic(&self) -> &WakuContentTopic {
        &self.content_topic
    }

    pub fn version(&self) -> WakuMessageVersion {
        self.version
    }

    pub fn timestamp(&self) -> usize {
        self.timestamp
    }

    /// Try decode the message with an expected symmetric key
    ///
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_decode_symmetricchar-messagejson-char-symmetrickey)
    pub fn try_decode_symmetric(&self, symmetric_key: &Key<Aes256Gcm>) -> Result<DecodedPayload> {
        waku_decode_symmetric(self, symmetric_key)
    }

    /// Try decode the message with an expected asymmetric key
    ///
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_decode_asymmetricchar-messagejson-char-privatekey)
    pub fn try_decode_asymmetric(&self, asymmetric_key: &SecretKey) -> Result<DecodedPayload> {
        waku_decode_asymmetric(self, asymmetric_key)
    }
}

/// A payload once decoded, used when a received Waku Message is encrypted
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DecodedPayload {
    /// Public key that signed the message (optional), hex encoded with 0x prefix
    #[serde(deserialize_with = "deserialize_optional_pk", default)]
    public_key: Option<PublicKey>,
    /// Message signature (optional), hex encoded with 0x prefix
    #[serde(deserialize_with = "deserialize_optional_signature", default)]
    signature: Option<Signature>,
    /// Decrypted message payload base64 encoded
    #[serde(with = "base64_serde")]
    data: Vec<u8>,
    /// Padding base64 encoded
    #[serde(with = "base64_serde")]
    padding: Vec<u8>,
}

impl DecodedPayload {
    pub fn public_key(&self) -> Option<&PublicKey> {
        self.public_key.as_ref()
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn padding(&self) -> &[u8] {
        &self.padding
    }
}

/// The content topic of a Waku message
/// as per the [specification](https://rfc.vac.dev/spec/36/#contentfilter-type)
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentFilter {
    /// The content topic of a Waku message
    content_topic: WakuContentTopic,
}

impl ContentFilter {
    pub fn new(content_topic: WakuContentTopic) -> Self {
        Self { content_topic }
    }

    pub fn content_topic(&self) -> &WakuContentTopic {
        &self.content_topic
    }
}

/// The criteria to create subscription to a light node in JSON Format
/// as per the [specification](https://rfc.vac.dev/spec/36/#filtersubscription-type)
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterSubscription {
    /// Array of [`ContentFilter`] being subscribed to / unsubscribed from
    content_filters: Vec<ContentFilter>,
    /// Optional pubsub topic
    pubsub_topic: Option<WakuPubSubTopic>,
}

impl FilterSubscription {
    pub fn new(content_filters: Vec<ContentFilter>, pubsub_topic: Option<WakuPubSubTopic>) -> Self {
        Self {
            content_filters,
            pubsub_topic,
        }
    }

    pub fn content_filters(&self) -> &[ContentFilter] {
        &self.content_filters
    }

    pub fn pubsub_topic(&self) -> Option<&WakuPubSubTopic> {
        self.pubsub_topic.as_ref()
    }
}

/// Criteria used to retrieve historical messages
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoreQuery {
    /// The pubsub topic on which messages are published
    pub pubsub_topic: Option<WakuPubSubTopic>,
    /// Array of [`ContentFilter`] to query for historical messages
    pub content_filters: Vec<ContentFilter>,
    /// The inclusive lower bound on the timestamp of queried messages.
    /// This field holds the Unix epoch time in nanoseconds
    pub start_time: Option<usize>,
    /// The inclusive upper bound on the timestamp of queried messages.
    /// This field holds the Unix epoch time in nanoseconds
    pub end_time: Option<usize>,
    /// Paging information in [`PagingOptions`] format
    pub paging_options: Option<PagingOptions>,
}

/// The response received after doing a query to a store node
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoreResponse {
    /// Array of retrieved historical messages in [`WakuMessage`] format
    #[serde(default)]
    pub messages: Vec<WakuMessage>,
    /// Paging information in [`PagingOptions`] format from which to resume further historical queries
    pub paging_options: Option<PagingOptions>,
}

impl StoreResponse {
    pub fn messages(&self) -> &[WakuMessage] {
        &self.messages
    }

    pub fn paging_options(&self) -> Option<&PagingOptions> {
        self.paging_options.as_ref()
    }
}

/// Paging information
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PagingOptions {
    /// Number of messages to retrieve per page
    pub page_size: usize,
    /// Message Index from which to perform pagination.
    /// If not included and forward is set to `true`, paging will be performed from the beginning of the list.
    /// If not included and forward is set to `false`, paging will be performed from the end of the list
    pub cursor: Option<MessageIndex>,
    /// `true` if paging forward, `false` if paging backward
    pub forward: bool,
}

/// Pagination index type
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageIndex {
    /// Hash of the message at this [``MessageIndex`]
    pub digest: String,
    /// UNIX timestamp in nanoseconds at which the message at this [`MessageIndex`] was received
    pub receiver_time: usize,
    /// UNIX timestamp in nanoseconds at which the message is generated by its sender
    pub sender_time: usize,
    /// The pubsub topic of the message at this [`MessageIndex`]
    pub pubsub_topic: WakuPubSubTopic,
}

/// WakuMessage encoding scheme
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Encoding {
    Proto,
    Rlp,
    Rfc26,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Encoding::Proto => "proto",
            Encoding::Rlp => "rlp",
            Encoding::Rfc26 => "rfc26",
        };
        f.write_str(s)
    }
}

impl FromStr for Encoding {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(Self::Proto),
            "rlp" => Ok(Self::Rlp),
            "rfc26" => Ok(Self::Rfc26),
            encoding => Err(format!("Unrecognized encoding: {encoding}")),
        }
    }
}

impl RegexRepresentation for Encoding {
    const REGEX: &'static str = r"\w";
}

/// A waku content topic `/{application_name}/{version}/{content_topic_name}/{encdoing}`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WakuContentTopic {
    pub application_name: Cow<'static, str>,
    pub version: usize,
    pub content_topic_name: Cow<'static, str>,
    pub encoding: Encoding,
}

impl WakuContentTopic {
    pub const fn new(
        application_name: &'static str,
        version: usize,
        content_topic_name: &'static str,
        encoding: Encoding,
    ) -> Self {
        Self {
            application_name: Cow::Borrowed(application_name),
            version,
            content_topic_name: Cow::Borrowed(content_topic_name),
            encoding,
        }
    }
}

impl FromStr for WakuContentTopic {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok((application_name, version, content_topic_name, encoding)) =
            scanf!(s, "/{}/{}/{}/{:/.+?/}", String, usize, String, Encoding)
        {
            Ok(WakuContentTopic {
                application_name: Cow::Owned(application_name),
                version,
                content_topic_name: Cow::Owned(content_topic_name),
                encoding,
            })
        } else {
            Err(
                format!(
                    "Wrong pub-sub topic format. Should be `/{{application-name}}/{{version-of-the-application}}/{{content-topic-name}}/{{encoding}}`. Got: {s}"
                )
            )
        }
    }
}

impl Display for WakuContentTopic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "/{}/{}/{}/{}",
            self.application_name, self.version, self.content_topic_name, self.encoding
        )
    }
}

impl Serialize for WakuContentTopic {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WakuContentTopic {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_string: String = String::deserialize(deserializer)?;
        as_string
            .parse::<WakuContentTopic>()
            .map_err(D::Error::custom)
    }
}

/// A waku pubsub topic in the form of `/waku/v2/{topic_name}/{encoding}`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WakuPubSubTopic {
    pub topic_name: Cow<'static, str>,
    pub encoding: Encoding,
}

impl WakuPubSubTopic {
    pub const fn new(topic_name: &'static str, encoding: Encoding) -> Self {
        Self {
            topic_name: Cow::Borrowed(topic_name),
            encoding,
        }
    }

    pub fn with_topic_name(topic_name: String, encoding: Encoding) -> Self {
        Self {
            topic_name: Cow::Owned(topic_name),
            encoding,
        }
    }
}

impl FromStr for WakuPubSubTopic {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok((topic_name, encoding)) = scanf!(s, "/waku/2/{}/{:/.+?/}", String, Encoding) {
            Ok(WakuPubSubTopic {
                topic_name: Cow::Owned(topic_name),
                encoding,
            })
        } else {
            Err(
                format!(
                    "Wrong pub-sub topic format. Should be `/waku/2/{{topic-name}}/{{encoding}}`. Got: {s}"
                )
            )
        }
    }
}

impl Display for WakuPubSubTopic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "/waku/2/{}/{}", self.topic_name, self.encoding)
    }
}

impl Serialize for WakuPubSubTopic {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WakuPubSubTopic {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_string: String = String::deserialize(deserializer)?;
        as_string
            .parse::<WakuPubSubTopic>()
            .map_err(D::Error::custom)
    }
}

mod base64_serde {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        base64::encode(value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base64_str: String = String::deserialize(deserializer)?;
        base64::decode(base64_str).map_err(D::Error::custom)
    }
}

pub fn deserialize_optional_pk<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<PublicKey>, D::Error>
where
    D: Deserializer<'de>,
{
    let base64_str: Option<String> = Option::<String>::deserialize(deserializer)?;
    base64_str
        .map(|base64_str| {
            let raw_bytes = base64::decode(base64_str).map_err(D::Error::custom)?;
            PublicKey::from_slice(&raw_bytes).map_err(D::Error::custom)
        })
        .transpose()
}

pub fn deserialize_optional_signature<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Signature>, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_str: Option<String> = Option::<String>::deserialize(deserializer)?;
    hex_str
        .map(|hex_str| {
            let raw_bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(&hex_str))
                .map_err(D::Error::custom)?;
            if ![64, 65].contains(&raw_bytes.len()) {
                return Err(D::Error::custom(
                    "Invalid signature, only 64 or 65 bytes len are supported",
                ));
            }
            Signature::from_compact(&raw_bytes[..64]).map_err(D::Error::custom)
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WakuPubSubTopic;
    #[test]
    fn parse_waku_topic() {
        let s = "/waku/2/default-waku/proto";
        let _: WakuPubSubTopic = s.parse().unwrap();
    }

    #[test]
    fn deserialize_waku_message() {
        let message = "{\"payload\":\"SGkgZnJvbSDwn6aAIQ==\",\"contentTopic\":\"/toychat/2/huilong/proto\",\"timestamp\":1665580926660}";
        let _: WakuMessage = serde_json::from_str(message).unwrap();
    }
}
