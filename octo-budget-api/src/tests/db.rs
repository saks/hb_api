use bigdecimal::BigDecimal;
use chrono::naive::NaiveDateTime;
use diesel::*;

use crate::db::{
    builders::UserBuilder,
    models::{AuthUser, Budget, Record},
    ConnectionPool, PooledConnection,
};

pub struct DbSession {
    with_transaction: bool,
    pooled_conn: PooledConnection,
}

impl DbSession {
    pub fn new() -> Self {
        let pool = ConnectionPool::new();
        let pooled_conn = pool.conn();

        DbSession {
            with_transaction: false,
            pooled_conn,
        }
    }

    pub fn conn(&self) -> &PooledConnection {
        &self.pooled_conn
    }

    pub fn count_records(&self) -> i64 {
        use crate::db::schema::records_record::table as records;

        records.count().first(&self.pooled_conn).unwrap()
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
            .get_result::<Budget>(&self.pooled_conn)
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
            .get_result::<Record>(&self.pooled_conn)
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
            .get_result::<Record>(&self.pooled_conn)
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
                .get_result::<Record>(&self.pooled_conn)
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
            .first(&self.pooled_conn)
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
                .get_result::<Record>(&self.pooled_conn)
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
            .get_result(&self.pooled_conn)
            .unwrap()
    }
}

impl Drop for DbSession {
    fn drop(&mut self) {
        if self.with_transaction {
            return;
        }

        for table_name in ["auth_user", "records_record", "budgets_budget"].iter() {
            self.pooled_conn
                .execute(&format!("TRUNCATE TABLE {} CASCADE", table_name))
                .expect("Error executing TRUNCATE");
        }
    }
}
