//! Waku node [configuration](https://rfc.vac.dev/spec/36/#jsonconfig-type) related items

use std::fmt::{Display, Formatter};
use std::str::FromStr;
// std
// crates
use crate::WakuPubSubTopic;
use multiaddr::Multiaddr;
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
    /// External address to advertise to other nodes. Can be ip4, ip6 or dns4, dns6.
    /// If null, the multiaddress(es) generated from the ip and port specified in the config (or default ones) will be used.
    /// Default: null
    pub advertise_addr: Option<Multiaddr>,
    /// Secp256k1 private key in Hex format (`0x123...abc`). Default random
    #[serde(with = "secret_key_serde")]
    pub node_key: Option<SecretKey>,
    /// Interval in seconds for pinging peers to keep the connection alive. Default `20`
    #[default(Some(20))]
    pub keep_alive_interval: Option<usize>,
    /// Enable relay protocol. Default `true`
    #[default(Some(true))]
    pub relay: Option<bool>,
    /// Enable store protocol to persist message history
    #[default(Some(false))]
    pub store: Option<bool>,
    /// Url connection string. Accepts SQLite and PostgreSQL connection strings
    #[default(Some("sqlite3://store.db".to_string()))]
    pub database_url: Option<String>,
    /// Max number of messages to store in the databas
    #[default(Some(1000))]
    pub store_retention_max_messages: Option<usize>,
    /// Max number of seconds that a message will be persisted in the database, default 1 day
    #[default(Some(86400))]
    pub store_retention_max_seconds: Option<usize>,
    pub relay_topics: Vec<WakuPubSubTopic>,
    /// The minimum number of peers required on a topic to allow broadcasting a message. Default `0`
    #[default(Some(0))]
    pub min_peers_to_publish: Option<usize>,
    /// Enable filter protocol. Default `false`
    #[default(Some(false))]
    pub filter: Option<bool>,
    /// Set the log level. Default `INFO`. Allowed values "DEBUG", "INFO", "WARN", "ERROR", "DPANIC", "PANIC", "FATAL"
    #[default(Some(WakuLogLevel::Info))]
    pub log_level: Option<WakuLogLevel>,
    /// Enable DiscoveryV5. Default `false`
    #[default(Some(false))]
    #[serde(rename = "discV5")]
    pub discv5: Option<bool>,
    /// Array of bootstrap nodes ENR.
    #[serde(rename = "discV5BootstrapNodes", default)]
    pub discv5_bootstrap_nodes: Vec<String>,
    /// UDP port for DiscoveryV5. Default `9000`.
    #[default(Some(9000))]
    #[serde(rename = "discV5UDPPort")]
    pub discv5_udp_port: Option<u16>,
    /// Gossipsub custom configuration.
    pub gossipsub_params: Option<GossipSubParams>,
}

