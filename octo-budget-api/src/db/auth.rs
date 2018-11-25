use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::result;

use crate::db::{models::AuthUser as UserModel, schema::auth_user, DbExecutor};

pub type FindUserResult = result::Result<UserModel, Error>;

pub struct FindUserMessage(pub String);

impl Message for FindUserMessage {
    type Result = FindUserResult;
}

impl Handler<FindUserMessage> for DbExecutor {
    type Result = FindUserResult;

    fn handle(&mut self, msg: FindUserMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &self.pool.get()?;
        let username = msg.0;

        auth_user::table
            .filter(auth_user::username.eq(&username))
            .first(connection)
            .map_err(|e| e.into())
    }
}
