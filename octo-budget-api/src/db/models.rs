use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{NaiveDate, NaiveDateTime};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_derive::Serialize;

use crate::db::schema::{auth_user, budgets_budget, records_record};

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
}

#[derive(Default, Clone)]
pub struct RecordBuilder {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub id: i32,
    pub tags: Vec<String>,
    pub transaction_type: String,
    pub user_id: i32,
}

impl RecordBuilder {
    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn transaction_type(mut self, transaction_type: &str) -> Self {
        self.transaction_type = transaction_type.to_string();
        self
    }

    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = BigDecimal::from(amount);
        self
    }

    pub fn finish(self) -> Record {
        use chrono::offset::Local;

        Record {
            id: self.id,
            amount: self.amount,
            amount_currency: self.amount_currency,
            created_at: Local::now().naive_local(),
            tags: self.tags,
            transaction_type: self.transaction_type,
            user_id: self.user_id,
        }
    }
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

#[derive(Default)]
pub struct BudgetBuilder {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub id: i32,
    pub name: String,
    pub tags: Vec<String>,
    pub tags_type: String,
    pub user_id: i32,
}

impl BudgetBuilder {
    pub fn tags_type(mut self, tags_type: &str) -> Self {
        self.tags_type = tags_type.to_string();
        self
    }

    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn user_id(mut self, user_id: i32) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn finish(self) -> Budget {
        use chrono::naive::NaiveDate;

        Budget {
            amount: self.amount,
            amount_currency: self.amount_currency,
            id: self.id,
            name: self.name,
            tags: self.tags,
            tags_type: self.tags_type,
            user_id: self.user_id,
            start_date: NaiveDate::from_ymd(2015, 3, 14),
        }
    }
}

#[derive(Serialize, Default)]
pub struct SerializedBudget {
    pub name: String,
    pub amount: f64,
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
        state.end()
    }
}
