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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::builders::UserBuilder;
    use actix::prelude::*;
    use futures::future;

    #[test]
    fn not_found_err() {
        System::run(move || {
            let fut = crate::db::start().send(FindUserByName("foo".to_string()));

            actix::spawn(fut.then(|res| {
                assert_eq!("Err(NotFound)", format!("{:?}", res.unwrap()));

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .unwrap();
    }

    #[test]
    fn found() {
        let mut session = crate::tests::DbSession::new();
        let user = session.create_user(UserBuilder::default());

        System::run(move || {
            let fut = crate::db::start().send(FindUserByName(user.username.to_owned()));

            actix::spawn(fut.then(move |res| {
                assert_eq!(user, res.unwrap().unwrap());

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .unwrap();
    }
}
