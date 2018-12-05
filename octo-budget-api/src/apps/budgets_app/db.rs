use std::result;

use actix::{Handler, Message};
use bigdecimal::{BigDecimal, Zero};
use chrono::NaiveDate;
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

fn budget_spent(budget: &Budget, conn: &PgConnection) -> Result<BigDecimal, Error> {
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

    match budget.tags_type.as_str() {
        "INCL" => {
            let query = query.filter(records_record::tags.overlaps_with(&budget.tags));

            Ok(query
                .first::<(Option<BigDecimal>)>(conn)?
                .unwrap_or_else(BigDecimal::zero))
        }
        "EXCL" => {
            use diesel::dsl::not;
            let query = query.filter(not(records_record::tags.overlaps_with(&budget.tags)));
            Ok(query
                .first::<(Option<BigDecimal>)>(conn)?
                .unwrap_or_else(BigDecimal::zero))
        }
        _ => Ok(query
            .first::<(Option<BigDecimal>)>(conn)?
            .unwrap_or_else(BigDecimal::zero)),
    }
}

fn ndays_in_the_current_month(today: NaiveDate) -> u32 {
    use chrono::Datelike;

    let year = today.year();
    let month = today.month();

    // the first day of the next month...
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let d = NaiveDate::from_ymd(y, m, 1);

    // ...is preceded by the last day of the original month
    d.pred().day()
}

fn serialize_budget(budget: Budget, conn: &PgConnection) -> Result<SerializedBudget, Error> {
    use bigdecimal::ToPrimitive;
    use chrono::Local;

    let mut res = SerializedBudget::default();
    let today = Local::today().naive_local();
    let spent = budget_spent(&budget, conn)?;
    let days_in_this_month = ndays_in_the_current_month(today);

    res.spent = spent.to_f64().unwrap_or(0.0);
    res.left = (budget.amount.clone() - spent).to_f64().unwrap_or(0.0);
    res.average_per_day = (budget.amount / BigDecimal::from(days_in_this_month))
        .to_f64()
        .unwrap_or(0.0);
    // res.left_average_per_day =
    // rest_days = days - date.today(
    // ).day + 1  # we need to take into account spendings for today
    // return (self.left / rest_days).quantize(
    //     Decimal('.01'), rounding=ROUND_DOWN)

    Ok(res)
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
    fn amount_aggregation_with_other_tags_type() {
        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        let budget = BudgetBuilder::default().user_id(user.id).finish();

        let record = RecordBuilder::default()
            .user_id(user.id)
            .transaction_type("EXP");

        session.create_record(record.clone().amount(1.0).finish());

        let amount = budget_spent(&budget, session.conn()).unwrap();

        assert_eq!(BigDecimal::from(1), amount);
    }

    #[test]
    fn amount_aggregation_with_including_tags() {
        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        let budget = BudgetBuilder::default()
            .user_id(user.id)
            .tags_type("INCL")
            .tags(vec!["foo"])
            .finish();

        let record = RecordBuilder::default()
            .user_id(user.id)
            .transaction_type("EXP");

        for (amount, tag) in [(1.0, "foo"), (3.0, "foo"), (2.0, "bar")].iter() {
            session.create_record(record.clone().amount(*amount).tags(vec![tag]).finish());
        }

        let amount = budget_spent(&budget, session.conn()).unwrap();

        assert_eq!(BigDecimal::from(4), amount);
    }

    #[test]
    fn amount_aggregation_with_excluding_tags() {
        let mut session = Session::new();
        let user = session.create_user("ok auth user", "dummy password");
        let budget = BudgetBuilder::default()
            .user_id(user.id)
            .tags_type("EXCL")
            .tags(vec!["foo"])
            .finish();

        let record = RecordBuilder::default()
            .user_id(user.id)
            .transaction_type("EXP");

        for (amount, tag) in [(1.0, "foo"), (3.0, "foo"), (2.0, "bar"), (4.0, "bar")].iter() {
            session.create_record(record.clone().amount(*amount).tags(vec![tag]).finish());
        }

        let amount = budget_spent(&budget, session.conn()).unwrap();

        assert_eq!(BigDecimal::from(6), amount);
    }
}
