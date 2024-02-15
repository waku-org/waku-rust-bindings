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
use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::Lazy;
// crates
use libc::c_void;
// internal

use crate::general::{MessageId, Result, WakuMessage};

pub use config::WakuNodeConfig;
pub use events::{Event, WakuMessageEvent};
pub use relay::{waku_create_content_topic, waku_default_pubsub_topic};

pub struct OpaqueWakunodePtr {
    obj_ptr: *mut c_void,
}

unsafe impl Send for OpaqueWakunodePtr {}

/// Handle to the underliying waku node
pub struct WakuNodeHandle {
    ctx: Arc<Mutex<OpaqueWakunodePtr>>,
    callback: Lazy<Mutex<Box<dyn FnMut(Event) + Send + Sync>>>,
}

impl WakuNodeHandle {
    /// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
    pub fn start(&self) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        management::waku_start(ctx.obj_ptr)
    }

    /// Stops a Waku node
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn stop(&self) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        management::waku_stop(ctx.obj_ptr)
    }

    /// Dial peer using a multiaddress
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
    /// Use 0 for no timeout
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
    pub fn connect(&self, address: &Multiaddr, timeout: Option<Duration>) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        peers::waku_connect(ctx.obj_ptr, address, timeout)
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
        let mut ctx = self.ctx.lock().unwrap();
        relay::waku_relay_publish_message(ctx.obj_ptr, message, pubsub_topic, timeout)
    }

    /// Subscribe to WakuRelay to receive messages matching a content filter.
    pub fn relay_subscribe(&self, pubsub_topic: &String) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        relay::waku_relay_subscribe(ctx.obj_ptr, pubsub_topic)
    }

    /// Closes the pubsub subscription to stop receiving messages matching a content filter. No more messages will be received from this pubsub topic
    pub fn relay_unsubscribe(&self, pubsub_topic: &String) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        relay::waku_relay_unsubscribe(ctx.obj_ptr, pubsub_topic)
    }

    pub fn set_event_callback<F: FnMut(Event) + Send + Sync + 'static>(&self, f: F) {
        let mut ctx = self.ctx.lock().unwrap();
        *self.callback.lock().unwrap() = Box::new(f);
        events::waku_set_event_callback(ctx.obj_ptr, &self.callback)
    }
}

/// Spawn a new Waku node with the given configuration (default configuration if `None` provided)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle> {
    let obj_ptr = management::waku_new(config)?;
    Ok(WakuNodeHandle {
        ctx: Arc::new(Mutex::new(OpaqueWakunodePtr { obj_ptr })),
        callback: Lazy::new(|| Mutex::new(Box::new(|_| {}))),
    })
}
