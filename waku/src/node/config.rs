//! Waku node [configuration](https://rfc.vac.dev/spec/36/#jsonconfig-type) related items

// std
// crates
use libsecp256k1::SecretKey;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
// internal

/// Waku node configuration
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakuNodeConfig {
    /// Listening IP address. Default `0.0.0.0`
    pub host: Option<std::net::IpAddr>,
    /// Libp2p TCP listening port. Default `60000`. Use `0` for **random**
    pub port: Option<usize>,
    /// External address to advertise to other nodes. Can be ip4, ip6 or dns4, dns6.
    /// If null, the multiaddress(es) generated from the ip and port specified in the config (or default ones) will be used.
    /// Default: null
    pub advertise_addr: Option<Multiaddr>,
    /// Secp256k1 private key in Hex format (`0x123...abc`). Default random
    #[serde(with = "secret_key_serde")]
    pub node_key: Option<SecretKey>,
    /// Interval in seconds for pinging peers to keep the connection alive. Default `20`
    pub keep_alive_interval: Option<usize>,
    /// Enable relay protocol. Default `true`
    pub relay: Option<bool>,
    /// The minimum number of peers required on a topic to allow broadcasting a message. Default `0`
    pub min_peers_to_publish: Option<usize>,
    /// Enable filter protocol. Default `false`
    pub filter: Option<bool>,
}

mod secret_key_serde {
    use libsecp256k1::SecretKey;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(key: &Option<SecretKey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let as_string: Option<String> = key.as_ref().map(|key| hex::encode(key.serialize()));
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
                    SecretKey::parse_slice(&key_bytes)
                        .map_err(|e| D::Error::custom(format!("{e}")))?,
                ))
            }
        }
    }
}
