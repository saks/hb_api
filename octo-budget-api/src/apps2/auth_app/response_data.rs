use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Data {
    token: String,
}

impl Data {
    pub fn from_token(token: String) -> Self {
        Self { token }
    }
}
