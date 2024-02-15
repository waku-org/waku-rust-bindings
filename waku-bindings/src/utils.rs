use crate::general::Result;
use core::str::FromStr;
use serde::de::DeserializeOwned;
use std::{slice, str};
use waku_sys::WakuCallBack;
use waku_sys::{RET_ERR, RET_MISSING_CALLBACK, RET_OK};

pub fn decode<T: DeserializeOwned>(input: &str) -> Result<T> {
    serde_json::from_str(input)
        .map_err(|err| format!("could not deserialize waku response: {}", err))
}

unsafe extern "C" fn trampoline<F>(
    _ret_code: ::std::os::raw::c_int,
    data: *const ::std::os::raw::c_char,
    data_len: usize,
    user_data: *mut ::std::os::raw::c_void,
) where
    F: FnMut(&str),
{
    let response = if data.is_null() {
        ""
    } else {
        str::from_utf8(slice::from_raw_parts(data as *mut u8, data_len))
            .expect("could not retrieve response")
    };

    if !user_data.is_null() {
        let user_data = &mut *(user_data as *mut F);
        user_data(response);
    }
}

pub fn get_trampoline<F>(_closure: &F) -> WakuCallBack
where
    F: FnMut(&str),
{
    Some(trampoline::<F>)
}

pub fn handle_no_response(code: i32, error: &str) -> Result<()> {
    match code as u32 {
        RET_OK => Ok(()),
        RET_ERR => Err(format!("waku error: {}", error)),
        RET_MISSING_CALLBACK => Err("missing callback".to_string()),
        _ => Err(format!("undefined return code {}", code)),
    }
}

pub fn handle_json_response<F: DeserializeOwned>(code: i32, result: &str) -> Result<F> {
    match code as u32 {
        RET_OK => decode(result),
        RET_ERR => Err(format!("waku error: {}", result)),
        RET_MISSING_CALLBACK => Err("missing callback".to_string()),
        _ => Err(format!("undefined return code {}", code)),
    }
}

pub fn handle_response<F: FromStr>(code: i32, result: &str) -> Result<F> {
    match code as u32 {
        RET_OK => result
            .parse()
            .map_err(|_| format!("could not parse value: {}", result)),
        RET_ERR => Err(format!("waku error: {}", result)),
        RET_MISSING_CALLBACK => Err("missing callback".to_string()),
        _ => Err(format!("undefined return code {}", code)),
    }
}
