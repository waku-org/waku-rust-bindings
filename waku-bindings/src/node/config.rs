//! Waku node [configuration](https://rfc.vac.dev/spec/36/#jsonconfig-type) related items

use std::fmt::{Display, Formatter};
use std::str::FromStr;
// std
// crates
use multiaddr::Multiaddr;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
// internal

/// Waku node configuration
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
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
    /// Set the log level. Default `INFO`. Allowed values "DEBUG", "INFO", "WARN", "ERROR", "DPANIC", "PANIC", "FATAL"
    pub log_level: Option<WakuLogLevel>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum WakuLogLevel {
    #[default]
    Info,
    Debug,
    Warn,
    Error,
    DPanic,
    Panic,
    Fatal,
}

impl FromStr for WakuLogLevel {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            "dpanic" => Ok(Self::DPanic),
            "panic" => Ok(Self::Panic),
            "fatal" => Ok(Self::Fatal),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unrecognized waku log level: {}. Allowed values \"DEBUG\", \"INFO\", \"WARN\", \"ERROR\", \"DPANIC\", \"PANIC\", \"FATAL\"", s),
            )),
        }
    }
}

impl Display for WakuLogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            WakuLogLevel::Info => "INFO",
            WakuLogLevel::Debug => "DEBUG",
            WakuLogLevel::Warn => "WARN",
            WakuLogLevel::Error => "ERROR",
            WakuLogLevel::DPanic => "DPANIC",
            WakuLogLevel::Panic => "PANIC",
            WakuLogLevel::Fatal => "FATAL",
        };
        write!(f, "{tag}")
    }
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
