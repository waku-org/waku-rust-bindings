//! Waku node implementation

mod config;
mod filter;
mod lightpush;
mod management;
mod peers;
mod relay;
mod store;

// std
use aes_gcm::{Aes256Gcm, Key};
use libsecp256k1::{PublicKey, SecretKey};
use multiaddr::Multiaddr;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::time::Duration;
// crates
// internal
use crate::general::{
    FilterSubscription, MessageId, PeerId, Result, StoreQuery, StoreResponse, WakuMessage,
    WakuPubSubTopic,
};

pub use config::WakuNodeConfig;
pub use peers::{Protocol, WakuPeerData, WakuPeers};
pub use relay::{waku_create_content_topic, waku_create_pubsub_topic, waku_dafault_pubsub_topic};

/// Shared flag to check if a waku node is already running in the current process
static WAKU_NODE_INITIALIZED: Mutex<bool> = Mutex::new(false);

/// Marker trait to disallow undesired waku node states in the handle
pub trait WakuNodeState {}

/// Waku node initialized state
pub struct Initialized;

/// Waku node running state
pub struct Running;

impl WakuNodeState for Initialized {}
impl WakuNodeState for Running {}

/// Handle to the underliying waku node
/// Safe to sendt to/through threads.
/// Only a waku node can be running at a time.
/// Referenes (`&`) to the handle can call queries and perform operations in a thread safe way.
/// Only an owned version of the handle can `start` or `stop` the node.
pub struct WakuNodeHandle<State: WakuNodeState>(PhantomData<State>);

/// We do not have any inner state, so the handle should be safe to be send among threads.
unsafe impl<State: WakuNodeState> Send for WakuNodeHandle<State> {}

/// References to the handle are safe to share, as they do not mutate the handle itself and
/// operations are performed by the bindings backend, which is supposed to be thread safe.
unsafe impl<State: WakuNodeState> Sync for WakuNodeHandle<State> {}

impl<State: WakuNodeState> WakuNodeHandle<State> {
    /// If the execution is successful, the result is the peer ID as a string (base58 encoded)
    ///
    /// wrapper around [`management::waku_peer_id`]
    pub fn peer_id(&self) -> Result<PeerId> {
        management::waku_peer_id()
    }

    /// Get the multiaddresses the Waku node is listening to
    ///
    /// wrapper around [`management::waku_listen_addresses`]
    pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
        management::waku_listen_addresses()
    }

    /// Add a node multiaddress and protocol to the waku node’s peerstore
    ///
    /// wrapper around [`peers::waku_add_peers`]
    pub fn add_peer(&self, address: Multiaddr, protocol_id: usize) -> Result<PeerId> {
        peers::waku_add_peers(address, protocol_id)
    }
}

fn stop_node() -> Result<()> {
    let mut node_initialized = WAKU_NODE_INITIALIZED
        .lock()
        .expect("Access to the mutex at some point");
    *node_initialized = false;
    management::waku_stop().map(|_| ())
}

impl WakuNodeHandle<Initialized> {
    /// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation
    ///
    /// wrapper around [`management::waku_start`]
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        management::waku_start().map(|_| WakuNodeHandle(Default::default()))
    }

    /// Stops a Waku node
    ///
    /// internally uses [`management::waku_stop`]
    pub fn stop(self) -> Result<()> {
        stop_node()
    }
}

impl WakuNodeHandle<Running> {
    /// Stops a Waku node
    ///
    /// internally uses [`management::waku_stop`]
    pub fn stop(self) -> Result<()> {
        stop_node()
    }

    /// Dial peer using a multiaddress
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
    /// Use 0 for no timeout
    ///
    /// wrapper around [`peers::waku_connect_peer_with_address`]
    pub fn connect_peer_with_address(
        &self,
        address: Multiaddr,
        timeout: Option<Duration>,
    ) -> Result<()> {
        peers::waku_connect_peer_with_address(address, timeout)
    }

    /// Dial peer using its peer ID
    ///
    /// wrapper around [`peers::waku_connect_peer_with_id`]
    pub fn connect_peer_with_id(&self, peer_id: PeerId, timeout: Option<Duration>) -> Result<()> {
        peers::waku_connect_peer_with_id(peer_id, timeout)
    }

    /// Disconnect a peer using its peerID
    ///
    /// wrapper around [`peers::waku_disconnect_peer_with_id`]
    pub fn disconnect_peer_with_id(&self, peer_id: PeerId) -> Result<()> {
        peers::waku_disconnect_peer_with_id(peer_id)
    }

    /// Get number of connected peers
    ///
    /// wrapper around [`peers::waku_peer_count`]
    pub fn peer_count(&self) -> Result<usize> {
        peers::waku_peer_count()
    }

    /// Retrieve the list of peers known by the Waku node
    ///
    /// wrapper around [`peers::waku_peers`]
    pub fn peers(&self) -> Result<WakuPeers> {
        peers::waku_peers()
    }

