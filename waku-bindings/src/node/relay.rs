//! Waku [relay](https://rfc.vac.dev/spec/36/#waku-relay) protocol related methods

// std
use std::ffi::{CStr, CString};
use std::time::Duration;
// crates
use aes_gcm::{Aes256Gcm, Key};
use secp256k1::{PublicKey, SecretKey};
// internal
use crate::general::{Encoding, MessageId, Result, WakuContentTopic, WakuMessage, WakuPubSubTopic};
use crate::utils::decode_and_free_response;

/// Create a content topic according to [RFC 23](https://rfc.vac.dev/spec/23/)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_content_topicchar-applicationname-unsigned-int-applicationversion-char-contenttopicname-char-encoding)
pub fn waku_create_content_topic(
    application_name: &str,
    application_version: usize,
    content_topic_name: &str,
    encoding: Encoding,
) -> WakuContentTopic {
    let application_name_ptr = CString::new(application_name)
        .expect("Application name should always transform to CString")
        .into_raw();
    let application_version = application_version
        .try_into()
        .expect("Version should fit within an u32");
    let content_topic_name_ptr = CString::new(content_topic_name)
        .expect("Content topic should always transform to CString")
        .into_raw();
    let encoding_ptr = CString::new(encoding.to_string())
        .expect("Encoding should always transform to CString")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_content_topic(
            application_name_ptr,
            application_version,
            content_topic_name_ptr,
            encoding_ptr,
        );
        drop(CString::from_raw(application_name_ptr));
        drop(CString::from_raw(content_topic_name_ptr));
        drop(CString::from_raw(encoding_ptr));
        res
    };
    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_str()
        .expect("&str from result should always be extracted")
        .parse()
        .expect("Content topic data should be always parseable");

    unsafe { waku_sys::waku_utils_free(result_ptr) };

    result
}

/// Create a pubsub topic according to [RFC 23](https://rfc.vac.dev/spec/23/)
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_pubsub_topicchar-name-char-encoding)
pub fn waku_create_pubsub_topic(topic_name: &str, encoding: Encoding) -> WakuPubSubTopic {
    let topic_name_ptr = CString::new(topic_name)
        .expect("Topic name should always transform to CString")
        .into_raw();
    let encoding_ptr = CString::new(encoding.to_string())
        .expect("Encoding should always transform to CString")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_pubsub_topic(topic_name_ptr, encoding_ptr);
        drop(CString::from_raw(topic_name_ptr));
        drop(CString::from_raw(encoding_ptr));
        res
    };

    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_str()
        .expect("&str from result should always be extracted")
        .parse()
        .expect("Pubsub topic data should be always parseable");

    unsafe { waku_sys::waku_utils_free(result_ptr) };

    result
}

/// Default pubsub topic used for exchanging waku messages defined in [RFC 10](https://rfc.vac.dev/spec/10/)
pub fn waku_dafault_pubsub_topic() -> WakuPubSubTopic {
    let result_ptr = unsafe { waku_sys::waku_default_pubsub_topic() };

    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_str()
        .expect("&str from result should always be extracted")
        .parse()
        .expect("Default pubsub topic should always be parseable");

    unsafe { waku_sys::waku_utils_free(result_ptr) };

    result
}

/// Get the list of subscribed pubsub topics in Waku Relay.
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_topics)
pub fn waku_relay_topics() -> Result<Vec<String>> {
    let result_ptr = unsafe { waku_sys::waku_relay_topics() };
    decode_and_free_response(result_ptr)
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

    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_relay_publish(
            message_ptr,
            pubsub_topic_ptr,
            timeout
                .map(|duration| {
                    duration
                        .as_millis()
                        .try_into()
                        .expect("Duration as milliseconds should fit in a i32")
                })
                .unwrap_or(0),
        );
        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(pubsub_topic_ptr));
        res
    };

    decode_and_free_response(result_ptr)
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
    let pk = hex::encode(public_key.serialize_uncompressed());
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
    let pk_ptr = CString::new(pk)
        .expect("CString should build properly from hex encoded public key")
        .into_raw();
    let sk_ptr = CString::new(sk)
        .expect("CString should build properly from hex encoded signing key")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_relay_publish_enc_asymmetric(
            message_ptr,
            pubsub_topic_ptr,
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
        drop(CString::from_raw(pk_ptr));
        drop(CString::from_raw(sk_ptr));
        res
    };

    decode_and_free_response(result_ptr)
}

/// Optionally sign, encrypt using symmetric encryption and publish a message using Waku Relay
/// As per the [specification](https://rfc.vac.dev/spec/36/#extern-char-waku_relay_publish_enc_symmetricchar-messagejson-char-pubsubtopic-char-symmetrickey-char-optionalsigningkey-int-timeoutms)
pub fn waku_relay_publish_encrypt_symmetric(
    message: &WakuMessage,
    pubsub_topic: Option<WakuPubSubTopic>,
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
    let symk_ptr = CString::new(symk)
        .expect("CString should build properly from hex encoded symmetric key")
        .into_raw();
    let sk_ptr = CString::new(sk)
        .expect("CString should build properly from hex encoded signing key")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_relay_publish_enc_symmetric(
            message_ptr,
            pubsub_topic_ptr,
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
        drop(CString::from_raw(symk_ptr));
        drop(CString::from_raw(sk_ptr));
        res
    };

    decode_and_free_response(result_ptr)
}

pub fn waku_enough_peers(pubsub_topic: Option<WakuPubSubTopic>) -> Result<bool> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();

    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_relay_enough_peers(pubsub_topic_ptr);
        drop(CString::from_raw(pubsub_topic_ptr));
        res
    };

    decode_and_free_response(result_ptr)
}

pub fn waku_relay_subscribe(pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();

    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_relay_subscribe(pubsub_topic_ptr);
        drop(CString::from_raw(pubsub_topic_ptr));
        res
    };

    decode_and_free_response::<bool>(result_ptr).map(|_| ())
}

pub fn waku_relay_unsubscribe(pubsub_topic: Option<WakuPubSubTopic>) -> Result<()> {
    let pubsub_topic = pubsub_topic
        .unwrap_or_else(waku_dafault_pubsub_topic)
        .to_string();

    let pubsub_topic_ptr = CString::new(pubsub_topic)
        .expect("CString should build properly from pubsub topic")
        .into_raw();

    let result_ptr = unsafe {
        let res = waku_sys::waku_relay_unsubscribe(pubsub_topic_ptr);
        drop(CString::from_raw(pubsub_topic_ptr));
        res
    };

    decode_and_free_response::<bool>(result_ptr).map(|_| ())
}
