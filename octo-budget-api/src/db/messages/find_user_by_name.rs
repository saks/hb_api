use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;
use std::convert::Into;

use crate::db::{models::AuthUser, schema::auth_user};

pub type FindUserResult = Result<AuthUser, Error>;

pub struct FindUserByName(String);

impl FindUserByName {
    pub fn new(username: impl Into<String>) -> Self {
        Self(username.into())
    }

    pub fn query(self, pool: &crate::db::PgPool) -> FindUserResult {
        let connection = pool.get()?;
        let username = self.0;

        auth_user::table
            .filter(auth_user::username.eq(&username))
            .first(&connection)
            .map_err(Into::into)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::db::builders::UserBuilder;
//     use actix::prelude::*;
//     use futures::future;
//
//     #[test]
//     fn not_found_err() {
//         System::run(move || {
//             let fut = crate::db::start().send(FindUserByName("foo".to_string()));
//
//             actix::spawn(fut.then(|res| {
//                 assert_eq!("Err(NotFound)", format!("{:?}", res.unwrap()));
//
//                 System::current().stop();
//                 future::result(Ok(()))
//             }));
//         })
//         .unwrap();
//     }
//
//     #[test]
//     fn found() {
//         let session = crate::tests::DbSession::new();
//         let user = session.create_user(UserBuilder::default());
//
//         System::run(move || {
//             let fut = crate::db::start().send(FindUserByName(user.username.to_owned()));
//
//             actix::spawn(fut.then(move |res| {
//                 assert_eq!(user, res.unwrap().unwrap());
//
//                 System::current().stop();
//                 future::result(Ok(()))
//             }));
//         })
//         .unwrap();
//     }
// }
