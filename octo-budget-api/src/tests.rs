use bigdecimal::BigDecimal;
use chrono::naive::NaiveDateTime;
use diesel::{Connection, PgConnection};
use std::env;

use crate::db::models::{AuthUser, Budget, Record};

fn database_url_from_env(env_var_name: &str) -> String {
    match env::var(env_var_name) {
        Ok(val) => {
            println!(r#"cargo:rustc-cfg=feature="backend_specific_database_url""#);
            val
        }
        _ => env::var("DATABASE_URL").expect("DATABASE_URL must be set in order to run tests"),
    }
}

// pub fn connection() -> PgConnection {
//     let connection = connection_without_transaction();
//
//     connection.begin_test_transaction().unwrap();
//     connection
// }

pub fn connection_without_transaction() -> PgConnection {
    let database_url = database_url_from_env("DATABASE_URL");
    PgConnection::establish(&database_url).unwrap()
}

pub struct Session {
    conn: PgConnection,
    with_transaction: bool,
}

impl Session {
    pub fn new() -> Self {
        Self {
            conn: connection_without_transaction(),
            with_transaction: false,
        }
    }

    // pub fn with_transaction() -> Self {
    //     Self {
    //         conn: connection(),
    //         with_transaction: true,
    //     }
    // }

    pub fn conn(&self) -> &PgConnection {
        &self.conn
    }

    pub fn create_budget(&mut self, budget: Budget) {
        use crate::db::schema::budgets_budget::dsl::*;
        use diesel::*;

        insert_into(budgets_budget)
            .values((
                name.eq(budget.name),
                amount.eq(budget.amount),
                amount_currency.eq(budget.amount_currency),
                start_date.eq(budget.start_date),
                tags.eq(budget.tags),
                tags_type.eq(budget.tags_type),
                user_id.eq(budget.user_id),
            ))
            .get_result::<Budget>(&self.conn)
            .unwrap();
    }

    pub fn create_record(&mut self, record: Record) {
        use crate::db::schema::records_record::dsl::*;
        use diesel::*;

        insert_into(records_record)
            .values((
                amount.eq(record.amount),
                amount_currency.eq(record.amount_currency),
                created_at.eq(record.created_at),
                tags.eq(record.tags),
                transaction_type.eq(record.transaction_type),
                user_id.eq(record.user_id),
            ))
            .get_result::<Record>(&self.conn)
            .unwrap();
    }

    pub fn create_records(&mut self, id_of_the_user: i32, count: u32) {
        use crate::db::schema::records_record::dsl::*;
        use diesel::*;

        for _ in 0..count {
            let tags_list: Vec<String> = vec![];
            insert_into(records_record)
                .values((
                    amount.eq(BigDecimal::from(123.12f64)),
                    amount_currency.eq("CAD"),
                    created_at.eq(NaiveDateTime::from_timestamp(0, 0)),
                    tags.eq(tags_list),
                    transaction_type.eq("EXP"),
                    user_id.eq(id_of_the_user),
                ))
                .get_result::<Record>(&self.conn)
                .unwrap();
        }
    }

    pub fn create_user(
        &mut self,
        username_str: &'static str,
        password_str: &'static str,
    ) -> AuthUser {
        use crate::db::schema::auth_user::dsl::*;
        use diesel::*;
        use djangohashers;

        let new_password = djangohashers::make_password(password_str);

        insert_into(auth_user)
            .values((
                username.eq(username_str),
                password.eq(&new_password),
                is_superuser.eq(false),
                is_active.eq(true),
                is_staff.eq(false),
                email.eq(""),
                first_name.eq(""),
                last_name.eq(""),
                date_joined.eq(NaiveDateTime::from_timestamp(0, 0)),
            ))
            .get_result(&self.conn)
            .unwrap()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.with_transaction {
            return;
        }

        for table_name in ["auth_user", "records_record", "budgets_budget"].iter() {
            self.conn
                .execute(&format!("TRUNCATE TABLE {} CASCADE", table_name))
                .expect("Error executing TRUNCATE");
        }
    }
}

// use crate::db::DbExecutor;
// use actix::{Arbiter, System};
// use futures::{future, Future};

#[macro_export]
macro_rules! get_message_result {
    ( $message:ident, $closure:expr ) => {{
        System::run(|| {
            Arbiter::spawn(DbExecutor::new().send($message).then(|res| {
                $closure(res.unwrap());
                System::current().stop();
                future::result(Ok(()))
            }));
        });
    };};
}
