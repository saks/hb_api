use chrono::naive::NaiveDateTime;
use diesel::{Connection, PgConnection};
use std::env;

use crate::db::models::{AuthUser, Record};

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

    // pub fn conn(&self) -> &PgConnection {
    //     &self.conn
    // }

    pub fn create_records(&mut self, id_of_the_user: i32, count: u32) {
        use crate::db::schema::records_record::dsl::*;
        use bigdecimal::BigDecimal;
        use diesel::*;

        for _ in 0..count {
            let tags_list: Vec<String> = vec![];
            let amount_num = BigDecimal::from(123.12f64);
            let _: Record = insert_into(records_record)
                .values((
                    amount.eq(amount_num),
                    amount_currency.eq("CAD"),
                    created_at.eq(NaiveDateTime::from_timestamp(0, 0)),
                    tags.eq(tags_list),
                    transaction_type.eq("EXP"),
                    user_id.eq(id_of_the_user),
                ))
                .get_result(&self.conn)
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

        self.conn
            .execute("TRUNCATE TABLE auth_user CASCADE")
            .expect("Error executing TRUNCATE");
    }
}
