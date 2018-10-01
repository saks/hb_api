use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;

use apps::auth::{check_password, create_token};
use db::{models::AuthUser as UserModel, schema::auth_user, DbExecutor};

#[derive(Deserialize, Debug)]
pub struct AuthenticateUser {
    pub username: String,
    pub password: String,
}

impl Message for AuthenticateUser {
    type Result = Result<String, Error>;
}

impl Handler<AuthenticateUser> for DbExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: AuthenticateUser, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0.get()?;

        let user: UserModel = auth_user::table
            .filter(auth_user::username.eq(&msg.username))
            .first(connection)?;

        check_password(&msg.password, &user.password).and_then(|_| create_token(user.id))
    }
}
