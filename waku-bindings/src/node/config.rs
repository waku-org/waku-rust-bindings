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
    /// Cluster id that the node is running in
    #[default(Some(0))]
    pub cluster_id: Option<usize>,
    /// Enable relay protocol. Default `true`
    #[default(Some(true))]
    pub relay: Option<bool>,
    pub relay_topics: Vec<String>,
    /// RLN configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rln_relay: Option<RLNConfig>,
}

/// RLN Relay configuration
#[derive(Clone, SmartDefault, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RLNConfig {
    /// Indicates if RLN support will be enabled.
    pub enabled: bool,
    /// Index of the onchain commitment to use
    #[serde(skip_serializing_if = "Option::is_none", rename = "membership-index")]
    pub membership_index: Option<usize>,
    /// On-chain dynamic group management
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<bool>,
    /// Path to the RLN merkle tree sled db (https://github.com/spacejam/sled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_path: Option<String>,
    /// Message rate in bytes/sec after which verification of proofs should happen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_threshold: Option<usize>,
    /// Path for persisting rln-relay credential
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_path: Option<String>,
    /// HTTP address of an Ethereum testnet client e.g., http://localhost:8540/
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_client_address: Option<String>,
    /// Address of membership contract on an Ethereum testnet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_contract_address: Option<String>,
    /// Password for encrypting RLN credentials
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_password: Option<String>,
    /// Set a user message limit for the rln membership registration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_message_limit: Option<u64>,
    /// Epoch size in seconds used to rate limit RLN memberships
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch_sec: Option<u64>,
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
