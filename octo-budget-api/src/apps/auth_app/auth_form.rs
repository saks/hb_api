use super::auth_error::AuthError;
use super::response_data::ResponseData;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct AuthForm {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl AuthForm {
    pub fn validate(self) -> Result<(String, String), ResponseData> {
        let AuthForm {
            username, password, ..
        } = self;

        let mut data = ResponseData::default();

        if username.is_none() {
            data.username_errors.push(AuthError::MustPresent);
        } else if let Some(val) = username.as_ref() {
            if val.is_empty() {
                data.username_errors.push(AuthError::CannotBeBlank);
            }
        }

        if password.is_none() {
            data.password_errors.push(AuthError::MustPresent);
        } else if let Some(val) = password.as_ref() {
            if val.is_empty() {
                data.password_errors.push(AuthError::CannotBeBlank);
            }
        }

        if data.has_errors() {
            Err(data)
        } else {
            let username = username.unwrap();
            let password = password.unwrap();

            Ok((username, password))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_ok() {
        let form = AuthForm {
            username: Some("foo".to_string()),
            password: Some("bar".to_string()),
        };

        let res = form.validate();
        assert!(res.is_ok());

        let (username, password) = res.unwrap();

        assert_eq!("foo", username);
        assert_eq!("bar", password);
    }

    #[test]
    fn test_no_username() {
        let form = AuthForm {
            username: None,
            password: Some("bar".to_string()),
        };

        let response_data = form.validate().unwrap_err();

        assert_eq!(vec![AuthError::MustPresent], response_data.username_errors);

        assert!(response_data.token.is_none());
        assert!(response_data.password_errors.is_empty());
        assert!(response_data.non_field_errors.is_empty());
    }

    #[test]
    fn test_no_password() {
        let form = AuthForm {
            username: Some("foo".to_string()),
            password: None,
        };

        let response_data = form.validate().unwrap_err();

        assert_eq!(vec![AuthError::MustPresent], response_data.password_errors);

        assert!(response_data.token.is_none());
        assert!(response_data.username_errors.is_empty());
        assert!(response_data.non_field_errors.is_empty());
    }

    #[test]
    fn test_username_is_empty() {
        let form = AuthForm {
            username: Some("".to_string()),
            password: Some("bar".to_string()),
        };

        let response_data = form.validate().unwrap_err();

        assert_eq!(
            vec![AuthError::CannotBeBlank],
            response_data.username_errors
        );

        assert!(response_data.token.is_none());
        assert!(response_data.password_errors.is_empty());
        assert!(response_data.non_field_errors.is_empty());
    }

    #[test]
    fn test_password_is_empty() {
        let form = AuthForm {
            username: Some("foo".to_string()),
            password: Some("".to_string()),
        };

        let response_data = form.validate().unwrap_err();

        assert_eq!(
            vec![AuthError::CannotBeBlank],
            response_data.password_errors
        );

        assert!(response_data.token.is_none());
        assert!(response_data.username_errors.is_empty());
        assert!(response_data.non_field_errors.is_empty());
    }

    #[test]
    fn test_username_and_password_is_empty() {
        let form = AuthForm {
            username: Some("".to_string()),
            password: Some("".to_string()),
        };

        let response_data = form.validate().unwrap_err();

        assert_eq!(
            vec![AuthError::CannotBeBlank],
            response_data.password_errors
        );

        assert_eq!(
            vec![AuthError::CannotBeBlank],
            response_data.username_errors
        );

        assert!(response_data.token.is_none());
        assert!(response_data.non_field_errors.is_empty());
    }

    #[test]
    fn test_no_username_and_no_password() {
        let form = AuthForm {
            username: None,
            password: None,
        };

        let response_data = form.validate().unwrap_err();

        assert_eq!(vec![AuthError::MustPresent], response_data.password_errors);
        assert_eq!(vec![AuthError::MustPresent], response_data.username_errors);

        assert!(response_data.token.is_none());
        assert!(response_data.non_field_errors.is_empty());
    }
}
