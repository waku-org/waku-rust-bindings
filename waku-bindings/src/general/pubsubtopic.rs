use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PubsubTopic(String);

impl PubsubTopic {
    // Constructor to create a new MyString
    pub fn new(value: &str) -> Self {
        PubsubTopic(value.to_string())
    }
}

// to allow conversion from `PubsubTopic` to `String`
impl From<&PubsubTopic> for String {
    fn from(topic: &PubsubTopic) -> Self {
        topic.0.to_string()
    }
}
