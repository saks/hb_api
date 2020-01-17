use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use failure::Fail;
use serde::{Deserialize, Serialize};

use crate::db::models::AuthUser;
use crate::errors::ValidationError;

#[derive(Deserialize, Debug, Default)]
pub struct Form {
    username: Option<String>,
    password: Option<String>,
}

impl Form {
    pub fn validate_password(user: &AuthUser, password: &str) -> Result<(), ValidationErrors> {
        match djangohashers::check_password(password, &user.password) {
            Ok(true) => Ok(()),
            _ => Err(ValidationErrors::bad_password()),
        }
    }
}

#[derive(Debug)]
pub struct Data {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Fail, Serialize, Default, PartialEq)]
pub struct ValidationErrors {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    password: Vec<ValidationError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    username: Vec<ValidationError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    non_field_errors: Vec<ValidationError>,
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ValidationErrors {
    fn error_response(&self) -> HttpResponse {
        let status_code = if self.non_field_errors.is_empty() {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::UNAUTHORIZED
        };

        HttpResponse::build(status_code).json(self)
    }
}

impl ValidationErrors {
    fn is_empty(&self) -> bool {
        self.username.is_empty() && self.password.is_empty() && self.non_field_errors.is_empty()
    }

    fn bad_password() -> Self {
        ValidationErrors {
            non_field_errors: vec![ValidationError::AuthFailed],
            username: vec![],
            password: vec![],
        }
    }
}

impl Form {
    pub fn validate(self) -> Result<Data, ValidationErrors> {
        let Self { username, password } = self;
        let mut errors = ValidationErrors::default();

        if username.is_none() {
            errors.username.push(ValidationError::MustPresent);
        } else if let Some(val) = username.as_ref() {
            if val.is_empty() {
                errors.username.push(ValidationError::CannotBeBlank);
            }
        }

        if password.is_none() {
            errors.password.push(ValidationError::MustPresent);
        } else if let Some(val) = password.as_ref() {
            if val.is_empty() {
                errors.password.push(ValidationError::CannotBeBlank);
            }
        }

        if errors.is_empty() {
            let password = password.unwrap();
            let username = username.unwrap();

            Ok(Data { username, password })
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests;
