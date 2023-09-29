// std
use std::ffi::CString;
// crates
use aes_gcm::{Aes256Gcm, Key};
use libc::*;
use secp256k1::{PublicKey, SecretKey};
// internal
use crate::general::{Result, WakuMessage};
use crate::utils::{get_trampoline, handle_json_response};

/// Optionally sign and encrypt a message using asymmetric encryption
pub fn waku_encode_asymmetric(
    message: &WakuMessage,
    public_key: &PublicKey,
    signing_key: Option<&SecretKey>,
) -> Result<WakuMessage> {
    let pk = hex::encode(public_key.serialize_uncompressed());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.secret_bytes()))
        .unwrap_or_default();
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let pk_ptr = CString::new(pk)
        .expect("CString should build properly from hex encoded public key")
        .into_raw();
    let sk_ptr = CString::new(sk)
        .expect("CString should build properly from hex encoded signing key")
        .into_raw();

    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_encode_asymmetric(
            message_ptr,
            pk_ptr,
            sk_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(pk_ptr));
        drop(CString::from_raw(sk_ptr));

        out
    };

    handle_json_response(code, &result)
}

/// Optionally sign and encrypt a message using symmetric encryption
pub fn waku_encode_symmetric(
    message: &WakuMessage,
    symmetric_key: &Key<Aes256Gcm>,
    signing_key: Option<&SecretKey>,
) -> Result<WakuMessage> {
    let symk = hex::encode(symmetric_key.as_slice());
    let sk = signing_key
        .map(|signing_key| hex::encode(signing_key.secret_bytes()))
        .unwrap_or_default();
    let message_ptr = CString::new(
        serde_json::to_string(&message)
            .expect("WakuMessages should always be able to success serializing"),
    )
    .expect("CString should build properly from the serialized waku message")
    .into_raw();
    let symk_ptr = CString::new(symk)
        .expect("CString should build properly from hex encoded symmetric key")
        .into_raw();
    let sk_ptr = CString::new(sk)
        .expect("CString should build properly from hex encoded signing key")
        .into_raw();

    let mut result: String = Default::default();
    let result_cb = |v: &str| result = v.to_string();
    let code = unsafe {
        let mut closure = result_cb;
        let cb = get_trampoline(&closure);
        let out = waku_sys::waku_encode_symmetric(
            message_ptr,
            symk_ptr,
            sk_ptr,
            cb,
            &mut closure as *mut _ as *mut c_void,
        );

        drop(CString::from_raw(message_ptr));
        drop(CString::from_raw(symk_ptr));
        drop(CString::from_raw(sk_ptr));

        out
    };

    handle_json_response(code, &result)
}
