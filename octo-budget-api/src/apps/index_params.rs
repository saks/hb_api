const DEFAULT_PER_PAGE: i64 = 10;
const DEFAULT_PAGE: i64 = 1;

use actix_web::{error::ResponseError, HttpResponse};
use failure_derive::Fail;
use serde_derive::{Deserialize, Serialize};

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
    pub fn validate(self) -> Result<Data, ValidationErrors> {
        let Self { page, per_page } = self;
        let _errors = ValidationErrors::default();

        // match transaction_type.as_str() {
        //     "EXP" | "INC" => {}
        //     other @ _ => errors
        //         .transaction_type
        //         .push(format!("\"{}\" is not a valid choice.", other)),
        // };
        //
        // let amount_number = match BigDecimal::from_f64(amount.amount) {
        //     Some(n) => n,
        //     None => {
        //         errors
        //             .amount
        //             .push(format!("Cannot parse a number from {}", amount.amount));
        //         BigDecimal::zero()
        //     }
        // };
        //
        // match amount.currency.code.as_str() {
        //     "CAD" => {}
        //     other @ _ => errors
        //         .currency_code
        //         .push(format!("\"{}\" is not a valid choice.", other)),
        // };

        if _errors.is_empty() {
            Ok(Data { page, per_page })
        } else {
            Err(_errors)
        }
    }
}
