use bigdecimal::BigDecimal;
use chrono::naive::NaiveDateTime;
use diesel::*;
use diesel::{Connection, PgConnection};

use crate::db::{
    builders::UserBuilder,
    models::{AuthUser, Budget, Record},
};

// #[macro_export]
// macro_rules! get_db_message_result {
//     ( $message:ident, $closure:expr ) => {{
//         System::run(|| {
//             Arbiter::spawn(crate::db::start().send($message).then(|res| {
//                 $closure(res.unwrap());
//                 System::current().stop();
//                 future::result(Ok(()))
//             }));
//         })
//         .expect("failed to start system");
//     }};
// }

// pub fn connection() -> PgConnection {
//     let connection = connection_without_transaction();
//
//     connection.begin_test_transaction().unwrap();
//     connection
// }

pub fn connection_without_transaction() -> PgConnection {
    let database_url = crate::config::DATABASE_URL.to_string();
    PgConnection::establish(&database_url).expect(&format!(
        "Failed to establish connection to URL: `{}'",
        database_url
    ))
}

// use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub struct DbSession {
    conn: PgConnection,
    with_transaction: bool,
    // pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbSession {
    pub fn new() -> Self {
        // let database_url = database_url_from_env("DATABASE_URL");
        // let manager = ConnectionManager::<PgConnection>::new(database_url.as_str());
        //
        // let pool = Pool::builder()
        //     .max_size(1) // max pool size
        //     .build(manager)
        //     .expect("Failed to create database connection pool.");

        DbSession {
            conn: connection_without_transaction(),
            with_transaction: false,
            // pool,
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

    pub fn count_records(&self) -> i64 {
        use crate::db::schema::records_record::table as records;

        records.count().first(&self.conn).unwrap()
    }

    pub fn create_budget(&mut self, budget: Budget) {
        use crate::db::schema::budgets_budget::dsl::*;

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

    pub fn create_record2(&mut self, id_of_the_user: i32) -> Record {
        use crate::db::schema::records_record::dsl::*;
        use diesel::*;
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
            .unwrap()
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

    pub fn create_records2(&self, id_of_the_user: i32, count: usize) -> Vec<Record> {
        use crate::db::schema::records_record::dsl::*;
        use diesel::*;

        let mut result = Vec::with_capacity(count);

        for _ in 0..count {
            let tags_list: Vec<String> = vec![];
            let record = insert_into(records_record)
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

            result.push(record);
        }

        result
    }

    pub fn find_record(&self, record_id: i32) -> Record {
        use crate::db::schema::records_record::table as records;
        use diesel::*;

        records
            .find(record_id)
            .first(&self.conn)
            .expect("failed to find record")
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

    pub fn create_user(&self, builder: UserBuilder) -> AuthUser {
        use crate::db::schema::auth_user::dsl::*;
        use diesel::*;

        let user = builder.finish();
        let new_password = djangohashers::make_password(&user.password);

        insert_into(auth_user)
            .values((
                username.eq(user.username),
                password.eq(&new_password),
                is_superuser.eq(user.is_superuser),
                is_active.eq(user.is_active),
                is_staff.eq(user.is_staff),
                email.eq(user.email),
                first_name.eq(user.first_name),
                last_name.eq(user.last_name),
                date_joined.eq(user.date_joined),
                tags.eq(user.tags),
            ))
            .get_result(&self.conn)
            .unwrap()
    }
}

impl Drop for DbSession {
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
