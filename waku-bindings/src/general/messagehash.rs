use crate::general::waku_decode::WakuDecode;
use hex::FromHex;
use serde::{Deserialize, Deserializer, Serialize};
use std::convert::TryInto;
use std::fmt;
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// Waku message hash, hex encoded sha256 digest of the message
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct MessageHash([u8; 32]);

impl MessageHash {
    fn to_hex_string(&self) -> String {
        self.0.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        })
    }
}

impl Hash for MessageHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use the inner array to contribute to the hash
        self.0.hash(state);
    }
}

impl FromStr for MessageHash {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        // Decode the hexadecimal string to a Vec<u8>
        // We expect a string format like: d38220de82fbcf2df865b680692fce98c36600fdd1d954b8a71e916dc4222b8e
        let bytes = Vec::from_hex(s).map_err(|e| format!("Hex decode error MessageHash: {}", e))?;

        // Ensure the length is exactly 32 bytes
        let res = bytes
            .try_into()
            .map_err(|_| "Hex string must represent exactly 32 bytes".to_string())?;

        Ok(MessageHash(res))
    }
}

impl<'de> Deserialize<'de> for MessageHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input as a vector of u8
        let vec: Vec<u8> = Deserialize::deserialize(deserializer)?;

        // Ensure the vector has exactly 32 elements
        let array: [u8; 32] = vec
            .try_into()
            .map_err(|_| serde::de::Error::custom("Expected an array of length 32"))?;

        Ok(MessageHash(array))
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
        write!(f, "{}", self.to_hex_string())
    }
}
