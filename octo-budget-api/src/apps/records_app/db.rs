use std::result;

use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;

use super::ResponseData;
use crate::apps::index_response::Data;
use crate::db::{models::Record as RecordModel, pagination::*, schema::records_record, DbExecutor};

pub type GetRecordsResult = result::Result<ResponseData, Error>;

#[derive(Clone)]
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
        let connection = &self.pool.get()?;

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

        Ok(Data {
            total,
            results,
            next,
            previous,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db::DbExecutor, get_db_message_result, tests::DbSession};
    use actix::{Arbiter, System};
    use futures::{future, Future};

    #[test]
    fn test_empty_result() {
        let message = GetRecordsMessage {
            page: 1,
            per_page: 10,
            user_id: 123,
        };

        get_db_message_result!(message, |res: GetRecordsResult| {
            let data: ResponseData = res.unwrap();

            assert_eq!(0, data.total);
            assert_eq!(false, data.next);
            assert_eq!(false, data.previous);
            assert!(data.results.is_empty());
        });
    }

    #[test]
    fn test_first_page_result() {
        let mut session = DbSession::new();
        let user = session.create_user("ok auth user", "dummy password");
        session.create_records(user.id, 12);

        let message = GetRecordsMessage {
            page: 1,
            per_page: 10,
            user_id: user.id,
        };

        get_db_message_result!(message, |res: GetRecordsResult| {
            let data: ResponseData = res.unwrap();

            assert_eq!(12, data.total);
            assert_eq!(false, data.previous);
            assert_eq!(true, data.next);
            assert_eq!(10, data.results.len());
        });
    }

    #[test]
    fn test_second_page_result() {
        let mut session = DbSession::new();
        let user = session.create_user("ok auth user", "dummy password");
        session.create_records(user.id, 12);

        let message = GetRecordsMessage {
            page: 2,
            per_page: 10,
            user_id: user.id,
        };

        get_db_message_result!(message, |res: GetRecordsResult| {
            let data: ResponseData = res.unwrap();

            assert_eq!(12, data.total);
            assert_eq!(true, data.previous);
            assert_eq!(false, data.next);
            assert_eq!(2, data.results.len());
        });
    }

    #[test]
    fn test_records_for_correct_user() {
        let mut session = DbSession::new();
        let user1 = session.create_user("user1", "dummy password");
        session.create_records(user1.id, 2);

        let user2 = session.create_user("user2", "dummy password");
        session.create_records(user2.id, 2);

        let message = GetRecordsMessage {
            page: 1,
            per_page: 10,
            user_id: user1.id,
        };

        let msg = message.clone();
        get_db_message_result!(message, move |res: GetRecordsResult| {
            let data: ResponseData = res.unwrap();

            assert_eq!(2, data.total);
            assert_eq!(false, data.previous);
            assert_eq!(false, data.next);
            assert_eq!(2, data.results.len());

            assert!(data.results.into_iter().all(|r| r.user_id == msg.user_id));
        });
    }
}
