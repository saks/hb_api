use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::convert::Into;

use crate::db::{models::AuthUser, schema::auth_user, DbExecutor};

pub type MessageResult = Result<AuthUser, Error>;

pub struct FindUserById(pub i32);

impl Message for FindUserById {
    type Result = MessageResult;
}

impl Handler<FindUserById> for DbExecutor {
    type Result = MessageResult;

    fn handle(&mut self, msg: FindUserById, _: &mut Self::Context) -> Self::Result {
        let connection = &self.pool.get()?;

        auth_user::table
            .filter(auth_user::id.eq(msg.0))
            .first(connection)
            .map_err(Into::into)
    }
}
