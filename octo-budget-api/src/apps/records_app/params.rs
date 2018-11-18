const DEFAULT_PER_PAGE: i64 = 10;

use super::response_data::ResponseData;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Params {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    DEFAULT_PER_PAGE
}

impl Params {
    // TODO: proper validation
    pub fn validate(self) -> Result<Self, ResponseData> {
        Ok(self)
    }
}
