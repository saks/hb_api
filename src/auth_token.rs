use serde::{Serialize, Serializer};
use serde_json;

use config;

#[derive(Debug, PartialEq)]
pub struct AuthToken {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Data {
    pub user_id: i32,
}

impl AuthToken {
    pub fn new(user_id: i32) -> Self {
        let data = Data { user_id };
        Self { data }
    }

    pub fn verify(header_str: &str) -> Result<Self, ()> {
        use frank_jwt::{decode, Algorithm};
        use time::now_utc;

        let payload = header_str.to_string();
        let secret = config::AUTH_TOKEN_SECRET.to_string();
        let (header, data) = decode(&payload, &secret, Algorithm::HS256).map_err(|_| ())?;

        let exp = header.get("exp").and_then(|exp| exp.as_i64()).ok_or(())?;

        let now = now_utc().to_timespec().sec;
        if exp < now {
            return Err(());
        }

        let data = serde_json::from_value(data).map_err(|_| ())?;

        Ok(Self { data })
    }

    fn to_string(&self) -> String {
        use config;
        use frank_jwt::{encode, Algorithm};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
        let payload = json!(self.data);
        let header = json!({ "exp": exp });
        let secret = &config::AUTH_TOKEN_SECRET.to_string();

        encode(header, secret, &payload, Algorithm::HS256).expect("Failed to generate token")
    }
}

impl Serialize for AuthToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = self.to_string();
        serializer.serialize_str(&string)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    #[test]
    fn test_create_token() {
        use frank_jwt::{decode, validate_signature, Algorithm};
        use std::env;

        let secret = "foo".to_string();
        env::remove_var("AUTH_TOKEN_SECRET");
        env::set_var("AUTH_TOKEN_SECRET", &secret);

        let token = AuthToken::new(123).to_string();

        assert_eq!(124, token.len());

        let data = decode(&token, &secret, Algorithm::HS256).unwrap();
        let (_header, data) = data;

        assert_eq!(123, data["user_id"]);

        let validation_result = validate_signature(&token, &secret, Algorithm::HS256);
        assert_eq!(Ok(true), validation_result);

        env::remove_var("AUTH_TOKEN_SECRET");
    }

    fn make_token(hours_from_now: i64, secret_str: &str) -> String {
        use frank_jwt::{encode, Algorithm};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(hours_from_now))
            .to_timespec()
            .sec;
        let header = json!({ "exp": exp });
        let payload = json!({ "user_id": 123 });
        let secret = secret_str.to_string();

        encode(header, &secret, &payload, Algorithm::HS256).expect("failed to encode token")
    }

    fn setup() {
        env::set_var("AUTH_TOKEN_SECRET", "foo");
    }

    fn teardown() {
        env::remove_var("AUTH_TOKEN_SECRET");
    }

    #[test]
    fn test_verify_token() {
        setup();

        let valid_token = make_token(33, "foo");

        let result = AuthToken::verify(&valid_token);

        assert!(result.is_ok());
        assert_eq!(AuthToken::new(123), result.unwrap());

        teardown()
    }

    #[test]
    fn test_verify_expired_token() {
        setup();

        let valid_token = make_token(-33, "foo");

        assert!(AuthToken::verify(&valid_token).is_err());

        teardown();
    }

    #[test]
    fn test_verify_token_with_wrong_signature() {
        setup();

        let valid_token = make_token(33, "bar");

        assert!(AuthToken::verify(&valid_token).is_err());

        teardown();
    }
}
