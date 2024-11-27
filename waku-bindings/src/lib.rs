//! # Waku
//!
//! Implementation on top of [`waku-bindings`](https://rfc.vac.dev/spec/36/)
mod general;
pub mod node;
pub mod utils;

// Re-export the LibwakuResponse type to make it accessible outside this module
pub use utils::LibwakuResponse;

// Required so functions inside libwaku can call RLN functions even if we
// use it within the bindings functions
#[allow(clippy::single_component_path_imports)]
#[allow(unused)]
use rln;

pub use node::{
    waku_create_content_topic, waku_new, Event, Initialized, Key, Multiaddr, PublicKey, RLNConfig,
    Running, SecretKey, WakuMessageEvent, WakuNodeConfig, WakuNodeHandle,
};

pub use general::contenttopic::{Encoding, WakuContentTopic};
pub use general::{MessageHash, Result, WakuMessage, WakuMessageVersion};
