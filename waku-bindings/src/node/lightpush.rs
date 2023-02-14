//! Waku [lightpush](https://rfc.vac.dev/spec/36/#waku-lightpush) protocol related methods

// std
use std::ffi::CString;
use std::time::Duration;
// crates
use aes_gcm::{Aes256Gcm, Key};
use secp256k1::{PublicKey, SecretKey};
// internal
use crate::general::{MessageId, PeerId, Result, WakuMessage, WakuPubSubTopic};
use crate::node::waku_dafault_pubsub_topic;
use crate::utils::decode_and_free_response;

/// Publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publishchar-messagejson-char-topic-char-peerid-int-timeoutms)
pub fn waku_lightpush_publish(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    peer_id: PeerId,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();
    let peer_id_ptr = CString::new(peer_id)
        .expect("CString should build properly from peer id")
        .into_raw();
    let result_ptr = unsafe {
        let res = waku_sys::waku_lightpush_publish(
            message_ptr,
            topic_ptr,
            peer_id_ptr,
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        );
        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(topic_ptr));
        drop(CString::from_raw(peer_id_ptr));
        res
    };

    decode_and_free_response(result_ptr)
}

/// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publish_enc_asymmetricchar-messagejson-char-pubsubtopic-char-peerid-char-publickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_lightpush_publish_encrypt_asymmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    peer_id: PeerId,
    public_key: &PublicKey,
    signing_key: Option<&SecretKey>,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let pk = hex::encode(public_key.serialize_uncompressed());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.secret_bytes()))
        .unwrap_or_else(String::new);
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();

    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();
    let peer_id_ptr = CString::new(peer_id)
        .expect("CString should build properly from peer id")
        .into_raw();
    let pk_ptr = CString::new(pk)
        .expect("CString should build properly from hex encoded public key")
        .into_raw();
    let sk_ptr = CString::new(sk)
        .expect("CString should build properly from hex encoded signing key")
        .into_raw();
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let result_ptr = unsafe {
        let res = waku_sys::waku_lightpush_publish_enc_asymmetric(
            message_ptr,
            pubsub_topic_ptr,
            peer_id_ptr,
            pk_ptr,
            sk_ptr,
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        );
        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(pubsub_topic_ptr));
        drop(CString::from_raw(peer_id_ptr));
        drop(CString::from_raw(pk_ptr));
        drop(CString::from_raw(sk_ptr));
        res
    };

    decode_and_free_response(result_ptr)
}

/// Optionally sign, encrypt using symmetric encryption and publish a message using Waku Lightpush
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_lightpush_publish_enc_symmetricchar-messagejson-char-pubsubtopic-char-peerid-char-symmetrickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_lightpush_publish_encrypt_symmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    peer_id: PeerId,
    symmetric_key: &Key<Aes256Gcm>,
    signing_key: Option<&SecretKey>,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let symk = hex::encode(symmetric_key.as_slice());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.secret_bytes()))
        .unwrap_or_else(String::new);
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();
    let peer_id_ptr = CString::new(peer_id)
        .expect("CString should build properly from peer id")
        .into_raw();
    let symk_ptr = CString::new(symk)
        .expect("CString should build properly from hex encoded symmetric key")
        .into_raw();
    let sk_ptr = CString::new(sk)
        .expect("CString should build properly from hex encoded signing key")
        .into_raw();
    let result_ptr = unsafe {
        let res = waku_sys::waku_lightpush_publish_enc_symmetric(
            message_ptr,
            pubsub_topic_ptr,
            peer_id_ptr,
            symk_ptr,
            sk_ptr,
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        );
        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(pubsub_topic_ptr));
        drop(CString::from_raw(peer_id_ptr));
        drop(CString::from_raw(symk_ptr));
        drop(CString::from_raw(sk_ptr));
        res
    };

    decode_and_free_response(result_ptr)
}
