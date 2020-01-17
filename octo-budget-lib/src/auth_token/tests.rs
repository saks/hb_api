use super::*;
use jsonwebtoken::{decode, Validation};

const TEST_SECRET: &[u8] = b"foo-bar-secret";
const TEST_USER_ID: i32 = 112233;

#[test]
fn test_create_token() {
    let token = AuthToken::new(TEST_USER_ID).encrypt(TEST_SECRET);
    assert_eq!(128, token.len());

    let decoded = decode::<Data>(&token, TEST_SECRET, &Validation::default()).unwrap();
    assert_eq!(TEST_USER_ID, decoded.claims.user_id);
}

#[test]
#[should_panic(expected = "InvalidSignature")]
fn test_create_token_with_invalid_secret() {
    let token = AuthToken::new(TEST_USER_ID).encrypt(TEST_SECRET);
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
    AuthToken::new(TEST_USER_ID)
        .expire_in_hours(hours_from_now)
        .encrypt(secret)
}
