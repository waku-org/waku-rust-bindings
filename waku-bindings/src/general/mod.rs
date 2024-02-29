//! Waku [general](https://rfc.vac.dev/spec/36/#general) types

// std
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
// crates
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_aux::prelude::*;
use sscanf::{scanf, RegexRepresentation};

/// Waku message version
pub type WakuMessageVersion = usize;
/// Waku message hash, hex encoded sha256 digest of the message
pub type MessageHash = String;

/// Waku response, just a `Result` with an `String` error.
pub type Result<T> = std::result::Result<T, String>;

// TODO: Properly type and deserialize payload form base64 encoded string
/// Waku message in JSON format.
/// as per the [specification](https://rfc.vac.dev/spec/36/#jsonmessage-type)
#[derive(Clone, Serialize, Deserialize, Debug)]
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
    pub timestamp: usize,
    #[serde(with = "base64_serde", default = "Vec::new")]
    pub meta: Vec<u8>,
    #[serde(default)]
    pub ephemeral: bool,
    // TODO: implement RLN fields
    #[serde(flatten)]
    _extras: serde_json::Value,
}

impl WakuMessage {
    pub fn new<PAYLOAD: AsRef<[u8]>, META: AsRef<[u8]>>(
        payload: PAYLOAD,
        content_topic: WakuContentTopic,
        version: WakuMessageVersion,
        timestamp: usize,
        meta: META,
        ephemeral: bool,
    ) -> Self {
        let payload = payload.as_ref().to_vec();
        let meta = meta.as_ref().to_vec();

        Self {
            payload,
            content_topic,
            version,
            timestamp,
            meta,
            ephemeral,
            _extras: Default::default(),
        }
    }
}

/// WakuMessage encoding scheme
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Encoding {
    Proto,
    Rlp,
    Rfc26,
    Unknown(String),
}

impl Display for Encoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Encoding::Proto => "proto",
            Encoding::Rlp => "rlp",
            Encoding::Rfc26 => "rfc26",
            Encoding::Unknown(value) => value,
        };
        f.write_str(s)
    }
}

impl FromStr for Encoding {
    type Err = std::io::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proto" => Ok(Self::Proto),
            "rlp" => Ok(Self::Rlp),
            "rfc26" => Ok(Self::Rfc26),
            encoding => Ok(Self::Unknown(encoding.to_string())),
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
    pub version: Cow<'static, str>,
    pub content_topic_name: Cow<'static, str>,
    pub encoding: Encoding,
}

impl WakuContentTopic {
    pub const fn new(
        application_name: &'static str,
        version: &'static str,
        content_topic_name: &'static str,
        encoding: Encoding,
    ) -> Self {
        Self {
            application_name: Cow::Borrowed(application_name),
            version: Cow::Borrowed(version),
            content_topic_name: Cow::Borrowed(content_topic_name),
            encoding,
        }
    }
}

impl FromStr for WakuContentTopic {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok((application_name, version, content_topic_name, encoding)) =
            scanf!(s, "/{}/{}/{}/{:/.+?/}", String, String, String, Encoding)
        {
            Ok(WakuContentTopic {
                application_name: Cow::Owned(application_name),
                version: Cow::Owned(version),
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
