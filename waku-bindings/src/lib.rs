//! # waku-bindings
//!
//! Rust bindings for logos-delivery (Waku). A node is driven through
//! [`LogosDeliveryCtx`], generated from the Nim source by nim-ffi and
//! re-exported here; this crate adds the typed domain layer ([`WakuMessage`],
//! [`WakuContentTopic`], [`MessageHash`]) and [`WakuNodeConfig`], which
//! serialises to the JSON [`LogosDeliveryCtx::create`] expects.
//!
//! ## Operations
//!
//! Every operation is an `async` method on [`LogosDeliveryCtx`] (each also has a
//! blocking twin without the `_async` suffix). Events are registered with
//! `add_on_*_listener`, which hand back a typed payload.
//!
//! ### Node lifecycle
//! - [`LogosDeliveryCtx::new_async`] / [`create`](LogosDeliveryCtx::create) —
//!   build a node from a [`WakuNodeConfig`] JSON string
//! - [`start_node_async`](LogosDeliveryCtx::start_node_async) /
//!   [`stop_node_async`](LogosDeliveryCtx::stop_node_async)
//! - teardown is the `Drop` impl — there is no explicit destroy
//!
//! ### Messaging
//! - [`subscribe_async`](LogosDeliveryCtx::subscribe_async) /
//!   [`unsubscribe_async`](LogosDeliveryCtx::unsubscribe_async) — by content topic
//! - [`send_async`](LogosDeliveryCtx::send_async) — takes a [`SendRequest`]
//!
//! ### Reliable channels
//! - [`channel_create_async`](LogosDeliveryCtx::channel_create_async) —
//!   `(channel_id, content_topic, sender_id, encryption)`; `encryption` is
//!   `"noop"` today
//! - [`channel_send_async`](LogosDeliveryCtx::channel_send_async) — takes a
//!   [`ChannelSendRequest`]
//! - [`channel_close_async`](LogosDeliveryCtx::channel_close_async)
//!
//! ### Events (typed listeners)
//! - messaging: `add_on_message_{sent,error,propagated,received}_listener`
//! - connectivity: `add_on_connection_{change,status_change}_listener`,
//!   `add_on_topic_health_change_listener`
//! - relay: `add_on_received_message_listener`
//! - channels: `add_on_channel_message_{received,sent,error}_listener`
//! - drop one with [`remove_event_listener`](LogosDeliveryCtx::remove_event_listener)
//!
//! Beneath these sits the lower-level `waku_*` kernel surface (relay publish,
//! filter, store, peer management, discovery) — also on [`LogosDeliveryCtx`],
//! `"use at your own risk"`.
//!
//! ## Example
//!
//! ```no_run
//! use waku_bindings::{LogosDeliveryCtx, WakuNodeConfig};
//! use std::time::Duration;
//!
//! # async fn run() -> Result<(), String> {
//! let config = serde_json::to_string(&WakuNodeConfig {
//!     tcp_port: Some(60000),
//!     ..Default::default()
//! }).unwrap();
//!
//! let node = LogosDeliveryCtx::new_async(config, Duration::from_secs(30)).await?;
//! node.add_on_received_message_listener(|event| {
//!     println!("received on {}", event.waku_message.content_topic);
//! });
//! node.start_node_async().await?;
//! node.waku_relay_subscribe_async("test".to_string()).await?;
//! // node is torn down when dropped
//! # Ok(())
//! # }
//! ```

pub mod general;
pub mod node;

// The full generated FFI surface: LogosDeliveryCtx and its methods, the request
// types (SendRequest, ChannelSendRequest), and one payload type per event. See
// the Operations section above for the map; browse `LogosDeliveryCtx` in the
// docs for the complete method list.
pub use waku_sys::*;

// Required so functions inside libwaku can call RLN functions even if we
// use it within the bindings functions
#[allow(clippy::single_component_path_imports)]
#[allow(unused)]
use rln;

// The typed domain layer this crate adds on top of the generated surface.
pub use general::contenttopic::{Encoding, WakuContentTopic};
pub use general::store::{StoreQueryRequest, StoreResponse, StoreWakuMessageResponse};
pub use general::{messagehash::MessageHash, Result, WakuMessage, WakuMessageVersion};
pub use node::{
    Key, Multiaddr, PublicKey, PubsubTopic, RLNConfig, SecretKey, WakuNodeConfig,
};
