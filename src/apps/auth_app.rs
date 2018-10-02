use std::convert::Into;

use actix_web::middleware::Logger;
use actix_web::{App, AsyncResponder, FutureResponse, HttpResponse, Json, State};
use futures::future::{ok as fut_ok, Future};

use apps::AppState;
use db::models::AuthUser as UserModel;

pub fn create_token(user_id: i32) -> Result<String, ValidationError> {
    use config;
    use frank_jwt::{encode, Algorithm};
    use time::{now_utc, Duration};

    let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
    let payload = json!({ "user_id": user_id });
    let header = json!({ "exp": exp });
    let secret = &config::AUTH_TOKEN_SECRET.to_string();

    encode(header, secret, &payload, Algorithm::HS256).map_err(|_| ValidationError::UnknownError)
}

pub fn check_password(password: &str, hash: &str) -> Result<(), ValidationError> {
    use djangohashers;

    match djangohashers::check_password(password, hash) {
        Ok(true) => Ok(()),
        _ => Err(ValidationError::AuthenticationFailed),
    }
}

#[derive(Debug, Fail)]
pub enum ValidationError {
    #[fail(display = "This field may not be blank.")]
    UsernameIsEmpty,
    #[fail(display = "This field is required.")]
    UsernameIsNotPresent,
    #[fail(display = "This field may not be blank.")]
    PasswordIsEmpty,
    #[fail(display = "This field is required.")]
    PasswordIsNotPresent,
    #[fail(display = "Unable to log in with provided credentials.")]
    AuthenticationFailed,
    #[fail(display = "Unknown error.")]
    UnknownError,
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

impl From<ValidationError> for ResponseData {
    fn from(err: ValidationError) -> ResponseData {
        let mut data = ResponseData::default();
        match err {
            ValidationError::UsernameIsEmpty => data.username_errors.push(err.to_string()),
            ValidationError::UsernameIsNotPresent => data.username_errors.push(err.to_string()),
            ValidationError::PasswordIsEmpty => data.password_errors.push(err.to_string()),
            ValidationError::PasswordIsNotPresent => data.password_errors.push(err.to_string()),
            ValidationError::AuthenticationFailed => data.non_field_errors.push(err.to_string()),
            ValidationError::UnknownError => data.non_field_errors.push(err.to_string()),
        }
        data
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct AuthForm {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl AuthForm {
    pub fn validate(self) -> Result<Credentials, ValidationError> {
        let AuthForm { username, password } = self;

        if username.is_none() {
            return Err(ValidationError::UsernameIsNotPresent);
        } else if let Some(val) = username.clone() {
            if val.is_empty() {
                return Err(ValidationError::UsernameIsEmpty);
            }
        }

        if password.is_none() {
            return Err(ValidationError::PasswordIsNotPresent);
        } else if let Some(val) = password.clone() {
            if val.is_empty() {
                return Err(ValidationError::PasswordIsEmpty);
            }
        }

        let username = username.unwrap();
        let password = password.unwrap();

        Ok(Credentials { username, password })
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
    fn validate(self) -> Result<String, ValidationError> {
        let Authenticator { credentials, user } = self;

        match user {
            Some(user) => {
                let Credentials { password, .. } = credentials;
                check_password(&password, &user.password).and_then(|_| create_token(user.id))
            }
            None => Err(ValidationError::AuthenticationFailed),
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
            .and_then(|res| match res {
                Ok((user, credentials)) => {
                    let auth = Authenticator { user, credentials };
                    match auth.validate() {
                        Ok(token) => {
                            let data = ResponseData::from_token(Some(token));
                            Ok(HttpResponse::Ok().json(data))
                        }
                        Err(err) => {
                            let data = ResponseData::from(err);
                            Ok(HttpResponse::BadRequest().json(data))
                        }
                    }
                }
                Err(_) => Ok(HttpResponse::InternalServerError().json("")),
            })
            .responder(),
        Err(err) => {
            let data = ResponseData::from(err);
            Box::new(fut_ok(HttpResponse::BadRequest().json(data)))
        }
    }
}

// fn xxx(res: Result<Option<UserModel>, Error>) -> Result<HttpResponse, WebError> {
//     match res {
//         Ok(maybe_user) => Authenticator::new(),
//     }
//     Ok(HttpResponse::Ok().json("from f"))
// }

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
