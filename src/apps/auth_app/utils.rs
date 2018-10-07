use actix_web::{Error as WebError, HttpResponse};

use super::auth_error::AuthError;
use super::ResponseData;
use db::auth::FindResult;
use db::models::AuthUser as UserModel;

pub fn create_token(user_id: i32) -> String {
    use config;
    use frank_jwt::{encode, Algorithm};
    use time::{now_utc, Duration};

    let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
    let payload = json!({ "user_id": user_id });
    let header = json!({ "exp": exp });
    let secret = &config::AUTH_TOKEN_SECRET.to_string();

    encode(header, secret, &payload, Algorithm::HS256).expect("Failed to generate token")
}

pub fn validate_user(find_result: FindResult) -> Result<UserModel, WebError> {
    find_result.map_err(|e| {
        println!("E: Failed to find user by username: {:?}", e);
        AuthError::AuthFailed.into()
    })
}

pub fn validate_password(user: UserModel, password: String) -> Result<UserModel, WebError> {
    use djangohashers;

    match djangohashers::check_password(&password, &user.password) {
        Ok(true) => Ok(user),
        _ => Err(AuthError::AuthFailed.into()),
    }
}

pub fn generate_token(user: UserModel) -> Result<HttpResponse, WebError> {
    let token = create_token(user.id);
    let data = ResponseData::from_token(token);

    Ok(HttpResponse::Ok().json(data))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_token() {
        use frank_jwt::{decode, validate_signature, Algorithm};
        use std::env;

        let secret = "foo".to_string();
        env::set_var("AUTH_TOKEN_SECRET", &secret);

        let token = create_token(123);

        assert_eq!(125, token.len());

        let data = decode(&token, &secret, Algorithm::HS256).unwrap();
        let (_header, data) = data;

        assert_eq!(123, data["user_id"]);

        let validation_result = validate_signature(&token, &secret, Algorithm::HS256);
        assert_eq!(Ok(true), validation_result);

        env::remove_var("AUTH_TOKEN_SECRET");
    }
}
