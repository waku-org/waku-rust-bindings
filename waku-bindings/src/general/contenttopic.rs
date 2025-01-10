// std
use crate::general::waku_decode::WakuDecode;
use crate::general::Result;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use sscanf::{scanf, RegexRepresentation};

/// WakuMessage encoding scheme
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum Encoding {
    #[default]
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
#[derive(Clone, Debug, Eq, PartialEq, Default)]
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

    pub fn join_content_topics(topics: Vec<WakuContentTopic>) -> String {
        topics
            .iter()
            .map(|topic| topic.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl WakuDecode for WakuContentTopic {
    fn decode(input: &str) -> Result<Self> {
        Ok(serde_json::from_str(input).expect("could not parse store resp"))
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
