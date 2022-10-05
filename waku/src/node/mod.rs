mod config;
mod management;
mod peers;
mod relay;

// std
use aes_gcm::{Aes256Gcm, Key};
use libsecp256k1::{PublicKey, SecretKey};
use multiaddr::Multiaddr;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::time::Duration;
// crates
// internal
use crate::general::{MessageId, PeerId, Result, WakuMessage, WakuPubSubTopic};

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

pub struct WakuNodeHandle<State: WakuNodeState>(PhantomData<State>);

impl<State: WakuNodeState> WakuNodeHandle<State> {
    pub fn peer_id(&self) -> Result<String> {
        management::waku_peer_id()
    }

    pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
        management::waku_listen_addressses()
    }

    pub fn add_peer(&mut self, address: Multiaddr, protocol_id: usize) -> Result<PeerId> {
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
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        management::waku_start().map(|_| WakuNodeHandle(Default::default()))
    }

    pub fn stop(self) -> Result<()> {
        stop_node()
    }
}

impl WakuNodeHandle<Running> {
    pub fn stop(self) -> Result<()> {
        stop_node()
    }

    pub fn connect_peer_with_address(
        &mut self,
        address: Multiaddr,
        timeout: Option<Duration>,
    ) -> Result<()> {
        peers::waku_connect_peer_with_address(address, timeout)
    }

    pub fn connect_peer_with_id(
        &mut self,
        peer_id: PeerId,
        timeout: Option<Duration>,
    ) -> Result<()> {
        peers::waku_connect_peer_with_id(peer_id, timeout)
    }

    pub fn disconnect_peer_with_id(&mut self, peer_id: PeerId) -> Result<()> {
        peers::waku_disconnect_peer_with_id(peer_id)
    }

    pub fn peer_count(&self) -> Result<usize> {
        peers::waku_peer_count()
    }

    pub fn peers(&self) -> Result<WakuPeers> {
        peers::waku_peers()
    }

    pub fn publish_message(
        &mut self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        timeout: Duration,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_message(message, pubsub_topic, timeout)
    }

    pub fn publish_encrypt_asymmetric(
        &mut self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        public_key: &PublicKey,
        signing_key: &SecretKey,
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

    pub fn publish_encrypt_symmetric(
        &mut self,
        message: &WakuMessage,
        pubsub_topic: Option<WakuPubSubTopic>,
        symmetric_key: &Key<Aes256Gcm>,
        signing_key: &SecretKey,
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

    pub fn enough_peers(&self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<bool> {
        relay::waku_enough_peers(pubsub_topic)
    }

    pub fn subscribe(&mut self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
        relay::waku_relay_subscribe(pubsub_topic)
    }

    pub fn unsubscribe(&mut self, pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
        relay::waku_relay_unsubscribe(pubsub_topic)
    }
}

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
