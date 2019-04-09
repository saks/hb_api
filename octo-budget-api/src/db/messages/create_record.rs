use std::result;

use actix::{Handler, Message};
use bigdecimal::BigDecimal;
use chrono::{Utc, NaiveDateTime};
use failure::Error;
use octo_budget_lib::auth_token::AuthToken;

use crate::apps::forms::record::FormData;
use crate::db::DbExecutor;

pub type CreateRecordResult = result::Result<(), Error>;

pub struct CreateRecord {
    amount: BigDecimal,
    amount_currency: String,
    created_at: NaiveDateTime,
    tags: Vec<String>,
    transaction_type: String,
    user_id: i32,
}

impl CreateRecord {
    pub fn new(data: &FormData, token: &AuthToken) -> Self {
        let created_at = Utc::now().naive_local();

        Self {
            amount: data.amount.clone(),
            amount_currency: data.amount_currency.clone(),
            tags: data.tags.clone(),
            transaction_type: data.transaction_type.clone(),
            user_id: token.user_id,
            created_at,
        }
    }
}

impl Message for CreateRecord {
    type Result = CreateRecordResult;
}

impl Handler<CreateRecord> for DbExecutor {
    type Result = CreateRecordResult;

    fn handle(&mut self, msg: CreateRecord, _: &mut Self::Context) -> Self::Result {
        use crate::db::schema::records_record::dsl::*;
        use diesel::prelude::*;
        use diesel::*;

        let connection = &self.pool.get()?;

        insert_into(records_record)
            .values((
                amount.eq(msg.amount),
                amount_currency.eq(msg.amount_currency),
                created_at.eq(msg.created_at),
                tags.eq(msg.tags),
                transaction_type.eq(msg.transaction_type),
                user_id.eq(msg.user_id),
            ))
            .execute(connection)?;

        Ok(())
    }
}

// TODO: add tests
