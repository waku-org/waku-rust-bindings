//! Reliable channels: an ordered, gap-detecting message stream over a content topic.

// std
use std::ffi::CString;
// crates
use base64::Engine;
use serde::Serialize;
// internal
use crate::general::libwaku_response::{handle_no_response, handle_response, LibwakuResponse};
use crate::general::Result;
use crate::handle_ffi_call;
use crate::node::context::WakuNodeContext;

/// Body of `logosdelivery_channel_send`, whose payload travels base64-encoded.
#[derive(Serialize)]
struct ChannelMessage {
    payload: String,
    ephemeral: bool,
}

/// Create (or restore) a reliable channel, returning its id.
///
/// Channel state is persisted, so re-creating a previously closed channel
/// resumes it rather than starting a new one.
pub async fn logosdelivery_channel_create(
    ctx: &WakuNodeContext,
    channel_id: &str,
    content_topic: &str,
    sender_id: &str,
) -> Result<String> {
    let channel_id =
        CString::new(channel_id).expect("CString should build properly from the channel id");
    let content_topic =
        CString::new(content_topic).expect("CString should build properly from the content topic");
    let sender_id =
        CString::new(sender_id).expect("CString should build properly from the sender id");

    handle_ffi_call!(
        waku_sys::logosdelivery_channel_create,
        handle_response,
        ctx.get_ptr(),
        channel_id.as_ptr(),
        content_topic.as_ptr(),
        sender_id.as_ptr()
    )
}

/// Send a message over a reliable channel, returning the id of the send request.
///
/// That id is what correlates this call with the `onChannelMessageSent` /
/// `onChannelMessageError` events reporting the message's fate.
pub async fn logosdelivery_channel_send(
    ctx: &WakuNodeContext,
    channel_id: &str,
    payload: &[u8],
    ephemeral: bool,
) -> Result<String> {
    let channel_id =
        CString::new(channel_id).expect("CString should build properly from the channel id");

    let message = ChannelMessage {
        payload: base64::engine::general_purpose::STANDARD.encode(payload),
        ephemeral,
    };
    let message = CString::new(
        serde_json::to_string(&message)
            .expect("ChannelMessage should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized channel message");

    handle_ffi_call!(
        waku_sys::logosdelivery_channel_send,
        handle_response,
        ctx.get_ptr(),
        channel_id.as_ptr(),
        message.as_ptr()
    )
}

/// Close a reliable channel, stopping its loops.
///
/// Persisted state survives, so the channel can be restored by re-creating it.
pub async fn logosdelivery_channel_close(ctx: &WakuNodeContext, channel_id: &str) -> Result<()> {
    let channel_id =
        CString::new(channel_id).expect("CString should build properly from the channel id");

    handle_ffi_call!(
        waku_sys::logosdelivery_channel_close,
        handle_no_response,
        ctx.get_ptr(),
        channel_id.as_ptr()
    )
}
