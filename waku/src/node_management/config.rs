// std
// crates
use libsecp256k1::SecretKey;
use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
// internal

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    host: Option<std::net::IpAddr>,
    port: Option<usize>,
    advertise_addr: Option<Multiaddr>,
    #[serde(with = "secret_key_serde")]
    node_key: Option<SecretKey>,
    keep_alive_interval: Option<usize>,
    relay: Option<bool>,
    min_peers_to_publish: Option<usize>,
    filter: Option<bool>,
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
