use std::result;

use actix::{Handler, Message};
use diesel::prelude::*;
use failure::Error;

use super::ResponseData;
use crate::apps::index_response::Data;
use crate::db::{
    models::{Budget, SerializedBudget},
    pagination::*,
    schema::budgets_budget,
    DbExecutor,
};

pub type GetBudgetsResult = result::Result<ResponseData, Error>;

#[derive(Clone)]
pub struct GetBudgetsMessage {
    pub user_id: i32,
    pub page: i64,
    pub per_page: i64,
}

impl Message for GetBudgetsMessage {
    type Result = GetBudgetsResult;
}

impl Handler<GetBudgetsMessage> for DbExecutor {
    type Result = GetBudgetsResult;

    fn handle(&mut self, msg: GetBudgetsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &self.pool.get()?;
        handle(&msg, &*connection)
    }
}

fn serialize_budget(budget: Budget, conn: &PgConnection) -> Result<SerializedBudget, Error> {
    use crate::db::schema::records_record;
    use bigdecimal::BigDecimal;
    use chrono::{Datelike, Local};
    use diesel::dsl::sum;

    let first_month_day = Local::now().naive_local().with_day0(0).unwrap();
    let query = records_record::table
        .select(sum(records_record::amount))
        .filter(
            records_record::user_id.eq(budget.user_id).and(
                records_record::transaction_type
                    .eq("EXP")
                    .and(records_record::created_at.ge(first_month_day)),
            ),
        );

    let x = query.first::<(Option<BigDecimal>)>(conn);

    println!("today: {:#?}", x);
    // today = date.today()
    // first_month_day = date(today.year, today.month, 1)
    // spent = Record.objects.filter(user=self.user,
    //                               transaction_type='EXP',
    //                               created_at__gte=first_month_day)
    // if self.tags_type == 'INCL' and self.tags:
    //     spent = spent.filter(tags__overlap=self.tags)
    // if self.tags_type == 'EXCL' and self.tags:
    //     spent = spent.exclude(tags__overlap=self.tags)
    // spent = spent.aggregate(spent=Sum('amount'))

    Ok(SerializedBudget::default())
}

fn get_page_of_budgets(
    msg: &GetBudgetsMessage,
    conn: &PgConnection,
) -> Result<(Vec<Budget>, i64), Error> {
    let query = budgets_budget::table
        .select(budgets_budget::all_columns)
        .filter(budgets_budget::user_id.eq(msg.user_id))
        .order(budgets_budget::name.asc())
        .paginate(msg.page)
        .per_page(msg.per_page);

    let query_results = query.load::<(Budget, i64)>(conn)?;

    let total = query_results.get(0).map(|x| x.1).unwrap_or(0);

    let results: Vec<Budget> = query_results.into_iter().map(|x| x.0).collect();

    Ok((results, total))
}

fn handle(msg: &GetBudgetsMessage, conn: &PgConnection) -> GetBudgetsResult {
    let (results, total) = get_page_of_budgets(&msg, conn)?;
    let total_pages = (total as f64 / msg.per_page as f64).ceil() as i64;

    let results = results
        .into_iter()
        .map(|budget| serialize_budget(budget, conn))
        .collect::<Result<Vec<SerializedBudget>, Error>>()?;

    let previous = msg.page > 1;
    let next = msg.page < total_pages;

    Ok(Data {
        total,
        results,
        next,
        previous,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::Session;

    #[test]
    fn test_empty_result() {
        let message = GetBudgetsMessage {
            page: 1,
            per_page: 10,
            user_id: 123,
        };
        let session = Session::new();

        let data = handle(&message, session.conn()).unwrap();

        assert_eq!(0, data.total);
        assert_eq!(false, data.next);
        assert_eq!(false, data.previous);
        assert!(data.results.is_empty());
    }

    #[test]
    fn test_first_page_result() {
        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        session.create_budget(user.id, vec!["foo", "bar"]);

        let message = GetBudgetsMessage {
            page: 1,
            per_page: 10,
            user_id: user.id,
        };
        let data = handle(&message, session.conn()).unwrap();

        assert_eq!(1, data.total);
        assert_eq!(false, data.previous);
        assert_eq!(false, data.next);
        assert_eq!(1, data.results.len());
    }

    // #[test]
    // fn test_second_page_result() {
    //     let mut session = Session::new();
    //     let user = session.create_user("ok auth user", "dummy password");
    //     session.create_records(user.id, 12);
    //
    //     let message = GetRecordsMessage {
    //         page: 2,
    //         per_page: 10,
    //         user_id: user.id,
    //     };
    //
    //     get_message_result!(message, |res: GetRecordsResult| {
    //         let data: ResponseData = res.unwrap();
    //
    //         assert_eq!(12, data.total);
    //         assert_eq!(true, data.previous);
    //         assert_eq!(false, data.next);
    //         assert_eq!(2, data.results.len());
    //     });
    // }
    //
    // #[test]
    // fn test_records_for_correct_user() {
    //     let mut session = Session::new();
    //     let user1 = session.create_user("user1", "dummy password");
    //     session.create_records(user1.id, 2);
    //
    //     let user2 = session.create_user("user2", "dummy password");
    //     session.create_records(user2.id, 2);
    //
    //     let message = GetRecordsMessage {
    //         page: 1,
    //         per_page: 10,
    //         user_id: user1.id,
    //     };
    //
    //     let msg = message.clone();
    //     get_message_result!(message, move |res: GetRecordsResult| {
    //         let data: ResponseData = res.unwrap();
    //
    //         assert_eq!(2, data.total);
    //         assert_eq!(false, data.previous);
    //         assert_eq!(false, data.next);
    //         assert_eq!(2, data.results.len());
    //
    //         assert!(data.results.into_iter().all(|r| r.user_id == msg.user_id));
    //     });
    // }
}
