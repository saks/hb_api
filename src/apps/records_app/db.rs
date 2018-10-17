use std::result;

use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;

use db::pagination::*;
use db::{models::Record as RecordModel, schema::records_record, DbExecutor};

pub type GetRecordsResult = result::Result<(Vec<RecordModel>, i64), Error>;

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

        let results = query.load::<(RecordModel, i64)>(&*connection)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records: Vec<RecordModel> = results.into_iter().map(|x| x.0).collect();

        Ok((records, total))
    }
}
