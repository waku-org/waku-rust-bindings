//! Waku node implementation

mod config;
mod discovery;
mod filter;
mod lightpush;
mod management;
mod peers;
mod relay;
mod store;

// std
pub use aes_gcm::{Aes256Gcm, Key};
pub use multiaddr::Multiaddr;
pub use secp256k1::{PublicKey, SecretKey};
use std::marker::PhantomData;
use std::sync::Mutex;
use std::time::Duration;
// crates
// internal

use crate::general::{
    FilterSubscription, MessageId, PeerId, ProtocolId, Result, StoreQuery, StoreResponse,
    WakuMessage, WakuPubSubTopic,
};

pub use config::{GossipSubParams, WakuLogLevel, WakuNodeConfig};
pub use discovery::{waku_dns_discovery, DnsInfo};
pub use peers::{Protocol, WakuPeerData, WakuPeers};
pub use relay::{waku_create_content_topic, waku_create_pubsub_topic, waku_dafault_pubsub_topic};
pub use store::{waku_local_store_query, waku_store_query};

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
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn peer_id(&self) -> Result<PeerId> {
        management::waku_peer_id()
    }

    /// Get the multiaddresses the Waku node is listening to
    /// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
    pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
        management::waku_listen_addresses()
    }

    /// Add a node multiaddress and protocol to the waku node’s peerstore.
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_add_peerchar-address-char-protocolid)
    pub fn add_peer(&self, address: &Multiaddr, protocol_id: ProtocolId) -> Result<PeerId> {
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
    /// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        management::waku_start().map(|_| WakuNodeHandle(Default::default()))
    }

    /// Stops a Waku node
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn stop(self) -> Result<()> {
        stop_node()
    }
}

impl WakuNodeHandle<Running> {
    /// Stops a Waku node
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn stop(self) -> Result<()> {
        stop_node()
    }

    /// Dial peer using a multiaddress
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
    /// Use 0 for no timeout
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
    pub fn connect_peer_with_address(
        &self,
        address: &Multiaddr,
        timeout: Option<Duration>,
    ) -> Result<()> {
        peers::waku_connect_peer_with_address(address, timeout)
    }

