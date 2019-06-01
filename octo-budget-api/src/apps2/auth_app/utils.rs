use octo_budget_lib::auth_token::AuthToken;

use super::response_data::Data;
use crate::config;
use crate::db::models::AuthUser as UserModel;

pub fn generate_token(user: &UserModel) -> Data {
    let secret = config::AUTH_TOKEN_SECRET.as_bytes();
    let token = AuthToken::new(user.id).encrypt(secret);
    Data::from_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        dotenv::dotenv().expect("Failed to parse .env file");
    }

    #[test]
    fn test_generate_token() {
        setup();

        let user = make_user_with_pass("foo");
        let data = generate_token(&make_user_with_pass("foo"));
        let token = AuthToken::new(user.id).encrypt(config::AUTH_TOKEN_SECRET.as_bytes());
        let expected_data = Data::from_token(token);

        assert_eq!(expected_data, data);
    }

    fn make_user_with_pass(password: &'static str) -> UserModel {
        setup();

        use chrono::naive::NaiveDateTime;

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
