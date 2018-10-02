use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::result;

use apps::auth_app::Credentials;
use db::{models::AuthUser as UserModel, schema::auth_user, DbExecutor};

type AuthResult = result::Result<(Option<UserModel>, Credentials), Error>;

impl Message for Credentials {
    type Result = AuthResult;
}

impl Handler<Credentials> for DbExecutor {
    type Result = AuthResult;

    fn handle(&mut self, msg: Credentials, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0.get()?;
        let username = msg.username.clone();

        let user = auth_user::table
            .filter(auth_user::username.eq(&username))
            .first(connection)
            .ok();

        Ok((user, msg))
    }
}
