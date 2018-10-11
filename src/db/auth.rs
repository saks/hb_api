use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::result;

use db::{models::AuthUser as UserModel, schema::auth_user, DbExecutor};

pub type FindUserResult = result::Result<UserModel, Error>;

pub struct FindUserMessage(pub String);

impl Message for FindUserMessage {
    type Result = FindUserResult;
}

impl Handler<FindUserMessage> for DbExecutor {
    type Result = FindUserResult;

    fn handle(&mut self, msg: FindUserMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0.get()?;
        let username = msg.0;

        auth_user::table
            .filter(auth_user::username.eq(&username))
            .first(connection)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use auth_user::dsl::*;
    // use diesel::insert;

    #[test]
    fn test_find_existing_user() {
        // let user = UserModel { email: "foo@foo.com",  };
        // use diesel::pg::PgConnection;

        // PgConnection::establish("")

        // auth_user::dsl::insert_into(auth_user::table).values(auth_user::username.eq("john")).execute(connection);
        // create user
        // find user
    }
}
