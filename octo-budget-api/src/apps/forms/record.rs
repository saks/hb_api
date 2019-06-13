use actix_web::{error::ResponseError, HttpResponse};
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use failure_derive::Fail;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Form {
    tags: Vec<String>,
    transaction_type: String,
    amount: Amount,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Amount {
    amount: f64,
    currency: Currency,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Currency {
    code: String,
    name: String,
}

#[derive(Debug, Default)]
pub struct FormData {
    pub transaction_type: String,
    pub tags: Vec<String>,
    pub amount: BigDecimal,
    pub amount_currency: String,
}

#[derive(Debug, Fail, Serialize, Default)]
pub struct ValidationErrors {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    transaction_type: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    amount: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    currency_code: Vec<String>,
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
        self.transaction_type.is_empty() && self.amount.is_empty() && self.currency_code.is_empty()
    }
}

impl Form {
    pub fn validate(self) -> Result<FormData, ValidationErrors> {
        let Self {
            transaction_type,
            tags,
            amount,
        } = self;
        let mut errors = ValidationErrors::default();

        match transaction_type.as_str() {
            "EXP" | "INC" => {}
            other => errors
                .transaction_type
                .push(format!("\"{}\" is not a valid choice.", other)),
        };

        let amount_number = match BigDecimal::from_f64(amount.amount) {
            Some(n) => n,
            None => {
                errors
                    .amount
                    .push(format!("Cannot parse a number from {}", amount.amount));
                BigDecimal::zero()
            }
        };

        match amount.currency.code.as_str() {
            "CAD" => {}
            other => errors
                .currency_code
                .push(format!("\"{}\" is not a valid choice.", other)),
        };

        if errors.is_empty() {
            Ok(FormData {
                transaction_type,
                tags,
                amount: amount_number,
                amount_currency: amount.currency.code,
            })
        } else {
            Err(errors)
        }
    }
}
