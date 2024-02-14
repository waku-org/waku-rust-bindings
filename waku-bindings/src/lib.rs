//! # Waku
//!
//! Implementation on top of [`waku-bindings`](https://rfc.vac.dev/spec/36/)
mod general;
mod node;
mod utils;

// Required so functions inside libwaku can call RLN functions even if we
// use it within the bindings functions
#[allow(clippy::single_component_path_imports)]
#[allow(unused)]
use rln;

pub use node::{
    waku_create_content_topic, waku_default_pubsub_topic, waku_new, Event, Key, Multiaddr,
    PublicKey, SecretKey, WakuMessageEvent, WakuNodeConfig, WakuNodeHandle,
};

pub use general::{Encoding, MessageId, Result, WakuContentTopic, WakuMessage, WakuMessageVersion};
