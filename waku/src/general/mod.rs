// std

// crates
use serde::{Deserialize, Serialize};
// internal

/// JsonResponse wrapper.
/// `go-waku` ffi returns this type as a `char *` as per the [specification](https://rfc.vac.dev/spec/36/#jsonresponse-type)
/// This is internal, as it is better to use rust plain `Result` type.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JsonResponse<T> {
    Result(T),
    Error(String),
}

/// Waku response, just a `Result` with an `String` error.
/// Convenient we can transform a [`JsonResponse`] into a [`Response`] (`Result`)
type Response<T> = Result<T, String>;

impl<T> From<JsonResponse<T>> for Response<T> {
    fn from(response: JsonResponse<T>) -> Self {
        match response {
            JsonResponse::Result(t) => Ok(t),
            JsonResponse::Error(e) => Err(e),
        }
    }
}

/// JsonMessage, Waku message in JSON format.
/// as per the [specification](https://rfc.vac.dev/spec/36/#jsonmessage-type)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessage {
    payload: Box<[u8]>,
    content_topic: String,
    version: usize,
    timestamp: usize,
}
