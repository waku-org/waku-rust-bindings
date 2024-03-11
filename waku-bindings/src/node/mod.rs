//! Waku node implementation

mod config;
mod context;
mod events;
mod management;
mod peers;
mod relay;

// std
pub use aes_gcm::Key;
pub use multiaddr::Multiaddr;
pub use secp256k1::{PublicKey, SecretKey};
use std::marker::PhantomData;
use std::time::Duration;
// internal
use crate::general::{MessageHash, Result, WakuMessage};
use context::WakuNodeContext;

pub use config::WakuNodeConfig;
pub use events::{Event, WakuMessageEvent};
pub use relay::waku_create_content_topic;

/// Marker trait to disallow undesired waku node states in the handle
pub trait WakuNodeState {}

/// Waku node initialized state
pub struct Initialized;

/// Waku node running state
pub struct Running;

impl WakuNodeState for Initialized {}
impl WakuNodeState for Running {}

/// Handle to the underliying waku node
pub struct WakuNodeHandle<State: WakuNodeState> {
    ctx: WakuNodeContext,
    phantom: PhantomData<State>,
}

/// Spawn a new Waku node with the given configuration (default configuration if `None` provided)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle<Initialized>> {
    Ok(WakuNodeHandle {
        ctx: management::waku_new(config)?,
        phantom: PhantomData,
    })
}

pub fn waku_destroy(node: WakuNodeHandle<Initialized>) -> Result<()> {
    management::waku_destroy(&node.ctx)
}

impl WakuNodeHandle<Initialized> {
    /// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        management::waku_start(&self.ctx).map(|_| WakuNodeHandle {
            ctx: self.ctx,
            phantom: PhantomData,
        })
    }
}

impl WakuNodeHandle<Running> {
    /// Stops a Waku node
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn stop(self) -> Result<WakuNodeHandle<Initialized>> {
        management::waku_stop(&self.ctx).map(|_| WakuNodeHandle {
            ctx: self.ctx,
            phantom: PhantomData,
        })
    }

    /// Get the multiaddresses the Waku node is listening to
    /// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
    pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
        management::waku_listen_addresses(&self.ctx)
    }

    /// Get the nwaku version
    pub fn version(&self) -> Result<String> {
        management::waku_version(&self.ctx)
    }

    /// Dial peer using a multiaddress
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
    /// Use 0 for no timeout
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
    pub fn connect(&self, address: &Multiaddr, timeout: Option<Duration>) -> Result<()> {
        peers::waku_connect(&self.ctx, address, timeout)
    }

    /// Publish a message using Waku Relay.
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
    /// The pubsub_topic parameter is optional and if not specified it will be derived from the contentTopic.
    pub fn relay_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: &String,
        timeout: Option<Duration>,
    ) -> Result<MessageHash> {
        relay::waku_relay_publish_message(&self.ctx, message, pubsub_topic, timeout)
    }

    /// Subscribe to WakuRelay to receive messages matching a content filter.
    pub fn relay_subscribe(&self, pubsub_topic: &String) -> Result<()> {
        relay::waku_relay_subscribe(&self.ctx, pubsub_topic)
    }

    /// Closes the pubsub subscription to stop receiving messages matching a content filter. No more messages will be received from this pubsub topic
    pub fn relay_unsubscribe(&self, pubsub_topic: &String) -> Result<()> {
        relay::waku_relay_unsubscribe(&self.ctx, pubsub_topic)
    }

    pub fn set_event_callback<F: FnMut(Event) + Send + Sync + 'static>(&self, f: F) {
        events::waku_set_event_callback(&self.ctx, f)
    }
}
