use bigdecimal::{BigDecimal, Zero};
use chrono::{Datelike, Local, NaiveDate};
use diesel::prelude::*;

use crate::apps::index_response::Data;
use crate::db::{
    models::{Budget, SerializedBudget},
    pagination::*,
    schema::budgets_budget,
    DatabaseQuery, PooledConnection,
};
use crate::errors::DbResult;

pub type GetBudgetsResult = DbResult<Data<SerializedBudget>>;

#[derive(Clone)]
pub struct GetBudgets {
    pub user_id: i32,
    pub page: i64,
    pub per_page: i64,
}

impl DatabaseQuery for GetBudgets {
    type Data = Data<SerializedBudget>;

    fn execute(&self, connection: PooledConnection) -> GetBudgetsResult {
        handle(self, &connection)
    }
}

fn budget_spent(budget: &Budget, connection: &PooledConnection) -> DbResult<BigDecimal> {
    use crate::db::schema::records_record;
    use diesel::dsl::{not, sum};

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

    let query_result = match budget.tags_type.as_str() {
        "INCL" => query
            .filter(records_record::tags.overlaps_with(&budget.tags))
            .first::<Option<BigDecimal>>(connection)?,
        "EXCL" => query
            .filter(not(records_record::tags.overlaps_with(&budget.tags)))
            .first::<Option<BigDecimal>>(connection)?,
        _ => query.first::<Option<BigDecimal>>(connection)?,
    };

    Ok(query_result.unwrap_or_else(BigDecimal::zero).with_scale(2))
}

fn ndays_in_the_current_month(today: NaiveDate) -> u32 {
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

// TODO: add tests
fn serialize_budget(budget: Budget, conn: &PooledConnection) -> DbResult<SerializedBudget> {
    use bigdecimal::ToPrimitive;

    let mut res = SerializedBudget::default();
    let today = Local::today().naive_local();
    let spent = budget_spent(&budget, conn)?;
    let days_in_this_month = ndays_in_the_current_month(today);

    // we need to take into account spendings for today
    let rest_days = days_in_this_month - today.day0() + 1;

    let left = (budget.amount.clone() - spent.clone())
        .to_f64()
        .unwrap_or(0.0);

    res.spent = spent.to_f64().unwrap_or(0.0);
    res.left = left;
    res.average_per_day = (budget.amount.clone() / BigDecimal::from(days_in_this_month))
        .to_f64()
        .unwrap_or(0.0);

    res.left_average_per_day = left / rest_days.to_f64().unwrap_or(0.0f64);
    res.name = budget.name;
    res.amount = budget.amount;

    Ok(res)
}

fn get_page_of_budgets(msg: &GetBudgets, conn: &PooledConnection) -> DbResult<(Vec<Budget>, i64)> {
    let query = budgets_budget::table
        .select(budgets_budget::all_columns)
        .filter(budgets_budget::user_id.eq(msg.user_id))
        .order(budgets_budget::id.asc())
        .paginate(msg.page)
        .per_page(msg.per_page);

    let query_results = query.load::<(Budget, i64)>(conn)?;

    let total = query_results.get(0).map(|x| x.1).unwrap_or(0);

    let results: Vec<Budget> = query_results.into_iter().map(|x| x.0).collect();

    Ok((results, total))
}

fn handle(msg: &GetBudgets, conn: &PooledConnection) -> GetBudgetsResult {
    let (results, total) = get_page_of_budgets(&msg, conn)?;
    let total_pages = (total as f64 / msg.per_page as f64).ceil() as i64;

    let results = results
        .into_iter()
        .map(|budget| serialize_budget(budget, conn))
        .collect::<DbResult<Vec<SerializedBudget>>>()?;

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
mod tests;
