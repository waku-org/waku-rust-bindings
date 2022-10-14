//! Waku [relay](https://rfc.vac.dev/spec/36/#waku-relay) protocol related methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates
use aes_gcm::{Aes256Gcm, Key};
use libsecp256k1::{PublicKey, SecretKey};
// internal
use crate::general::{
    Encoding, JsonResponse, MessageId, Result, WakuContentTopic, WakuMessage, WakuPubSubTopic,
};

/// Create a content topic according to [RFC 23](https://rfc.vac.dev/spec/23/)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_content_topicchar-applicationname-unsigned-int-applicationversion-char-contenttopicname-char-encoding)
pub fn waku_create_content_topic(
    application_name: &str,
    application_version: usize,
    content_topic_name: &str,
    encoding: Encoding,
) -> WakuContentTopic {
    unsafe {
        CStr::from_ptr(waku_sys::waku_content_topic(
            CString::new(application_name)
                .expect("Application name should always transform to CString")
                .into_raw(),
            application_version
                .try_into()
                .expect("Version should fit within an u32"),
            CString::new(content_topic_name)
                .expect("Conmtent topic should always transform to CString")
                .into_raw(),
            CString::new(encoding.to_string())
                .expect("Encoding should always transform to CString")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("&str from result should always be extracted")
    .parse()
    .expect("Content topic data should be always parseable")
}

/// Create a pubsub topic according to [RFC 23](https://rfc.vac.dev/spec/23/)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_pubsub_topicchar-name-char-encoding)
pub fn waku_create_pubsub_topic(topic_name: &str, encoding: Encoding) -> WakuPubSubTopic {
    unsafe {
        CStr::from_ptr(waku_sys::waku_pubsub_topic(
            CString::new(topic_name)
                .expect("Topic name should always transform to CString")
                .into_raw(),
            CString::new(encoding.to_string())
                .expect("Encoding should always transform to CString")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("&str from result should always be extracted")
    .parse()
    .expect("Pubsub topic data should be always parseable")
}

/// Default pubsub topic used for exchanging waku messages defined in [RFC 10](https://rfc.vac.dev/spec/10/)
pub fn waku_dafault_pubsub_topic() -> WakuPubSubTopic {
    unsafe { CStr::from_ptr(waku_sys::waku_default_pubsub_topic()) }
        .to_str()
        .expect("&str from result should always be extracted")
        .parse()
        .expect("Default pubsub topic should always be parseable")
}

/// Publish a message using Waku Relay
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publishchar-messagejson-char-pubsubtopic-int-timeoutms)
pub fn waku_relay_publish_message(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_relay_publish(
            CString::new(
                serde_json::to_string(&message)
                    .expect("WakuMessages should always be able to success serializing"),
            )
            .expect("CString should build properly from the serialized waku message")
            .into_raw(),
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
            timeout
                .map(|duration| {
                    duration
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        ))
    }
    .to_str()
    .expect("&str from result should always be extracted");
    let message_id: JsonResponse<MessageId> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    message_id.into()
}

/// Optionally sign, encrypt using asymmetric encryption and publish a message using Waku Relay
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publish_enc_asymmetricchar-messagejson-char-pubsubtopic-char-publickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_relay_publish_encrypt_asymmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    public_key: &PublicKey,
    signing_key: Option<&SecretKey>,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let pk = hex::encode(public_key.serialize());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.serialize()))
        .unwrap_or_else(String::new);
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_relay_publish_enc_asymmetric(
            CString::new(
                serde_json::to_string(&message)
                    .expect("WakuMessages should always be able to success serializing"),
            )
            .expect("CString should build properly from the serialized waku message")
            .into_raw(),
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
            CString::new(pk)
                .expect("CString should build properly from hex encoded public key")
                .into_raw(),
            CString::new(sk)
                .expect("CString should build properly from hex encoded signing key")
                .into_raw(),
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        ))
        .to_str()
        .expect("Response should always succeed to load to a &str")
    };
    let message_id: JsonResponse<MessageId> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    message_id.into()
}

/// Optionally sign, encrypt using symmetric encryption and publish a message using Waku Relay
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publish_enc_symmetricchar-messagejson-char-pubsubtopic-char-symmetrickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_relay_publish_encrypt_symmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
    symmetric_key: &PublicKey,
    signing_key: Option<&SecretKey>,
    timeout: Option<Duration>,
) -> Result<MessageId> {
    let symk = hex::encode(symmetric_key.serialize());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.serialize()))
        .unwrap_or_else(String::new);
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_relay_publish_enc_symmetric(
            CString::new(
                serde_json::to_string(&message)
                    .expect("WakuMessages should always be able to success serializing"),
            )
            .expect("CString should build properly from the serialized waku message")
            .into_raw(),
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
            CString::new(symk)
                .expect("CString should build properly from hex encoded symmetric key")
                .into_raw(),
            CString::new(sk)
                .expect("CString should build properly from hex encoded signing key")
                .into_raw(),
            timeout
                .map(|timeout| {
                    timeout
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        ))
        .to_str()
        .expect("Response should always succeed to load to a &str")
    };
    let message_id: JsonResponse<MessageId> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    message_id.into()
}

pub fn waku_enough_peers(pubsub_topic: Option<WakuPubSubTopic>) -> Result<bool> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_relay_enough_peers(
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");
    let enough_peers: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    enough_peers.into()
}

pub fn waku_relay_subscribe(pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_relay_subscribe(
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");
    let enough_peers: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    Result::from(enough_peers).map(|_| ())
}

pub fn waku_relay_unsubscribe(pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();
    let result = unsafe {
        CStr::from_ptr(waku_sys::waku_relay_unsubscribe(
            CString::new(pubsub_topic)
                .expect("CString should build properly from pubsub topic")
                .into_raw(),
        ))
    }
    .to_str()
    .expect("Response should always succeed to load to a &str");
    let enough_peers: JsonResponse<bool> =
        serde_json::from_str(result).expect("JsonResponse should always succeed to deserialize");
    Result::from(enough_peers).map(|_| ())
}
