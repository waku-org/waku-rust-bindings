use crate::general::waku_decode::WakuDecode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

/// Waku message hash, hex encoded sha256 digest of the message
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Hash)]
pub struct MessageHash(String);

impl FromStr for MessageHash {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(MessageHash(s.to_string()))
    }
}

impl WakuDecode for MessageHash {
    fn decode(input: &str) -> Result<Self, String> {
        MessageHash::from_str(input)
    }
}

// Implement the Display trait
impl fmt::Display for MessageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
