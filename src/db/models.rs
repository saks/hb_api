use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::NaiveDateTime;
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Queryable, Serialize, Debug, Clone, PartialEq)]
pub struct AuthUser {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Queryable, Debug, Clone, PartialEq)]
pub struct Record {
    pub amount: BigDecimal,
    pub amount_currency: String,
    pub created_at: NaiveDateTime,
    pub id: i32,
    pub tags: Vec<String>,
    pub transaction_type: String,
    pub user_id: i32,
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
