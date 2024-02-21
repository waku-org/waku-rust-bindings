//! Waku node [configuration](https://rfc.vac.dev/spec/36/#jsonconfig-type) related items

// std
// crates
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
// internal

/// Waku node configuration
#[derive(Clone, SmartDefault, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WakuNodeConfig {
    /// Listening IP address. Default `0.0.0.0`
    #[default(Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))))]
    pub host: Option<std::net::IpAddr>,
    /// Libp2p TCP listening port. Default `60000`. Use `0` for **random**
    #[default(Some(60000))]
    pub port: Option<usize>,
    /// Secp256k1 private key in Hex format (`0x123...abc`). Default random
    #[serde(with = "secret_key_serde", rename = "key")]
    pub node_key: Option<SecretKey>,
    /// Enable relay protocol. Default `true`
    #[default(Some(true))]
    pub relay: Option<bool>,
    pub relay_topics: Vec<String>,
}

mod secret_key_serde {
    use secp256k1::SecretKey;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &Option<SecretKey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let as_string: Option<String> = key.as_ref().map(|key| hex::encode(key.secret_bytes()));
        as_string.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SecretKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_string: Option<String> = Option::<String>::deserialize(deserializer)?;
        match as_string {
            None => Ok(None),
            Some(s) => {
                let key_bytes = hex::decode(s).map_err(|e| D::Error::custom(format!("{e}")))?;
                Ok(Some(
                    SecretKey::from_slice(&key_bytes)
                        .map_err(|e| D::Error::custom(format!("{e}")))?,
                ))
            }
        }
    }
}
