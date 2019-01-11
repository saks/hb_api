use serde_derive::Serialize;

#[derive(Serialize, Debug, PartialEq)]
pub struct Data {
    token: String,
}

impl Data {
    pub fn from_token(token: String) -> Self {
        Self { token }
    }
}
