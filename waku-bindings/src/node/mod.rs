//! Waku node implementation

mod config;
mod events;
mod management;
mod peers;
mod relay;

// std
pub use aes_gcm::{Aes256Gcm, Key};
pub use multiaddr::Multiaddr;
pub use secp256k1::{PublicKey, SecretKey};
use std::time::Duration;
// crates
use libc::c_void;
// internal

use crate::general::{MessageId, Result, WakuMessage};

pub use config::WakuNodeConfig;
pub use events::{Event, WakuMessageEvent};
pub use relay::{waku_create_content_topic, waku_default_pubsub_topic};

/// Handle to the underliying waku node
pub struct WakuNodeHandle {
    ctx: *mut c_void,
}

impl WakuNodeHandle {
    /// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
    pub fn start(&self) -> Result<()> {
        management::waku_start(self.ctx)
    }

    /// Stops a Waku node
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn stop(&self) -> Result<()> {
        management::waku_stop(self.ctx)
    }

    /// Dial peer using a multiaddress
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
    /// Use 0 for no timeout
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
    pub fn connect(&self, address: &Multiaddr, timeout: Option<Duration>) -> Result<()> {
        peers::waku_connect(self.ctx, address, timeout)
    }

    /// Publish a message using Waku Relay.
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
    /// The pubsub_topic parameter is optional and if not specified it will be derived from the contentTopic.
    pub fn relay_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: &String,
        timeout: Option<Duration>,
    ) -> Result<MessageId> {
        relay::waku_relay_publish_message(self.ctx, message, pubsub_topic, timeout)
    }

    /// Subscribe to WakuRelay to receive messages matching a content filter.
    pub fn relay_subscribe(&self, pubsub_topic: &String) -> Result<()> {
        relay::waku_relay_subscribe(self.ctx, pubsub_topic)
    }

    /// Closes the pubsub subscription to stop receiving messages matching a content filter. No more messages will be received from this pubsub topic
    pub fn relay_unsubscribe(&self, pubsub_topic: &String) -> Result<()> {
        relay::waku_relay_unsubscribe(self.ctx, pubsub_topic)
    }

    pub fn set_event_callback<F: FnMut(Event) + Send + Sync + 'static>(&self, f: F) {
        events::waku_set_event_callback(self.ctx, f)
    }
}

/// Spawn a new Waku node with the given configuration (default configuration if `None` provided)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle> {
    Ok(WakuNodeHandle {
        ctx: management::waku_new(config)?,
    })
}
