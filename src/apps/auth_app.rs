use std::convert::Into;

use actix_web::middleware::Logger;
use actix_web::{
    App, AsyncResponder, Error as WebError, FutureResponse, HttpResponse, Json, State,
};
use failure::Error;
use futures::future::{ok as fut_ok, Future};

use apps::AppState;
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

pub fn check_password(password: &str, hash: &str) -> Result<(), ValidationErrors> {
    use djangohashers;

    match djangohashers::check_password(password, hash) {
        Ok(true) => Ok(()),
        _ => {
            let mut errors = ValidationErrors::default();
            errors.non_field.push(ValidationError::AuthenticationFailed);
            Err(errors)
        }
    }
}

#[derive(Debug, Fail)]
pub enum ValidationError {
    #[fail(display = "This field may not be blank.")]
    CannotBeBlank,
    #[fail(display = "This field is required.")]
    MustPresent,
    #[fail(display = "Unable to log in with provided credentials.")]
    AuthenticationFailed,
}

#[derive(Serialize, Debug, Default)]
struct ResponseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "password")]
    password_errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "username")]
    username_errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    non_field_errors: Vec<String>,
    #[serde(skip)]
    form: AuthForm,
}

impl ResponseData {
    pub fn from_token(token: Option<String>) -> Self {
        Self {
            token,
            ..Self::default()
        }
    }
}

impl From<ValidationErrors> for ResponseData {
    fn from(errors: ValidationErrors) -> ResponseData {
        let mut data = ResponseData::default();

        data.username_errors = errors.username.iter().map(|e| e.to_string()).collect();
        data.password_errors = errors.password.iter().map(|e| e.to_string()).collect();
        data.non_field_errors = errors.non_field.iter().map(|e| e.to_string()).collect();

        data
    }
}

#[derive(Debug, Default)]
pub struct ValidationErrors {
    username: Vec<ValidationError>,
    password: Vec<ValidationError>,
    non_field: Vec<ValidationError>,
}

impl ValidationErrors {
    fn is_empty(&self) -> bool {
        self.username.is_empty() & self.password.is_empty() && self.non_field.is_empty()
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct AuthForm {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl AuthForm {
    pub fn validate(self) -> Result<Credentials, ValidationErrors> {
        use self::ValidationError::*;

        let AuthForm { username, password } = self;
        let mut errors = ValidationErrors::default();

        if username.is_none() {
            errors.username.push(MustPresent);
        } else if let Some(val) = username.clone() {
            if val.is_empty() {
                errors.username.push(CannotBeBlank);
            }
        }

        if password.is_none() {
            errors.password.push(MustPresent);
        } else if let Some(val) = password.clone() {
            if val.is_empty() {
                errors.password.push(CannotBeBlank);
            }
        }

        if errors.is_empty() {
            let username = username.unwrap();
            let password = password.unwrap();

            Ok(Credentials { username, password })
        } else {
            Err(errors)
        }
    }
}

impl Into<ResponseData> for AuthForm {
    fn into(self) -> ResponseData {
        let mut result = ResponseData::default();
        result.form = self;
        result
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

struct Authenticator {
    user: Option<UserModel>,
    credentials: Credentials,
}

impl Authenticator {
    fn validate(self) -> Result<String, ValidationErrors> {
        let Authenticator { credentials, user } = self;

        match user {
            Some(user) => {
                let Credentials { password, .. } = credentials;
                check_password(&password, &user.password).map(|_| create_token(user.id))
            }
            None => {
                let mut errors = ValidationErrors::default();
                errors.non_field.push(ValidationError::AuthenticationFailed);
                Err(errors)
            }
        }
    }
}

fn create((form_json, state): (Json<AuthForm>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let form = form_json.into_inner();

    match form.validate() {
        Ok(credentials) => state
            .db
            .send(credentials)
            .from_err()
            .and_then(authenticate_user)
            .responder(),
        Err(errors) => {
            let data = ResponseData::from(errors);
            Box::new(fut_ok(HttpResponse::BadRequest().json(data)))
        }
    }
}

enum AuthResult2 {
    Success(ResponseData),
    Invalid(ResponseData),
    ServerError,
}

impl From<AuthResult2> for Result<HttpResponse, WebError> {
    fn from(res: AuthResult2) -> Result<HttpResponse, WebError> {
        Ok(match res {
            AuthResult2::Success(data) => HttpResponse::Ok().json(data),
            AuthResult2::Invalid(data) => HttpResponse::BadRequest().json(data),
            AuthResult2::ServerError => HttpResponse::InternalServerError().json(""),
        })
    }
}

fn authenticate_user2(result: Result<(Option<UserModel>, Credentials), Error>) -> AuthResult2 {
    match result {
        Ok((user, credentials)) => {
            let auth = Authenticator { user, credentials };
            match auth.validate() {
                Ok(token) => AuthResult2::Success(ResponseData::from_token(Some(token))),
                Err(err) => AuthResult2::Invalid(ResponseData::from(err)),
            }
        }
        Err(_) => AuthResult2::ServerError,
    }
}

fn authenticate_user(
    result: Result<(Option<UserModel>, Credentials), Error>,
) -> Result<HttpResponse, WebError> {
    authenticate_user2(result).into()
}

pub fn build() -> App<AppState> {
    App::with_state(AppState::new())
        .prefix("/auth/jwt")
        .middleware(Logger::default())
        .resource("/create/", |r| {
            r.post().with_config(create, |((cfg, _),)| {
                cfg.limit(1024); // <- limit size of the payload
            })
        })
}

#[cfg(test)]
mod test {
    extern crate bytes;
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

    fn make_user(password_hash: &'static str) -> UserModel {
        UserModel {
            id: 123,
            username: "".to_string(),
            password: password_hash.to_string(),
            email: "".to_string(),
            is_active: true,
        }
    }

    fn make_creds(password: &'static str) -> Credentials {
        Credentials {
            username: "foo".to_string(),
            password: password.to_string(),
        }
    }

    #[test]
    fn test_authenticate_user_success() {
        let user = make_user(
            "pbkdf2_sha256$100000$Nk15JZg3MdZa$BKvnIMgDEAH1B6/ns9xw9PdQNP8Fq8rSHnrZ+8l0xCo=",
        );
        let find_result = Ok((Some(user.clone()), make_creds("zxcasdqwe123")));

        let response = authenticate_user(find_result);
        let expected_response = HttpResponse::Ok().json(json!({"token":create_token(user.id)}));

        assert_eq!(expected_response.body(), response.unwrap().body());
    }

    #[test]
    fn test_authenticate_user_no_user_found() {
        let response = authenticate_user(Ok((None, make_creds(""))));
        let expeted_response = HttpResponse::Ok()
            .json(json!({"non_field_errors":["Unable to log in with provided credentials."]}));

        assert_eq!(expeted_response.body(), response.unwrap().body());
    }

    #[test]
    fn test_authenticate_user_invalid_password() {
        let user = make_user(
            "pbkdf2_sha256$100000$Nk15JZg3MdZa$BKvnIMgDEAH1B6/ns9xw9PdQNP8Fq8rSHnrZ+8l0xCo=",
        );
        let response = authenticate_user(Ok((Some(user.clone()), make_creds("foo"))));
        let expected_response = HttpResponse::Ok()
            .json(json!({"non_field_errors":["Unable to log in with provided credentials."]}));

        assert_eq!(expected_response.body(), response.unwrap().body());
    }
}
