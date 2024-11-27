//! Waku node implementation

mod config;
mod context;
mod events;
mod filter;
mod lightpush;
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
use crate::general::contenttopic::{Encoding, WakuContentTopic};
pub use crate::general::pubsubtopic::PubsubTopic;
use crate::general::{MessageHash, Result, WakuMessage};
use crate::utils::LibwakuResponse;

use crate::node::context::WakuNodeContext;
pub use config::RLNConfig;
pub use config::WakuNodeConfig;
pub use events::{Event, WakuMessageEvent};
pub use relay::waku_create_content_topic;

use std::time::SystemTime;

// Define state marker types
pub struct Initialized;
pub struct Running;

/// Handle to the underliying waku node
pub struct WakuNodeHandle<State> {
    ctx: WakuNodeContext,
    _state: PhantomData<State>,
}

/// Spawn a new Waku node with the given configuration (default configuration if `None` provided)
/// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_newchar-jsonconfig)
pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle<Initialized>> {
    Ok(WakuNodeHandle {
        ctx: management::waku_new(config)?,
        _state: PhantomData,
    })
}

impl<State> WakuNodeHandle<State> {
    /// Get the nwaku version
    pub fn version(&self) -> Result<String> {
        management::waku_version(&self.ctx)
    }

    pub fn waku_destroy(self) -> Result<()> {
        let res = management::waku_destroy(&self.ctx);
        self.ctx.reset_ptr();
        res
    }
}

impl WakuNodeHandle<Initialized> {
    /// Start a Waku node mounting all the protocols that were enabled during the Waku node instantiation.
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_start)
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        management::waku_start(&self.ctx).map(|_| WakuNodeHandle {
            ctx: self.ctx,
            _state: PhantomData,
        })
    }

    pub fn set_event_callback<F: FnMut(LibwakuResponse) + 'static + Sync + Send>(
        &self,
        closure: F,
    ) -> Result<()> {
        self.ctx.waku_set_event_callback(closure)
    }
}

impl WakuNodeHandle<Running> {
    /// Stops a Waku node
    /// as per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_stop)
    pub fn stop(self) -> Result<WakuNodeHandle<Initialized>> {
        management::waku_stop(&self.ctx).map(|_| WakuNodeHandle {
            ctx: self.ctx,
            _state: PhantomData,
        })
    }

    /// Get the multiaddresses the Waku node is listening to
    /// as per [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_listen_addresses)
    pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
        management::waku_listen_addresses(&self.ctx)
    }

    /// Dial peer using a multiaddress
    /// If `timeout` as milliseconds doesn't fit into a `i32` it is clamped to [`i32::MAX`]
    /// If the function execution takes longer than `timeout` value, the execution will be canceled and an error returned.
    /// Use 0 for no timeout
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_connect_peerchar-address-int-timeoutms)
    pub fn connect(&self, address: &Multiaddr, timeout: Option<Duration>) -> Result<()> {
        peers::waku_connect(&self.ctx, address, timeout)
    }

    pub fn relay_publish_txt(
        &self,
        pubsub_topic: &PubsubTopic,
        msg_txt: &String,
        content_topic_name: &'static str,
        timeout: Option<Duration>,
    ) -> Result<MessageHash> {
        let content_topic = WakuContentTopic::new("waku", "2", content_topic_name, Encoding::Proto);
        let message = WakuMessage::new(
            msg_txt,
            content_topic,
            0,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .unwrap(),
            Vec::new(),
            false,
        );

        relay::waku_relay_publish_message(&self.ctx, &message, pubsub_topic, timeout)
    }

    /// Publish a message using Waku Relay.
    /// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
    /// The pubsub_topic parameter is optional and if not specified it will be derived from the contentTopic.
    pub fn relay_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: &PubsubTopic,
        timeout: Option<Duration>,
    ) -> Result<MessageHash> {
        relay::waku_relay_publish_message(&self.ctx, message, pubsub_topic, timeout)
    }

    /// Subscribe to WakuRelay to receive messages matching a content filter.
    pub fn relay_subscribe(&self, pubsub_topic: &PubsubTopic) -> Result<()> {
        relay::waku_relay_subscribe(&self.ctx, pubsub_topic)
    }

    /// Closes the pubsub subscription to stop receiving messages matching a content filter. No more messages will be received from this pubsub topic
    pub fn relay_unsubscribe(&self, pubsub_topic: &PubsubTopic) -> Result<()> {
        relay::waku_relay_unsubscribe(&self.ctx, pubsub_topic)
    }

    pub fn filter_subscribe(
        &self,
        pubsub_topic: &PubsubTopic,
        content_topics: Vec<WakuContentTopic>,
    ) -> Result<()> {
        filter::waku_filter_subscribe(&self.ctx, pubsub_topic, content_topics)
    }

    pub fn filter_unsubscribe(
        &self,
        pubsub_topic: &PubsubTopic,
        content_topics: Vec<WakuContentTopic>,
    ) -> Result<()> {
        filter::waku_filter_unsubscribe(&self.ctx, pubsub_topic, content_topics)
    }

    pub fn filter_unsubscribe_all(&self) -> Result<()> {
        filter::waku_filter_unsubscribe_all(&self.ctx)
    }

    pub fn lightpush_publish_message(
        &self,
        message: &WakuMessage,
        pubsub_topic: &PubsubTopic,
    ) -> Result<MessageHash> {
        lightpush::waku_lightpush_publish_message(&self.ctx, message, pubsub_topic)
    }
}
