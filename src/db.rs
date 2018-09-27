use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use actix_web::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use r2d2;
use std::env;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct FindUser {
    pub username: String,
}

impl Message for FindUser {
    type Result = Result<models::AuthUser, Error>;
}

impl Handler<FindUser> for DbExecutor {
    type Result = Result<models::AuthUser, Error>;

    fn handle(&mut self, msg: FindUser, _: &mut Self::Context) -> Self::Result {
        let connection: &PgConnection = &self.0.get().unwrap();

        let mut results = schema::auth_user::table
            .filter(schema::auth_user::username.eq(&msg.username))
            .limit(1)
            .load::<models::AuthUser>(connection)
            .expect("Failed to load data from db");

        Ok(results.pop().unwrap())
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub mod models;
pub mod schema;

pub fn print_users() {
    use djangohashers::check_password;

    let connection = establish_connection();
    let results = schema::auth_user::table
        .filter(schema::auth_user::username.eq("we"))
        .limit(1)
        .load::<models::AuthUser>(&connection)
        .expect("Failed to load data from db");

    println!("Displaying {} users", results.len());
    for user in results {
        // if user.password.is_empty() {
        //     continue;
        // }
        // let pass = &format!("${}", user.password);
        println!(
            "id: {}, email: {}, username: {}, auth: {:?}",
            user.id,
            user.email,
            user.username,
            check_password("zxcasdqwe123", &user.password),
        );
    }
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
