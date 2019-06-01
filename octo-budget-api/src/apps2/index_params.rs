const DEFAULT_PER_PAGE: i64 = 10;
const DEFAULT_PAGE: i64 = 1;

use actix_web::{error::ResponseError, HttpResponse};
use failure_derive::Fail;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Fail, Serialize, Default)]
pub struct ValidationErrors {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    page: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    per_page: Vec<String>,
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ValidationErrors {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(self)
    }
}

impl ValidationErrors {
    fn is_empty(&self) -> bool {
        self.page.is_empty() && self.per_page.is_empty()
    }
}

pub struct Data {
    pub page: i64,
    pub per_page: i64,
}

impl Params {
    // TODO: add tests
    pub fn validate(&self) -> Result<Data, ValidationErrors> {
        let Self { page, per_page } = self;
        let mut errors = ValidationErrors::default();

        if page < &0 {
            errors.page.push("Must be a positive number".to_string());
        }

        if per_page < &0 {
            errors
                .per_page
                .push("Must be a positive number".to_string());
        }

        if errors.is_empty() {
            Ok(Data {
                page: *page,
                per_page: *per_page,
            })
        } else {
            Err(errors)
        }
    }
}
