use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use actix_web::{error, Error};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use r2d2;
use std::env;

pub mod models;
pub mod schema;

// start of AuthenticateUser
use auth::create_token;
use djangohashers;

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
        let connection: &PgConnection = &self.0.get().unwrap();

        let mut results = schema::auth_user::table
            .filter(schema::auth_user::username.eq(&msg.username))
            .limit(1)
            .load::<models::AuthUser>(connection)
            .expect("Failed to load data from db");

        let user = results.pop().unwrap();

        match djangohashers::check_password(&msg.password, &user.password) {
            Ok(true) => Ok(create_token(user.id).unwrap()),
            _ => Err(error::ErrorUnauthorized("foo")),
        }
    }
}

// end of AuthenticateUser

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub fn db_executor() -> Addr<DbExecutor> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    SyncArbiter::start(3, move || DbExecutor(pool.clone()))
}
