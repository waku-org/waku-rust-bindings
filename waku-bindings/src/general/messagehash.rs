use crate::general::waku_decode::WakuDecode;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt;
use std::fmt::Write;
use std::hash::Hash;
use std::str::FromStr;

/// Waku message hash, hex encoded sha256 digest of the message
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Hash)]
pub struct MessageHash([u8; 32]);

impl MessageHash {
    fn to_hex_string(&self) -> String {
        self.0.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02X}");
            output
        })
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
