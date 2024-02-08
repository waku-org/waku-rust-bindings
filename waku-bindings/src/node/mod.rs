//! Waku node implementation

mod config;
mod management;
mod peers;
mod relay;

// std
pub use aes_gcm::{Aes256Gcm, Key};
pub use multiaddr::Multiaddr;
pub use secp256k1::{PublicKey, SecretKey};
use std::marker::PhantomData;
use std::sync::Mutex;
use std::time::Duration;
// crates
// internal

use crate::general::{MessageId, Result, WakuMessage, WakuPubSubTopic};

pub use config::WakuNodeConfig;
pub use relay::{waku_create_content_topic, waku_default_pubsub_topic};

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
    //     /// If the execution is successful, the result is the peer ID as a string (base58 encoded)
    //     /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    //     pub fn peer_id(&self) -> Result<PeerId> {
    //         management::waku_peer_id()
    //     }

    //     /// Get the multiaddresses the Waku node is listening to
    //     /// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
    //     pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
    //         management::waku_listen_addresses()
    //     }

    //     /// Add a node multiaddress and protocol to the waku node’s peerstore.
    //     /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_add_peerchar-address-char-protocolid)
    //     pub fn add_peer(&self, address: &Multiaddr, protocol_id: ProtocolId) -> Result<PeerId> {
    //         peers::waku_add_peers(address, protocol_id)
    //     }
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
    pub fn connect(&self, address: &Multiaddr, timeout: Option<Duration>) -> Result<()> {
        peers::waku_connect(address, timeout)
    }

    /// Publish a message using Waku Relay.
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
    /// The pubsub_topic parameter is optional and if not specified it will be derived from the contentTopic.
    pub fn relay_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: &WakuPubSubTopic,
        timeout: Option<Duration>,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_message(message, pubsub_topic, timeout)
    }

    /// Subscribe to WakuRelay to receive messages matching a content filter.
    pub fn relay_subscribe(&self, pubsub_topic: &WakuPubSubTopic) -> Result<()> {
        relay::waku_relay_subscribe(pubsub_topic)
    }

    /// Closes the pubsub subscription to stop receiving messages matching a content filter. No more messages will be received from this pubsub topic
    pub fn relay_unsubscribe(&self, pubsub_topic: &WakuPubSubTopic) -> Result<()> {
        relay::waku_relay_unsubscribe(pubsub_topic)
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
