// std

// crates
use serde::Deserialize;
// internal

/// JsonResponse wrapper.
/// `go-waku` ffi returns this type as a `char *` as per the [specification](https://rfc.vac.dev/spec/36/#jsonresponse-type)
///
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JsonResponse<T> {
    Result(T),
    Error(String),
}

type Response<T> = Result<T, String>;

impl<T> From<JsonResponse<T>> for Response<T> {
    fn from(response: JsonResponse<T>) -> Self {
        match response {
            JsonResponse::Result(t) => Ok(t),
            JsonResponse::Error(e) => Err(e),
        }
    }
}
