use super::auth_error::AuthError;
use octo_budget_lib::auth_token::AuthToken;

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct ResponseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<AuthToken>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "password")]
    pub password_errors: Vec<AuthError>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "username")]
    pub username_errors: Vec<AuthError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub non_field_errors: Vec<AuthError>,
}

impl ResponseData {
    pub fn from_token(token: AuthToken) -> Self {
        Self {
            token: Some(token),
            ..Self::default()
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.password_errors.is_empty()
            || !self.username_errors.is_empty()
            || !self.non_field_errors.is_empty()
    }
}
