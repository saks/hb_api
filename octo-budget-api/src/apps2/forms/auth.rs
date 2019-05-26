use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use failure_derive::Fail;
use serde_derive::{Deserialize, Serialize};

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
    fn render_response(&self) -> HttpResponse {
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
mod tests {
    use super::*;
    #[test]
    fn test_validate_password_ok() {
        let user = make_user_with_pass("foo");
        let result = Form::validate_password(&user, "foo");

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_err() {
        let user = make_user_with_pass("foo");
        let result = Form::validate_password(&user, "bar");

        assert_eq!(ValidationErrors::bad_password(), result.unwrap_err());
    }

    fn make_user_with_pass(password: &'static str) -> AuthUser {
        use chrono::naive::NaiveDateTime;

        AuthUser {
            id: 123,
            username: "".to_string(),
            password: djangohashers::make_password(password),
            email: "".to_string(),
            is_active: true,
            is_superuser: true,
            first_name: "".to_string(),
            last_name: "".to_string(),
            is_staff: false,
            date_joined: NaiveDateTime::from_timestamp(0, 0),
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_validate_ok() {
        let form = Form {
            username: Some("foo".to_string()),
            password: Some("bar".to_string()),
        };

        let res = form.validate();
        assert!(res.is_ok());

        let params = res.unwrap();

        assert_eq!("foo", params.username);
        assert_eq!("bar", params.password);
    }

    #[test]
    fn test_no_username() {
        let form = Form {
            username: None,
            password: Some("bar".to_string()),
        };

        let errors = form.validate().unwrap_err();

        assert_eq!(vec![ValidationError::MustPresent], errors.username);

        assert!(errors.password.is_empty());
        assert!(errors.non_field_errors.is_empty());
    }

    #[test]
    fn test_no_password() {
        let form = Form {
            username: Some("foo".to_string()),
            password: None,
        };

        let errors = form.validate().unwrap_err();

        assert_eq!(vec![ValidationError::MustPresent], errors.password);

        assert!(errors.username.is_empty());
        assert!(errors.non_field_errors.is_empty());
    }

    #[test]
    fn test_username_is_empty() {
        let form = Form {
            username: Some("".to_string()),
            password: Some("bar".to_string()),
        };

        let errors = form.validate().unwrap_err();

        assert_eq!(vec![ValidationError::CannotBeBlank], errors.username);

        assert!(errors.password.is_empty());
        assert!(errors.non_field_errors.is_empty());
    }

    #[test]
    fn test_password_is_empty() {
        let form = Form {
            username: Some("foo".to_string()),
            password: Some("".to_string()),
        };

        let errors = form.validate().unwrap_err();

        assert_eq!(vec![ValidationError::CannotBeBlank], errors.password);

        assert!(errors.username.is_empty());
        assert!(errors.non_field_errors.is_empty());
    }

    #[test]
    fn test_username_and_password_is_empty() {
        let form = Form {
            username: Some("".to_string()),
            password: Some("".to_string()),
        };

        let errors = form.validate().unwrap_err();

        assert_eq!(vec![ValidationError::CannotBeBlank], errors.password);

        assert_eq!(vec![ValidationError::CannotBeBlank], errors.username);

        assert!(errors.non_field_errors.is_empty());
    }

    #[test]
    fn test_no_username_and_no_password() {
        let form = Form {
            username: None,
            password: None,
        };

        let errors = form.validate().unwrap_err();

        assert_eq!(vec![ValidationError::MustPresent], errors.password);
        assert_eq!(vec![ValidationError::MustPresent], errors.username);

        assert!(errors.non_field_errors.is_empty());
    }
}
