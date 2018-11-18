use serde::{Serialize, Serializer};

use crate::config;

#[derive(Debug, PartialEq, Deserialize)]
pub struct AuthToken {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Data {
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub exp: i64,
    //     username: String,
    //     email: String,
}

impl From<Data> for Claims {
    fn from(data: Data) -> Self {
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
        let user_id = data.user_id;

        Self { user_id, exp }
    }
}

impl AuthToken {
    pub fn new(user_id: i32) -> Self {
        let data = Data { user_id };
        Self { data }
    }

    pub fn verify(token: &str) -> Result<Self, ()> {
        use jsonwebtoken::{decode, Validation};
        let secret = (&**config::AUTH_TOKEN_SECRET).as_ref();

        let token_data = decode::<Claims>(token, secret, &Validation::default()).map_err(|_| ())?;

        let user_id = token_data.claims.user_id;
        let data = Data { user_id };

        Ok(Self { data })
    }

    fn to_string(&self) -> String {
        use crate::config;
        use jsonwebtoken::{encode, Header};

        let my_claims: Claims = self.data.clone().into();
        let secret = (&**config::AUTH_TOKEN_SECRET).as_ref();

        encode(&Header::default(), &my_claims, secret).expect("Failed to generate token")
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
    use jsonwebtoken::{decode, Validation};
    use std::env;

    const TEST_SECRET: &[u8] = b"foo-bar-secret";
    const TEST_USER_ID: i32 = 112233;

    #[test]
    fn test_create_token() {
        setup();

        let token = AuthToken::new(TEST_USER_ID).to_string();
        assert_eq!(128, token.len());

        let decoded = decode::<Claims>(&token, TEST_SECRET, &Validation::default()).unwrap();
        assert_eq!(TEST_USER_ID, decoded.claims.user_id);

        teardown();
    }

    #[test]
    #[should_panic(expected = "InvalidSignature")]
    fn test_create_token_with_invalid_secret() {
        setup();

        let token = AuthToken::new(TEST_USER_ID).to_string();
        teardown(); // cleanup state before panic

        decode::<Claims>(&token, b"secret", &Validation::default()).unwrap();
    }

    #[test]
    fn test_verify_token() {
        setup();

        let valid_token = make_token(33, TEST_SECRET);

        let result = AuthToken::verify(&valid_token);

        // assert!(result.is_ok());
        assert_eq!(AuthToken::new(TEST_USER_ID), result.unwrap());

        teardown()
    }

    #[test]
    fn test_verify_expired_token() {
        setup();

        let valid_token = make_token(-33, TEST_SECRET);

        assert!(AuthToken::verify(&valid_token).is_err());

        teardown();
    }

    #[test]
    fn test_verify_token_with_wrong_signature() {
        setup();

        let valid_token = make_token(33, b"bar");

        assert!(AuthToken::verify(&valid_token).is_err());

        teardown();
    }

    fn make_token(hours_from_now: i64, secret: &[u8]) -> String {
        use jsonwebtoken::{encode, Header};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(hours_from_now))
            .to_timespec()
            .sec;
        let user_id = TEST_USER_ID;
        let my_claims = Claims { user_id, exp };

        encode(&Header::default(), &my_claims, secret).expect("Failed to generate token")
    }

    fn setup() {
        env::set_var(
            "AUTH_TOKEN_SECRET",
            String::from_utf8(TEST_SECRET.to_vec()).unwrap(),
        );
    }

    fn teardown() {
        env::remove_var("AUTH_TOKEN_SECRET");
    }
}
