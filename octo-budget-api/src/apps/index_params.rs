const DEFAULT_PER_PAGE: i64 = 10;
const DEFAULT_PAGE: i64 = 1;

use super::index_response::Data;
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Params {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    DEFAULT_PAGE
}

fn default_per_page() -> i64 {
    DEFAULT_PER_PAGE
}

impl Params {
    // TODO: proper validation
    pub fn validate<T: Serialize>(self) -> Result<Self, Data<T>> {
        Ok(self)
    }
}
