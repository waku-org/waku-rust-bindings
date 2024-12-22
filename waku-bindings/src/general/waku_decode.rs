use crate::general::Result;
use multiaddr::Multiaddr;
// Define the WakuDecode trait
pub trait WakuDecode: Sized {
    fn decode(input: &str) -> Result<Self>;
}

impl WakuDecode for String {
    fn decode(input: &str) -> Result<Self> {
        Ok(input.to_string())
    }
}

pub fn decode<T: WakuDecode>(input: String) -> Result<T> {
    T::decode(input.as_str())
}

impl WakuDecode for Vec<Multiaddr> {
    fn decode(input: &str) -> Result<Self> {
        input
            .split(',')
            .map(|s| s.trim().parse::<Multiaddr>().map_err(|err| err.to_string()))
            .collect::<Result<Vec<Multiaddr>>>() // Collect results into a Vec
            .map_err(|err| format!("could not parse Multiaddr: {}", err))
    }
}
