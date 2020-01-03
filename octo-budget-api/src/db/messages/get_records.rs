use crate::db::{
    models::Record as RecordModel, pagination::*, schema::records_record, DatabaseQuery,
    PooledConnection,
};
use failure::Error;

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

    fn execute(&self, connection: PooledConnection) -> Result<Self::Data, Error> {
        use diesel::prelude::*;

        let query = records_record::table
            .select(records_record::all_columns)
            .filter(records_record::user_id.eq(self.user_id))
            .order(records_record::created_at.desc())
            .paginate(self.page)
            .per_page(self.per_page);

        let query_results = query.load::<(RecordModel, i64)>(&*connection)?;

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
mod tests {
    use super::*;
    use crate::{
        db::{builders::UserBuilder, ConnectionPool},
        tests::DbSession,
    };

    #[actix_rt::test]
    async fn test_empty_result() {
        let conn_pool = ConnectionPool::new();
        let query = GetRecords {
            page: 1,
            per_page: 10,
            user_id: 123,
        };

        let data = conn_pool
            .execute(query)
            .await
            .expect("failed to get records");
        assert_eq!(0, data.total);
        assert_eq!(false, data.next);
        assert_eq!(false, data.previous);
        assert!(data.results.is_empty());
    }

    #[actix_rt::test]
    async fn test_first_page_result() {
        let mut session = DbSession::new();
        let user = session.create_user(UserBuilder::default().password("dummy password"));
        session.create_records(user.id, 12);

        let query = GetRecords {
            page: 1,
            per_page: 10,
            user_id: user.id,
        };
        let conn_pool = ConnectionPool::new();

        let data = conn_pool
            .execute(query)
            .await
            .expect("failed to get records");

        assert_eq!(12, data.total);
        assert_eq!(false, data.previous);
        assert_eq!(true, data.next);
        assert_eq!(10, data.results.len());
    }

    #[actix_rt::test]
    async fn test_second_page_result() {
        let mut session = DbSession::new();
        let user = session.create_user(UserBuilder::default().password("dummy password"));
        session.create_records(user.id, 12);

        let query = GetRecords {
            page: 2,
            per_page: 10,
            user_id: user.id,
        };
        let conn_pool = ConnectionPool::new();

        let data = conn_pool
            .execute(query)
            .await
            .expect("failed to get records");

        assert_eq!(12, data.total);
        assert_eq!(true, data.previous);
        assert_eq!(false, data.next);
        assert_eq!(2, data.results.len());
    }

    #[actix_rt::test]
    async fn test_records_for_correct_user() {
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

        let conn_pool = ConnectionPool::new();
        let query = GetRecords {
            page: 1,
            per_page: 10,
            user_id: user1.id,
        };

        let data = conn_pool
            .execute(query)
            .await
            .expect("failed to get records");

        assert_eq!(2, data.total);
        assert_eq!(false, data.previous);
        assert_eq!(false, data.next);
        assert_eq!(2, data.results.len());
    }
}
