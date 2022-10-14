//! Waku [lightpush](https://rfc.vac.dev/spec/36/#waku-lightpush) protocol related methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates
use aes_gcm::{Aes256Gcm, Key};
use secp256k1::{PublicKey, SecretKey};
// internal
use crate::general::{JsonResponse, MessageId, PeerId, Result, WakuMessage, WakuPubSubTopic};
use crate::node::waku_dafault_pubsub_topic;

/// Publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publishchar-messagejson-char-topic-char-peerid-int-timeoutms)
pub fn waku_lightpush_publish(
    message: &WakuMessage,
    pubsub_topic: WakuPubSubTopic,
    peer_id: PeerId,
    timeout: Duration,
) -> Result<MessageId> {
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_lightpush_publish(
            CString::new(
                serde_json::to_string(&message)
                    .expect("WakuMessages should always be able to success serializing"),
            )
            .expect("CString should build properly from the serialized waku message")
            .into_raw(),
            CString::new(pubsub_topic.to_string())
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
            CString::new(peer_id)
                .expect("CString should build properly from peer id")
                .into_raw(),
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");

    let response: JsonResponse<MessageId> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");

    response.into()
}

/// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publish_enc_asymmetricchar-messagejson-char-pubsubtopic-char-peerid-char-publickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_lightpush_publish_encrypt_asymmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    peer_id: PeerId,
    public_key: &PublicKey,
    signing_key: Option<&SecretKey>,
    timeout: Duration,
) -> Result<MessageId> {
    let pk = hex::encode(public_key.serialize());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.secret_bytes()))
        .unwrap_or_else(String::new);
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_lightpush_publish_enc_asymmetric(
            CString::new(
                serde_json::to_string(&message)
                    .expect("WakuMessages should always be able to success serializing"),
            )
            .expect("CString should build properly from the serialized waku message")
            .into_raw(),
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
            CString::new(peer_id)
                .expect("CString should build properly from peer id")
                .into_raw(),
            CString::new(pk)
                .expect("CString should build properly from hex encoded public key")
                .into_raw(),
            CString::new(sk)
                .expect("CString should build properly from hex encoded signing key")
                .into_raw(),
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        ))
        .to_str()
        .expect("Response should always succeed to load to a &str")
    };
    let message_id: JsonResponse<MessageId> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    message_id.into()
}

/// Optionally sign, encrypt using symmetric encryption and publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publish_enc_symmetricchar-messagejson-char-pubsubtopic-char-peerid-char-symmetrickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_lightpush_publish_encrypt_symmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    peer_id: PeerId,
    symmetric_key: &Key<Aes256Gcm>,
    signing_key: Option<&SecretKey>,
    timeout: Duration,
) -> Result<MessageId> {
    let symk = hex::encode(symmetric_key.as_slice());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.secret_bytes()))
        .unwrap_or_else(String::new);
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_lightpush_publish_enc_symmetric(
            CString::new(
                serde_json::to_string(&message)
                    .expect("WakuMessages should always be able to success serializing"),
            )
            .expect("CString should build properly from the serialized waku message")
            .into_raw(),
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
            CString::new(peer_id)
                .expect("CString should build properly from peer id")
                .into_raw(),
            CString::new(symk)
                .expect("CString should build properly from hex encoded symmetric key")
                .into_raw(),
            CString::new(sk)
                .expect("CString should build properly from hex encoded signing key")
                .into_raw(),
            timeout
                .as_millis()
                .try_into()
                .expect("Duration as milliseconds should fit in a i32"),
        ))
        .to_str()
        .expect("Response should always succeed to load to a &str")
    };
    let message_id: JsonResponse<MessageId> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    message_id.into()
}
