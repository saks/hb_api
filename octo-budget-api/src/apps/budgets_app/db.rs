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

// fn serialize_budget(budget: Budget) -> SerializedBudget {
fn serialize_budget(budget: Budget, _conn: &PgConnection) -> SerializedBudget {
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

    let x = query.first::<(Option<BigDecimal>)>(_conn);

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

    SerializedBudget::default()
}

impl Handler<GetBudgetsMessage> for DbExecutor {
    type Result = GetBudgetsResult;

    fn handle(&mut self, msg: GetBudgetsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &self.pool.get()?;

        let query = budgets_budget::table
            .select(budgets_budget::all_columns)
            .filter(budgets_budget::user_id.eq(msg.user_id))
            .order(budgets_budget::name.asc())
            .paginate(msg.page)
            .per_page(msg.per_page);

        let query_results = query.load::<(Budget, i64)>(&*connection)?;

        let total = query_results.get(0).map(|x| x.1).unwrap_or(0);
        let total_pages = (total as f64 / msg.per_page as f64).ceil() as i64;

        let results = query_results
            .into_iter()
            .map(|x| serialize_budget(x.0, &*connection))
            .collect();

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
