use crate::db::{models::Record as RecordModel, pagination::*, schema::records_record, DbExecutor};
use actix::{Handler, Message};
use failure::Error;
use std::result;

use crate::apps2::index_response::Data;

pub type ResponseData = Data<RecordModel>;
pub type GetRecordsResult = result::Result<ResponseData, Error>;

#[derive(Clone)]
pub struct GetRecords {
    pub user_id: i32,
    pub page: i64,
    pub per_page: i64,
}

impl Message for GetRecords {
    type Result = GetRecordsResult;
}

impl Handler<GetRecords> for DbExecutor {
    type Result = GetRecordsResult;

    fn handle(&mut self, msg: GetRecords, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

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
    use crate::db::builders::UserBuilder;
    use crate::{get_db_message_result, tests::DbSession};
    use actix::{Arbiter, System};
    use futures::{future, Future};

    #[test]
    fn test_empty_result() {
        System::run(move || {
            let message = GetRecords {
                page: 1,
                per_page: 10,
                user_id: 123,
            };
            let fut = crate::db::start().send(message);

            actix::spawn(fut.then(|res| {
                let data: ResponseData = res.unwrap().unwrap();

                assert_eq!(0, data.total);
                assert_eq!(false, data.next);
                assert_eq!(false, data.previous);
                assert!(data.results.is_empty());

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to start system");
    }

    #[test]
    fn test_first_page_result() {
        let mut session = DbSession::new();
        let user = session.create_user(UserBuilder::default().password("dummy password"));
        session.create_records(user.id, 12);

        let message = GetRecords {
            page: 1,
            per_page: 10,
            user_id: user.id,
        };

        System::run(move || {
            let fut = crate::db::start().send(message);

            actix::spawn(fut.then(|res| {
                let data: ResponseData = res.unwrap().unwrap();

                assert_eq!(12, data.total);
                assert_eq!(false, data.previous);
                assert_eq!(true, data.next);
                assert_eq!(10, data.results.len());

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to start system");
    }

    #[test]
    fn test_second_page_result() {
        let mut session = DbSession::new();
        let user = session.create_user(UserBuilder::default().password("dummy password"));
        session.create_records(user.id, 12);

        let message = GetRecords {
            page: 2,
            per_page: 10,
            user_id: user.id,
        };

        System::run(move || {
            let fut = crate::db::start().send(message);

            actix::spawn(fut.then(|res| {
                let data: ResponseData = res.unwrap().unwrap();

                assert_eq!(12, data.total);
                assert_eq!(true, data.previous);
                assert_eq!(false, data.next);
                assert_eq!(2, data.results.len());

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to start system");
    }

    #[test]
    fn test_records_for_correct_user() {
        let mut session = DbSession::new();
        let user1 = session.create_user(
            UserBuilder::default()
                .username("user1")
                .password("dummy password"),
        );
        session.create_records(user1.id, 2);

        let user2 = session.create_user(
            UserBuilder::default()
                .username("user2")
                .password("dummy password"),
        );
        session.create_records(user2.id, 2);

        let message = GetRecords {
            page: 1,
            per_page: 10,
            user_id: user1.id,
        };

        System::run(move || {
            let fut = crate::db::start().send(message);

            actix::spawn(fut.then(|res| {
                let data: ResponseData = res.unwrap().unwrap();

                assert_eq!(2, data.total);
                assert_eq!(false, data.previous);
                assert_eq!(false, data.next);
                assert_eq!(2, data.results.len());

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to start system");
    }
}
