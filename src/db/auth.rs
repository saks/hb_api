use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::result;

use db::{models::AuthUser as UserModel, schema::auth_user, DbExecutor};

pub type FindResult = result::Result<UserModel, Error>;

pub struct Username(pub String);

impl Message for Username {
    type Result = FindResult;
}

impl Handler<Username> for DbExecutor {
    type Result = FindResult;

    fn handle(&mut self, msg: Username, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0.get()?;
        let username = msg.0;

        auth_user::table
            .filter(auth_user::username.eq(&username))
            .first(connection)
            .map_err(|e| e.into())
    }
}
