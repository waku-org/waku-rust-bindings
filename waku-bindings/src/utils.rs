use crate::general::Result;
use core::str::FromStr;
use std::convert::TryFrom;
use std::{slice, str};
use waku_sys::WakuCallBack;
use waku_sys::{RET_ERR, RET_MISSING_CALLBACK, RET_OK};

#[derive(Debug, Default, PartialEq)]
pub enum LibwakuResponse {
    Success(Option<String>),
    Failure(String),
    MissingCallback,
    #[default]
    Undefined,
}

impl TryFrom<(u32, &str)> for LibwakuResponse {
    type Error = String;

    fn try_from((ret_code, response): (u32, &str)) -> std::result::Result<Self, Self::Error> {
        let opt_value = Some(response.to_string()).filter(|s| !s.is_empty());
        match ret_code {
            RET_OK => Ok(LibwakuResponse::Success(opt_value)),
            RET_ERR => Ok(LibwakuResponse::Failure(format!(
                "waku error: {}",
                response
            ))),
            RET_MISSING_CALLBACK => Ok(LibwakuResponse::MissingCallback),
            _ => Err(format!("undefined return code {}", ret_code)),
        }
    }
}

// Define the WakuDecode trait
pub trait WakuDecode: Sized {
    fn decode(input: &str) -> Result<Self>;
}

pub fn decode<T: WakuDecode>(input: String) -> Result<T> {
    T::decode(input.as_str())
}

unsafe extern "C" fn trampoline<F>(
    ret_code: ::std::os::raw::c_int,
    data: *const ::std::os::raw::c_char,
    data_len: usize,
    user_data: *mut ::std::os::raw::c_void,
) where
    F: FnMut(LibwakuResponse),
{
    let user_data = &mut *(user_data as *mut F);

    let response = if data.is_null() {
        ""
    } else {
        str::from_utf8(slice::from_raw_parts(data as *mut u8, data_len))
            .expect("could not retrieve response")
    };

    let result = LibwakuResponse::try_from((ret_code as u32, response))
        .expect("invalid response obtained from libwaku");

    user_data(result);
}

pub fn get_trampoline<F>(_closure: &F) -> WakuCallBack
where
    F: FnMut(LibwakuResponse),
{
    Some(trampoline::<F>)
}

pub fn handle_no_response(code: i32, result: LibwakuResponse) -> Result<()> {
    if result == LibwakuResponse::Undefined && code as u32 == RET_OK {
        // Some functions will only execute the callback on error
        return Ok(());
    }

    match result {
        LibwakuResponse::Success(_) => Ok(()),
        LibwakuResponse::Failure(v) => Err(v),
        LibwakuResponse::MissingCallback => panic!("callback is required"),
        LibwakuResponse::Undefined => panic!(
            "undefined ffi state: code({}) was returned but callback was not executed",
            code
        ),
    }
}

pub fn handle_json_response<F: WakuDecode>(code: i32, result: LibwakuResponse) -> Result<F> {
    match result {
        LibwakuResponse::Success(v) => decode(v.unwrap_or_default()),
        LibwakuResponse::Failure(v) => Err(v),
        LibwakuResponse::MissingCallback => panic!("callback is required"),
        LibwakuResponse::Undefined => panic!(
            "undefined ffi state: code({}) was returned but callback was not executed",
            code
        ),
    }
}

pub fn handle_response<F: FromStr>(code: i32, result: LibwakuResponse) -> Result<F> {
    match result {
        LibwakuResponse::Success(v) => v
            .unwrap_or_default()
            .parse()
            .map_err(|_| "could not parse value".into()),
        LibwakuResponse::Failure(v) => Err(v),
        LibwakuResponse::MissingCallback => panic!("callback is required"),
        LibwakuResponse::Undefined => panic!(
            "undefined ffi state: code({}) was returned but callback was not executed",
            code
        ),
    }
}
