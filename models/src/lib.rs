#[macro_use]
extern crate diesel;

use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{NaiveDate, NaiveDateTime};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

pub mod schema;
use schema::{auth_user, budgets_budget, records_record};

#[derive(Queryable, Serialize, Debug, Clone, PartialEq, Insertable)]
#[table_name = "auth_user"]
pub struct AuthUser {
    pub date_joined: NaiveDateTime,
    pub email: String,
    pub first_name: String,
    pub id: i32,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub last_name: String,
    pub password: String,
    pub tags: Vec<String>,
    pub username: String,
}

#[derive(Queryable, Debug, Clone, PartialEq, Insertable)]
#[table_name = "records_record"]
pub struct Record {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub created_at: NaiveDateTime,
    pub id: i32,
    pub tags: Vec<String>,
    pub transaction_type: String,
    pub user_id: i32,
    pub comment: Option<String>,
}

#[derive(Queryable, Debug, Clone, PartialEq, Insertable)]
#[table_name = "budgets_budget"]
pub struct Budget {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub id: i32,
    pub name: String,
    pub start_date: NaiveDate,
    pub tags: Vec<String>,
    pub tags_type: String,
    pub user_id: i32,
}

#[derive(Serialize, Default)]
pub struct SerializedBudget {
    pub name: String,
    pub amount: BigDecimal,
    pub spent: f64,
    pub left: f64,
    pub average_per_day: f64,
    pub left_average_per_day: f64,
}

#[derive(Debug, Serialize)]
enum CurrencyCode {
    #[serde(rename = "CAD")]
    Cad,
}

#[derive(Debug, Serialize)]
enum CurrencyName {
    #[serde(rename = "Canadian Dollar")]
    Cad,
}

#[derive(Debug, Serialize)]
struct Currency {
    code: CurrencyCode,
    name: CurrencyName,
}

#[derive(Debug, Serialize)]
struct Amount {
    amount: f64,
    currency: Currency,
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Record", 6)?;

        let currency = Currency {
            code: CurrencyCode::Cad,
            name: CurrencyName::Cad,
        };
        let amount = Amount {
            amount: self.amount.to_f64().unwrap(),
            currency,
        };

        state.serialize_field("amount", &amount)?;
        state.serialize_field("created_at", &self.created_at.timestamp())?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("tags", &self.tags)?;
        state.serialize_field("transaction_type", &self.transaction_type)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("comment", &self.comment)?;
        state.end()
    }
}
