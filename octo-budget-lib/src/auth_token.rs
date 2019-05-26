use failure::Error;
use serde::{Serialize as SerdeSerialize, Serializer};
use serde_derive::{Deserialize, Serialize};

const DEFAULT_EXPIRE_IN_HOURS: i64 = 24;

#[derive(Debug, PartialEq, Deserialize)]
pub struct AuthToken<'a> {
    pub user_id: i32,
    secret: &'a [u8],
    expire_in_hours: i64,
}

impl<'a> AuthToken<'a> {
    pub fn new(user_id: i32, secret: &'a [u8]) -> Self {
        let expire_in_hours = DEFAULT_EXPIRE_IN_HOURS;

        Self {
            user_id,
            expire_in_hours,
            secret,
        }
    }

    pub fn to_string(&self) -> String {
        use jsonwebtoken::{encode, Header};

        let headers = &Header::default();
        let secret = self.secret;
        let data = self.data();

        encode(headers, &data, secret).expect("Failed to generate token")
    }

    pub fn expire_in_hours(mut self, n: i64) -> Self {
        self.expire_in_hours = n;
        self
    }

    pub fn from(token: &str, secret: &'a [u8]) -> Result<Self, Error> {
        use jsonwebtoken::{decode, Validation};

        let token_data = decode::<Data>(token, secret, &Validation::default())?;
        let user_id = token_data.claims.user_id;

        Ok(Self::new(user_id, secret))
    }

    pub fn data(&self) -> Data {
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(self.expire_in_hours))
            .to_timespec()
            .sec;

        Data {
            exp,
            user_id: self.user_id,
        }
    }
}

impl<'a> SerdeSerialize for AuthToken<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Data {
    pub user_id: i32,
    pub exp: i64,
    // pub username: &'a str,
    // pub email: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, Validation};

    const TEST_SECRET: &[u8] = b"foo-bar-secret";
    const TEST_USER_ID: i32 = 112233;

    #[test]
    fn test_create_token() {
        let token = AuthToken::new(TEST_USER_ID, TEST_SECRET).to_string();
        assert_eq!(128, token.len());

        let decoded = decode::<Data>(&token, TEST_SECRET, &Validation::default()).unwrap();
        assert_eq!(TEST_USER_ID, decoded.claims.user_id);
    }

    #[test]
    #[should_panic(expected = "InvalidSignature")]
    fn test_create_token_with_invalid_secret() {
        let token = AuthToken::new(TEST_USER_ID, TEST_SECRET).to_string();
        decode::<Data>(&token, b"secret", &Validation::default()).unwrap();
    }

    #[test]
    fn test_verify_token() {
        let valid_token = make_token(24, TEST_SECRET);
        let result = AuthToken::from(&valid_token, TEST_SECRET).unwrap().user_id;

        assert_eq!(TEST_USER_ID, result);
    }

    #[test]
    fn test_verify_expired_token() {
        let token = make_token(-33, TEST_SECRET);

        assert!(AuthToken::from(&token, TEST_SECRET).is_err());
    }

    #[test]
    fn test_verify_token_with_wrong_signature() {
        let valid_token = make_token(33, b"bar");

        assert!(AuthToken::from(&valid_token, TEST_SECRET).is_err());
    }

    fn make_token(hours_from_now: i64, secret: &[u8]) -> String {
        AuthToken::new(TEST_USER_ID, secret)
            .expire_in_hours(hours_from_now)
            .to_string()
    }
}
