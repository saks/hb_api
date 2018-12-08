use actix_web::Error;
use octo_budget_lib::auth_token::AuthToken;

use super::{auth_error::AuthError, response_data::Data};
use crate::config;
use crate::db::{auth::FindUserResult, models::AuthUser as UserModel};

pub fn validate_user(find_result: FindUserResult) -> Result<UserModel, Error> {
    find_result.map_err(|e| {
        println!("E: Failed to find user by username: {:?}", e);
        AuthError::AuthFailed.into()
    })
}

pub fn validate_password(user: UserModel, password: String) -> Result<UserModel, AuthError> {
    use djangohashers;

    match djangohashers::check_password(&password, &user.password) {
        Ok(true) => Ok(user),
        _ => Err(AuthError::AuthFailed)?,
    }
}

pub fn generate_token(user: &UserModel) -> Data {
    let secret = config::AUTH_TOKEN_SECRET.as_bytes();
    let token = AuthToken::new(user.id, secret).to_string();
    Data::from_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_ok() {
        let user = make_user_with_pass("foo");
        let result = validate_password(user.clone(), "foo".to_string());

        assert_eq!(user, result.unwrap());
    }

    #[test]
    fn test_validate_password_err() {
        let user = make_user_with_pass("foo");
        let result = validate_password(user.clone(), "bar".to_string());

        assert_eq!(AuthError::AuthFailed, result.unwrap_err());
    }

    #[test]
    fn test_generate_token() {
        let user = make_user_with_pass("foo");
        let data = generate_token(&make_user_with_pass("foo"));
        let token = AuthToken::new(user.id, config::AUTH_TOKEN_SECRET.as_bytes()).to_string();
        let expected_data = Data::from_token(token);

        assert_eq!(expected_data, data);
    }

    fn make_user_with_pass(password: &'static str) -> UserModel {
        use chrono::naive::NaiveDateTime;
        use djangohashers;

        UserModel {
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
}
