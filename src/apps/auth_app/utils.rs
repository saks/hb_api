use actix_web::Error;

use super::{auth_error::AuthError, response_data::ResponseData};
use auth_token::AuthToken;
use db::{auth::FindUserResult, models::AuthUser as UserModel};

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

pub fn generate_token(user: &UserModel) -> ResponseData {
    let token = AuthToken::new(user.id);
    ResponseData::from_token(token)
}

#[cfg(test)]
mod test {
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
        assert_eq!(ResponseData::from_token(AuthToken::new(user.id)), data);
    }

    fn make_user_with_pass(password: &'static str) -> UserModel {
        use djangohashers;

        UserModel {
            id: 123,
            username: "".to_string(),
            password: djangohashers::make_password(password),
            email: "".to_string(),
            is_active: true,
        }
    }
}
