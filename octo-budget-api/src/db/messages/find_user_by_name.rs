use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::convert::Into;

use crate::db::{models::AuthUser, schema::auth_user, DbExecutor};

pub type FindUserResult = Result<AuthUser, Error>;

pub struct FindUserByName(pub String);

impl Message for FindUserByName {
    type Result = FindUserResult;
}

impl Handler<FindUserByName> for DbExecutor {
    type Result = FindUserResult;

    fn handle(&mut self, msg: FindUserByName, _: &mut Self::Context) -> Self::Result {
        let connection = &self.pool.get()?;
        let username = msg.0;

        auth_user::table
            .filter(auth_user::username.eq(&username))
            .first(connection)
            .map_err(Into::into)
    }
}
