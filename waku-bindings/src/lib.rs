//! # Waku
//!
//! The FFI surface is generated from the Nim source by nim-ffi and re-exported
//! here wholesale: drive a node through `LogosDeliveryCtx`, and observe it
//! through its typed `add_on_*_listener` methods. What this crate adds on top is
//! the domain layer — [`WakuMessage`], [`WakuContentTopic`], [`MessageHash`] and
//! friends — plus [`WakuNodeConfig`], which serialises to the JSON that
//! `LogosDeliveryCtx::create` expects.
pub mod general;
pub mod node;

// The generated bindings: LogosDeliveryCtx, ListenerHandle, and one payload type
// per event.
pub use waku_sys::*;

// Required so functions inside libwaku can call RLN functions even if we
// use it within the bindings functions
#[allow(clippy::single_component_path_imports)]
#[allow(unused)]
use rln;

pub use general::contenttopic::{Encoding, WakuContentTopic};
pub use general::store::{StoreQueryRequest, StoreResponse, StoreWakuMessageResponse};
pub use general::{messagehash::MessageHash, Result, WakuMessage, WakuMessageVersion};
pub use node::{
    Key, Multiaddr, PublicKey, PubsubTopic, RLNConfig, SecretKey, WakuNodeConfig,
};