    /// Publish a message using Waku Relay
    ///
    /// wrapper around [`relay::waku_relay_publish_message`]
    pub fn relay_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        timeout: Duration,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_message(message, pubsub_topic, timeout)
    }

    /// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Relay
    ///
    /// wrapper around [`relay::waku_relay_publish_encrypt_asymmetric`]
    pub fn relay_publish_encrypt_asymmetric(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        public_key: &PublicKey,
        signing_key: Option<&SecretKey>,
        timeout: Duration,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_encrypt_asymmetric(
            message,
            pubsub_topic,
            public_key,
            signing_key,
            timeout,
        )
    }

    /// Optionally sign, encrypt using symmetric encryption and publish a message using Waku Relay
    ///
    /// wrapper around [`relay::waku_relay_publish_encrypt_symmetric`]
    pub fn relay_publish_encrypt_symmetric(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        symmetric_key: &Key<Aes256Gcm>,
        signing_key: Option<&SecretKey>,
        timeout: Duration,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_encrypt_symmetric(
            message,
            pubsub_topic,
            symmetric_key,
            signing_key,
            timeout,
        )
    }

    /// Determine if there are enough peers to publish a message on a given pubsub topic
    ///
    /// wrapper around [`relay::waku_enough_peers`]
    pub fn relay_enough_peers(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<bool> {
        relay::waku_enough_peers(pubsub_topic)
    }

    /// Subscribe to a Waku Relay pubsub topic to receive messages
    ///
    /// wrapper around [`relay::waku_relay_subscribe`]
    pub fn relay_subscribe(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
        relay::waku_relay_subscribe(pubsub_topic)
    }

    /// Closes the pubsub subscription to a pubsub topic. No more messages will be received from this pubsub topic
    ///
    /// wrapper around [`relay::waku_relay_unsubscribe`]
    pub fn relay_unsubscribe(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
        relay::waku_relay_unsubscribe(pubsub_topic)
    }

    /// Retrieves historical messages on specific content topics
    ///
    /// wrapper around [`store::waku_store_query`]
    pub fn store_query(
        query: &StoreQuery,
        peer_id: PeerId,
        timeout: Duration,
    ) -> Result<StoreResponse> {
        store::waku_store_query(query, peer_id, timeout)
    }

    /// Publish a message using Waku Lightpush
    ///
    /// wrapper around [`lightpush::waku_lightpush_publish`]
    pub fn lightpush_publish(
        message: &WakuMessage,
        pubsub_topic: WakuPubSubTopic,
        peer_id: PeerId,
        timeout: Duration,
    ) -> Result<MessageId> {
        lightpush::waku_lightpush_publish(message, pubsub_topic, peer_id, timeout)
    }

    /// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Lightpush
    ///
    /// wrapper around [`lightpush::waku_lightpush_publish_encrypt_asymmetric`]
    pub fn lightpush_publish_encrypt_asymmetric(
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        peer_id: PeerId,
        public_key: &PublicKey,
        signing_key: Option<&SecretKey>,
        timeout: Duration,
    ) -> Result<MessageId> {
        lightpush::waku_lightpush_publish_encrypt_asymmetric(
            message,
            pubsub_topic,
            peer_id,
            public_key,
            signing_key,
            timeout,
        )
    }

    /// Optionally sign, encrypt using symmetric encryption and publish a message using Waku Lightpush
    ///
    /// wrapper around [`lightpush::waku_lightpush_publish_encrypt_symmetric`]
    pub fn lightpush_publish_encrypt_symmetric(
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        peer_id: PeerId,
        symmetric_key: &Key<Aes256Gcm>,
        signing_key: Option<&SecretKey>,
        timeout: Duration,
    ) -> Result<MessageId> {
        lightpush::waku_lightpush_publish_encrypt_symmetric(
            message,
            pubsub_topic,
            peer_id,
            symmetric_key,
            signing_key,
            timeout,
        )
    }

    /// Creates a subscription in a lightnode for messages that matches a content filter and optionally a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
    ///
    /// wrapper around [`filter::waku_filter_subscribe`]
    pub fn filter_subscribe(
        filter_subscription: &FilterSubscription,
        peer_id: PeerId,
        timeout: Duration,
    ) -> Result<()> {
        filter::waku_filter_subscribe(filter_subscription, peer_id, timeout)
    }

    /// Removes subscriptions in a light node matching a content filter and, optionally, a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
    ///
    /// wrapper around [`filter::waku_filter_unsubscribe`]
    pub fn filter_unsubscribe(
        filter_subscription: &FilterSubscription,
        timeout: Duration,
    ) -> Result<()> {
        filter::waku_filter_unsubscribe(filter_subscription, timeout)
    }
}

/// Spawn a new Waku node with the givent configuration (default configuration if `None` provided)
/// Internally uses [`management::waku_new`]
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle<Initialized>> {
    let mut node_initialized = WAKU_NODE_INITIALIZED
        .lock()
        .expect("Access to the mutex at some point");
    if *node_initialized {
        return Err("Waku node is already initialized".into());
    }
    *node_initialized = true;
    management::waku_new(config).map(|_| WakuNodeHandle(Default::default()))
}

#[cfg(test)]
mod tests {
    use super::waku_new;

    #[test]
    fn exclusive_running() {
        let handle1 = waku_new(None).unwrap();
        let handle2 = waku_new(None);
        assert!(handle2.is_err());
        let stop_handle = handle1.start().unwrap();
        stop_handle.stop().unwrap();
    }
}
