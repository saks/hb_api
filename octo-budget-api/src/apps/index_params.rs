const DEFAULT_PER_PAGE: i64 = 10;
const DEFAULT_PAGE: i64 = 1;

use actix_web::{error::ResponseError, HttpResponse};
use failure::Fail;
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

#[cfg_attr(test, derive(Debug))]
pub struct Data {
    pub page: i64,
    pub per_page: i64,
}

impl Params {
    pub fn validate(&self) -> Result<Data, ValidationErrors> {
        let Self { page, per_page } = self;
        let mut errors = ValidationErrors::default();

        if page.is_negative() {
            errors.page.push("Must be a positive number".to_string());
        }

        if per_page.is_negative() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn errors_json(params: Params) -> String {
        serde_json::to_string(&params.validate().unwrap_err()).expect("Failed to convert to json")
    }

    #[test]
    fn it_is_ok_when_valid() {
        let params = Params { page: 0, per_page: 10 };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn data_is_correct_when_valid() {
        let params = Params { page: 3, per_page: 10 };
        let data = params.validate().expect("is expected to be valid");

        assert_eq!(3, data.page);
        assert_eq!(10, data.per_page);
    }

    #[test]
    fn invalid_when_page_number_is_negative() {
        let params = Params { page: -1, per_page: 123 };

        assert_eq!("{\"page\":[\"Must be a positive number\"]}", errors_json(params));
    }

    #[test]
    fn invalid_when_per_page_is_negative() {
        let params = Params { page: 0, per_page: -1 };

        assert_eq!("{\"per_page\":[\"Must be a positive number\"]}", errors_json(params));
    }
}
