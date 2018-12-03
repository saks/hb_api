use std::result;

use actix::{Handler, Message};
use bigdecimal::{BigDecimal, Zero};
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

fn select_budget_amount(budget: &Budget, conn: &PgConnection) -> Result<BigDecimal, Error> {
    use crate::db::schema::records_record;
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

    Ok(query
        .first::<(Option<BigDecimal>)>(conn)?
        .unwrap_or_else(BigDecimal::zero))
}

fn serialize_budget(budget: Budget, conn: &PgConnection) -> Result<SerializedBudget, Error> {
    use crate::db::schema::records_record;
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
    use crate::db::models::{BudgetBuilder, RecordBuilder};
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

        for _ in 0..12 {
            session.create_budget(BudgetBuilder::default().user_id(user.id).finish());
        }

        let message = GetBudgetsMessage {
            page: 1,
            per_page: 10,
            user_id: user.id,
        };
        let data = handle(&message, session.conn()).unwrap();

        assert_eq!(12, data.total);
        assert_eq!(false, data.previous);
        assert_eq!(true, data.next);
        assert_eq!(10, data.results.len());
    }

    #[test]
    fn test_second_page_result() {
        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        for _ in 0..12 {
            session.create_budget(BudgetBuilder::default().user_id(user.id).finish());
        }

        let message = GetBudgetsMessage {
            page: 2,
            per_page: 10,
            user_id: user.id,
        };
        let data = handle(&message, session.conn()).unwrap();

        assert_eq!(12, data.total);
        assert_eq!(true, data.previous);
        assert_eq!(false, data.next);
        assert_eq!(2, data.results.len());
    }

    #[test]
    fn test_records_for_correct_user() {
        let mut session = Session::new();
        let user1 = session.create_user("user1", "dummy password");
        session.create_budget(BudgetBuilder::default().user_id(user1.id).finish());

        let user2 = session.create_user("user2", "dummy password");
        session.create_budget(BudgetBuilder::default().user_id(user2.id).finish());

        let message = GetBudgetsMessage {
            page: 1,
            per_page: 10,
            user_id: user1.id,
        };
        let data = handle(&message, session.conn()).unwrap();

        assert_eq!(1, data.total);
        assert_eq!(false, data.previous);
        assert_eq!(false, data.next);
        assert_eq!(1, data.results.len());
    }

    #[test]
    fn amount_aggregation_without_tags() {
        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        let budget = BudgetBuilder::default().user_id(user.id).finish();

        let record = RecordBuilder::default()
            .user_id(user.id)
            .transaction_type("EXP");

        session.create_record(record.clone().amount(1.0).finish());
        session.create_record(record.clone().amount(2.0).finish());
        session.create_record(record.clone().amount(4.0).finish());

        let amount = select_budget_amount(&budget, session.conn()).unwrap();

        assert_eq!(BigDecimal::from(7.0f64), amount);
    }

    #[test]
    fn amount_aggregation_with_including_tags() {
        use bigdecimal::One;

        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        let budget = BudgetBuilder::default()
            .user_id(user.id)
            .tags_type("INCL")
            .finish();

        let record = RecordBuilder::default()
            .user_id(user.id)
            .transaction_type("EXP")
            .amount(1.0);

        session.create_record(record.clone().tags(vec!["foo"]).finish());
        session.create_record(record.clone().tags(vec!["bar"]).finish());

        let _amount = select_budget_amount(&budget, session.conn()).unwrap();

        // TODO: make it work
        // assert_eq!(BigDecimal::one(), _amount);
    }
}