    /// Dial peer using a peer id
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// The peer must be already known.
    /// It must have been added before with [`WakuNodeHandle::add_peer`] or previously dialed with [`WakuNodeHandle::connect_peer_with_address`]
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peeridchar-peerid-int-timeoutms)
    pub fn connect_peer_with_id(&self, peer_id: &PeerId, timeout: Option<Duration>) -> Result<()> {
        peers::waku_connect_peer_with_id(peer_id, timeout)
    }

    /// Disconnect a peer using its peer id
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_disconnect_peerchar-peerid)
    pub fn disconnect_peer_with_id(&self, peer_id: &PeerId) -> Result<()> {
        peers::waku_disconnect_peer_with_id(peer_id)
    }

    /// Get number of connected peers
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_peer_count)
    pub fn peer_count(&self) -> Result<usize> {
        peers::waku_peer_count()
    }

    /// Retrieve the list of peers known by the Waku node
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_peers)
    pub fn peers(&self) -> Result<WakuPeers> {
        peers::waku_peers()
    }

    /// Publish a message using Waku Relay
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
    pub fn relay_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        timeout: Option<Duration>,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_message(message, pubsub_topic, timeout)
    }

    /// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Relay
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publish_enc_asymmetricchar-messagejson-char-pubsubtopic-char-publickey-char-optionalsigningkey-int-timeoutms)
    pub fn relay_publish_encrypt_asymmetric(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        public_key: &PublicKey,
        signing_key: Option<&SecretKey>,
        timeout: Option<Duration>,
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
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publish_enc_symmetricchar-messagejson-char-pubsubtopic-char-symmetrickey-char-optionalsigningkey-int-timeoutms)
    pub fn relay_publish_encrypt_symmetric(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        symmetric_key: &Key<Aes256Gcm>,
        signing_key: Option<&SecretKey>,
        timeout: Option<Duration>,
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
    pub fn relay_enough_peers(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<bool> {
        relay::waku_enough_peers(pubsub_topic)
    }

    /// Subscribe to a Waku Relay pubsub topic to receive messages
    pub fn relay_subscribe(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
        relay::waku_relay_subscribe(pubsub_topic)
    }

    /// Closes the pubsub subscription to a pubsub topic. No more messages will be received from this pubsub topic
    pub fn relay_unsubscribe(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
        relay::waku_relay_unsubscribe(pubsub_topic)
    }

    /// Retrieves historical messages on specific content topics. This method may be called with [`PagingOptions`](`crate::general::PagingOptions`),
    /// to retrieve historical messages on a per-page basis. If the request included [`PagingOptions`](`crate::general::PagingOptions`),
    /// the node must return messages on a per-page basis and include [`PagingOptions`](`crate::general::PagingOptions`) in the response.
    /// These [`PagingOptions`](`crate::general::PagingOptions`) must contain a cursor pointing to the Index from which a new page can be requested
    pub fn store_query(
        &self,
        query: &StoreQuery,
        peer_id: &PeerId,
        timeout: Option<Duration>,
    ) -> Result<StoreResponse> {
        store::waku_store_query(query, peer_id, timeout)
    }

    /// Retrieves locally stored historical messages on specific content topics. This method may be called with [`PagingOptions`](`crate::general::PagingOptions`),
    /// to retrieve historical messages on a per-page basis. If the request included [`PagingOptions`](`crate::general::PagingOptions`),
    /// the node must return messages on a per-page basis and include [`PagingOptions`](`crate::general::PagingOptions`) in the response.
    /// These [`PagingOptions`](`crate::general::PagingOptions`) must contain a cursor pointing to the Index from which a new page can be requested
    pub fn local_store_query(&self, query: &StoreQuery) -> Result<StoreResponse> {
        store::waku_local_store_query(query)
    }

    /// Publish a message using Waku Lightpush
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publishchar-messagejson-char-topic-char-peerid-int-timeoutms)
    pub fn lightpush_publish(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        peer_id: PeerId,
        timeout: Option<Duration>,
    ) -> Result<MessageId> {
        lightpush::waku_lightpush_publish(message, pubsub_topic, peer_id, timeout)
    }

    /// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Lightpush
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publish_enc_asymmetricchar-messagejson-char-pubsubtopic-char-peerid-char-publickey-char-optionalsigningkey-int-timeoutms)
    pub fn lightpush_publish_encrypt_asymmetric(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        peer_id: PeerId,
        public_key: &PublicKey,
        signing_key: Option<&SecretKey>,
        timeout: Option<Duration>,
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
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publish_enc_symmetricchar-messagejson-char-pubsubtopic-char-peerid-char-symmetrickey-char-optionalsigningkey-int-timeoutms)
    pub fn lightpush_publish_encrypt_symmetric(
        &self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        peer_id: PeerId,
        symmetric_key: &Key<Aes256Gcm>,
        signing_key: Option<&SecretKey>,
        timeout: Option<Duration>,
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
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_subscribechar-filterjson-char-peerid-int-timeoutms)
    pub fn filter_subscribe(
        &self,
        filter_subscription: &FilterSubscription,
        peer_id: PeerId,
        timeout: Duration,
    ) -> Result<()> {
        filter::waku_filter_subscribe(filter_subscription, peer_id, timeout)
    }

    /// Removes subscriptions in a light node matching a content filter and, optionally, a [`WakuPubSubTopic`](`crate::general::WakuPubSubTopic`)
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_filter_unsubscribechar-filterjson-int-timeoutms)
    pub fn filter_unsubscribe(
        &self,
        filter_subscription: &FilterSubscription,
        timeout: Duration,
    ) -> Result<()> {
        filter::waku_filter_unsubscribe(filter_subscription, timeout)
    }
}

/// Spawn a new Waku node with the given configuration (default configuration if `None` provided)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
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
    use serial_test::serial;

    #[test]
    #[serial]
    fn exclusive_running() {
        let handle1 = waku_new(None).unwrap();
        let handle2 = waku_new(None);
        assert!(handle2.is_err());
        let stop_handle = handle1.start().unwrap();
        stop_handle.stop().unwrap();
    }
}