#[derive(Clone, SmartDefault, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GossipSubParams {
    /// Sets the optimal degree for a GossipSub topic mesh. For example, if D == 6,
    /// each peer will want to have about six peers in their mesh for each topic they're subscribed to.
    /// `d` should be set somewhere between `dlo` and `dhi`.
    #[serde(rename = "d")]
    pub d: Option<i32>,
    /// Sets the lower bound on the number of peers we keep in a GossipSub topic mesh.
    /// If we have fewer than dlo peers, we will attempt to graft some more into the mesh at
    /// the next heartbeat.
    #[serde(rename = "d_low")]
    pub dlo: Option<i32>,
    /// Sets the upper bound on the number of peers we keep in a GossipSub topic mesh.
    /// If we have more than dhi peers, we will select some to prune from the mesh at the next heartbeat.
    #[serde(rename = "d_high")]
    pub dhi: Option<i32>,
    /// `dscore` affects how peers are selected when pruning a mesh due to over subscription.
    /// At least dscore of the retained peers will be high-scoring, while the remainder are
    /// chosen randomly.
    #[serde(rename = "d_score")]
    pub dscore: Option<i32>,
    /// Sets the quota for the number of outbound connections to maintain in a topic mesh.
    /// When the mesh is pruned due to over subscription, we make sure that we have outbound connections
    /// to at least dout of the survivor peers. This prevents sybil attackers from overwhelming
    /// our mesh with incoming connections.
    ///
    /// dout must be set below Dlo, and must not exceed D / 2.
    #[serde(rename = "d_out")]
    pub dout: Option<i32>,
    /// Controls the size of the message cache used for gossip.
    /// The message cache will remember messages for history_length heartbeats.
    pub history_length: Option<i32>,
    /// Controls how many cached message ids we will advertise in
    /// IHAVE gossip messages. When asked for our seen message IDs, we will return
    /// only those from the most recent history_gossip heartbeats. The slack between
    /// history_gossip and history_length allows us to avoid advertising messages
    /// that will be expired by the time they're requested.
    ///
    /// history_gossip must be less than or equal to history_length to
    /// avoid a runtime panic.
    pub history_gossip: Option<i32>,
    /// dlazy affects how many peers we will emit gossip to at each heartbeat.
    /// We will send gossip to at least dlazy peers outside our mesh. The actual
    /// number may be more, depending on gossip_factor and how many peers we're
    /// connected to.
    pub dlazy: Option<i32>,
    /// `gossip_factor` affects how many peers we will emit gossip to at each heartbeat.
    /// We will send gossip to gossip_factor * (total number of non-mesh peers), or
    /// Dlazy, whichever is greater.
    pub gossip_factor: Option<f64>,
    /// Controls how many times we will allow a peer to request
    /// the same message id through IWANT gossip before we start ignoring them. This is designed
    /// to prevent peers from spamming us with requests and wasting our resources.
    pub gossip_retransmission: Option<i32>,
    /// Short delay before the heartbeat timer begins
    /// after the router is initialized.
    pub heartbeat_initial_delay_ms: Option<i32>,
    /// Controls the time between heartbeats.
    pub heartbeat_interval_seconds: Option<i32>,
    /// Duration threshold for heartbeat processing before emitting
    /// a warning; this would be indicative of an overloaded peer.
    pub slow_heartbeat_warning: Option<f64>,
    /// Controls how long we keep track of the fanout state. If it's been
    /// fanout_ttl_seconds since we've published to a topic that we're not subscribed to,
    /// we'll delete the fanout map for that topic.
    pub fanout_ttl_seconds: Option<i32>,
    /// Controls the number of peers to include in prune Peer eXchange.
    /// When we prune a peer that's eligible for PX (has a good score, etc), we will try to
    /// send them signed peer records for up to prune_peers other peers that we
    /// know of.
    pub prune_peers: Option<i32>,
    /// Controls the backoff time for pruned peers. This is how long
    /// a peer must wait before attempting to graft into our mesh again after being pruned.
    /// When pruning a peer, we send them our value of PruneBackoff so they know
    /// the minimum time to wait. Peers running older versions may not send a backoff time,
    /// so if we receive a prune message without one, we will wait at least PruneBackoff
    /// before attempting to re-graft.
    pub prune_backoff_seconds: Option<i32>,
    /// Controls the backoff time to use when unsuscribing
    /// from a topic. A peer should not resubscribe to this topic before this
    /// duration.
    pub unsubscribe_backoff_seconds: Option<i32>,
    /// Controls the number of active connection attempts for peers obtained through PX.
    pub connectors: Option<i32>,
    /// Sets the maximum number of pending connections for peers attempted through px.
    pub max_pending_connections: Option<i32>,
    /// Controls the timeout for connection attempts.
    pub connection_timeout_seconds: Option<i32>,
    /// Number of heartbeat ticks for attempting to reconnect direct peers
    /// that are not currently connected.
    pub direct_connect_ticks: Option<u64>,
    /// Initial delay before opening connections to direct peers
    pub direct_connect_initial_delay_seconds: Option<i32>,
    /// Number of heartbeat ticks for attempting to improve the mesh
    /// with opportunistic grafting. Every opportunistic_graft_ticks we will attempt to select some
    /// high-scoring mesh peers to replace lower-scoring ones, if the median score of our mesh peers falls
    /// below a threshold (see https://godoc.org/github.com/libp2p/go-libp2p-pubsub#PeerScoreThresholds).
    pub opportunistic_graft_ticks: Option<u64>,
    /// Number of peers to opportunistically graft.
    pub opportunistic_graft_peers: Option<i32>,
    /// If a GRAFT comes before graft_flood_threshold_seconds has elapsed since the last PRUNE,
    /// then there is an extra score penalty applied to the peer through P7.
    pub graft_flood_threshold_seconds: Option<i32>,
    /// Maximum number of messages to include in an IHAVE message.
    /// Also controls the maximum number of IHAVE ids we will accept and request with IWANT from a
    /// peer within a heartbeat, to protect from IHAVE floods. You should adjust this value from the
    /// default if your system is pushing more than 5000 messages in history_gossip heartbeats;
    /// with the defaults this is 1666 messages/s.
    #[serde(rename = "maxIHaveLength")]
    pub max_ihave_length: Option<i32>,
    /// Maximum number of IHAVE messages to accept from a peer within a heartbeat.
    #[serde(rename = "maxIHaveMessages")]
    pub max_ihave_messages: Option<i32>,
    /// Time to wait for a message requested through IWANT following an IHAVE advertisement.
    /// If the message is not received within this window, a broken promise is declared and
    /// the router may apply bahavioural penalties.
    #[serde(rename = "iwantFollowupTimeSeconds")]
    pub iwant_followup_time_seconds: Option<i32>,
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
                format!("Unrecognized waku log level: {s}. Allowed values \"DEBUG\", \"INFO\", \"WARN\", \"ERROR\", \"DPANIC\", \"PANIC\", \"FATAL\""),
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
