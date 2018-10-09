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
        } else if let Some(val) = username.clone() {
            if val.is_empty() {
                data.username_errors.push(AuthError::CannotBeBlank);
            }
        }

        if password.is_none() {
            data.password_errors.push(AuthError::MustPresent);
        } else if let Some(val) = password.clone() {
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
