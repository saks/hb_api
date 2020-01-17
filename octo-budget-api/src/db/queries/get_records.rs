use crate::db::{
    models::Record as RecordModel, pagination::*, schema::records_record, DatabaseQuery,
    PooledConnection,
};
use crate::errors::DbResult;

use crate::apps::index_response::Data;

pub type ResponseData = Data<RecordModel>;

#[derive(Clone)]
pub struct GetRecords {
    pub user_id: i32,
    pub page: i64,
    pub per_page: i64,
}

impl DatabaseQuery for GetRecords {
    type Data = ResponseData;

    fn execute(&self, connection: PooledConnection) -> DbResult<Self::Data> {
        use diesel::prelude::*;

        let query = records_record::table
            .select(records_record::all_columns)
            .filter(records_record::user_id.eq(self.user_id))
            .order(records_record::created_at.desc())
            .paginate(self.page)
            .per_page(self.per_page);

        let query_results = query.load::<(RecordModel, i64)>(&connection)?;

        let total = query_results.get(0).map(|x| x.1).unwrap_or(0);
        let total_pages = (total as f64 / self.per_page as f64).ceil() as i64;

        let results = query_results.into_iter().map(|x| x.0).collect();

        let previous = self.page > 1;
        let next = self.page < total_pages;

        Ok(Data {
            total,
            results,
            next,
            previous,
        })
    }
}

#[cfg(test)]
mod tests;
