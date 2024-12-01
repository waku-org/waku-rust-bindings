pub struct MessageHash {
    pub data: [u8; 32],
}

impl MessageHash {
    // Create a new hash with default (zeroed) data
    pub fn new() -> Self {
        MessageHash { data: [0u8; 32] }
    }
}
