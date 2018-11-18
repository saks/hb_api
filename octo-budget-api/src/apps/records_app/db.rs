use std::result;

use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;

use super::response_data::ResponseData;
use crate::db::pagination::*;
use crate::db::{models::Record as RecordModel, schema::records_record, DbExecutor};

pub type GetRecordsResult = result::Result<ResponseData, Error>;

pub struct GetRecordsMessage {
    pub user_id: i32,
    pub page: i64,
    pub per_page: i64,
}

impl Message for GetRecordsMessage {
    type Result = GetRecordsResult;
}

impl Handler<GetRecordsMessage> for DbExecutor {
    type Result = GetRecordsResult;

    fn handle(&mut self, msg: GetRecordsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0.get()?;

        let query = records_record::table
            .select(records_record::all_columns)
            .filter(records_record::user_id.eq(msg.user_id))
            .order(records_record::created_at.desc())
            .paginate(msg.page)
            .per_page(msg.per_page);

        let query_results = query.load::<(RecordModel, i64)>(&*connection)?;

        let total = query_results.get(0).map(|x| x.1).unwrap_or(0);
        let total_pages = (total as f64 / msg.per_page as f64).ceil() as i64;

        let results = query_results.into_iter().map(|x| x.0).collect();

        let previous = msg.page > 1;
        let next = msg.page < total_pages;

        Ok(ResponseData {
            total,
            results,
            next,
            previous,
        })
    }
}
